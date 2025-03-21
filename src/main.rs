mod cryptography;
mod middleware;
mod routes;
mod storage;

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    handler::Handler,
    middleware as axum_middleware,
    routing::{delete, get, post},
};
use bytesize::ByteSize;
use clap::Parser;
use clap_duration::duration_range_value_parse;
use cryptography::Cryptography;
use dotenvy::dotenv;
use duration_human::{DurationHuman, DurationHumanValidator};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use storage::StorageHandler;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer,
    normalize_path::NormalizePathLayer,
    trace::{self, TraceLayer},
};
use tracing::{Level, debug, info};
use tracing_subscriber::EnvFilter;
use url::Url;

const UPLOADS_DIRNAME: &str = "uploads";
const PERSISTED_SALT_FILENAME: &str = "persisted_salt";

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Arguments {
    /// The internet socket address that the server should be ran on.
    #[arg(
        long = "address",
        env = "DOLLHOUSE_ADDRESS",
        default_value = "127.0.0.1:8731"
    )]
    address: SocketAddr,

    /// The base url to use when generating links to uploads.
    ///
    /// This is only for link generation, you'll need to handle the reverse proxy yourself.
    #[arg(
        long = "public-url",
        env = "DOLLHOUSE_PUBLIC_URL",
        default_value = "http://127.0.0.1:8731"
    )]
    public_url: Url,

    /// One or more bearer tokens to use when interacting with authenticated endpoints.
    #[clap(
        long = "tokens",
        env = "DOLLHOUSE_TOKENS",
        required = true,
        value_delimiter = ','
    )]
    tokens: Vec<String>,

    /// The amount of time since last access before a file is automatically purged from storage.
    #[clap(long = "expiry-time", env = "DOLLHOUSE_EXPIRY_TIME", default_value="31 days", value_parser = duration_range_value_parse!(min: 1min, max: 100years))]
    expiry_time: DurationHuman,

    /// The interval to run the expiry check on.
    ///
    /// This may be an intensive operation if you store thousands of files with long expiry times.
    #[clap(long = "expiry-interval", env = "DOLLHOUSE_EXPIRY_INTERVAL", default_value="60 min", value_parser = duration_range_value_parse!(min: 1min, max: 100years))]
    expiry_interval: DurationHuman,

    /// A path to the directory where data should be stored.
    ///
    /// CAUTION: This directory should not be used for anything else as it and all subdirectories will be automatically managed.
    #[clap(
        long = "data-path", 
        env = "DOLLHOUSE_DATA_PATH",
        default_value = dirs::data_local_dir().unwrap().join("dollhouse").into_os_string()
    )]
    data_path: PathBuf,

    /// The maximum allowed filesize for all uploads.
    #[clap(
        long = "upload-limit",
        env = "DOLLHOUSE_UPLOAD_LIMIT",
        default_value = "50MB"
    )]
    upload_limit: ByteSize,

    /// Enforce uploads be of either the `image/*` or `video/*` MIME type.
    ///
    //  MIME types are determined by the magic numbers of uploaded content, if the mimetype cannot be determined the file will be rejected.
    #[clap(
        long = "limit-to-media",
        env = "DOLLHOUSE_LIMIT_TO_MEDIA",
        default_value_t = true
    )]
    limit_to_media: std::primitive::bool,
}

#[derive(Debug, Clone)]
struct AppState {
    storage: Arc<StorageHandler>,
    /// Base URL for use when returning public facing links.
    public_base_url: Url,
    /// Reject files that are not of image/* or video/* types.
    limit_to_media: bool,
    /// Collection of bearer tokens for actions that require authentication.
    auth_tokens: Vec<String>,
    /// Used for all hash operations to avoid rainbow tables.
    persisted_salt: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    // Init required state.
    let storage = Arc::new(StorageHandler::new(
        &args.data_path.join(UPLOADS_DIRNAME),
        Duration::from(&args.expiry_time),
    )?);
    let persisted_salt = {
        let path = args.data_path.join(PERSISTED_SALT_FILENAME);
        if let Some(salt) = Cryptography::get_persisted_salt(&path)? {
            salt
        } else {
            Cryptography::create_persisted_salt(&path)?
        }
    };

    let router = Router::new()
        .route("/", get(routes::index_handler))
        .route("/health", get(routes::health_handler))
        .route(
            "/api/upload",
            post(
                routes::uploads::create_upload_handler.layer(DefaultBodyLimit::max(
                    args.upload_limit
                        .0
                        .try_into()
                        .context("upload limit does not fit into usize")?,
                )),
            ),
        )
        .route(
            "/api/upload/{id}",
            delete(routes::uploads::delete_image_handler),
        )
        .route("/uploads/{id}", get(routes::uploads::get_upload_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new())
        .layer(axum_middleware::from_fn(middleware::header_middleware))
        .with_state(AppState {
            storage: Arc::clone(&storage),
            public_base_url: args.public_url.clone(),
            limit_to_media: args.limit_to_media,
            auth_tokens: args.tokens,
            persisted_salt: persisted_salt,
        });

    // File expiry background task.
    let storage_clone = Arc::clone(&storage);
    tokio::spawn(async move {
        loop {
            debug!("Running check to find expired files");
            storage_clone.remove_expired_files().unwrap();
            tokio::time::sleep(Duration::from(&args.expiry_interval)).await;
        }
    });

    // Start webserver.
    let tcp_listener = TcpListener::bind(args.address).await?;
    info!(
        "Internal server listening on http://{} and exposed as {}",
        args.address, args.public_url
    );
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl-c");
        })
        .await?;

    Ok(())
}
