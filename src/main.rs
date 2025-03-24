mod cryptography;
mod middleware;
mod mime;
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
use mime_guess::{Mime, mime::IMAGE_STAR};
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use storage::StorageHandler;
use tokio::{net::TcpListener, signal};
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

    /// A path to the directory where data should be stored.
    ///
    /// CAUTION: This directory should not be used for anything else as it and all subdirectories will be automatically managed.
    #[clap(
        long = "data-path", 
        env = "DOLLHOUSE_DATA_PATH",
        default_value = dirs::data_local_dir().unwrap().join("dollhouse").into_os_string()
    )]
    data_path: PathBuf,

    /// The amount of time since last access before a file is automatically purged from storage.
    #[clap(long = "upload-expiry-time", env = "DOLLHOUSE_UPLOAD_EXPIRY_TIME", default_value="31 days", value_parser = duration_range_value_parse!(min: 1min, max: 100years))]
    upload_expiry_time: DurationHuman,

    /// The interval to run the expiry check on.
    ///
    /// This may be an intensive operation if you store thousands of files with long expiry times.
    #[clap(long = "upload-expiry-interval", env = "DOLLHOUSE_UPLOAD_EXPIRY_INTERVAL", default_value="60 min", value_parser = duration_range_value_parse!(min: 1min, max: 100years))]
    upload_expiry_interval: DurationHuman,

    /// The maximum file size that is allowed to be uploaded.
    #[clap(
        long = "upload-size-limit",
        env = "DOLLHOUSE_UPLOAD_SIZE_LIMIT",
        default_value = "50MB"
    )]
    upload_size_limit: ByteSize,

    /// File mimetypes that are allowed to be uploaded.
    /// Supports type wildcards (e.g. 'image/*', '*/*').
    ///
    /// MIME types are determined by the magic numbers of uploaded content, if the mimetype cannot be determined the file will be rejected.
    #[clap(
        long = "upload-mimetypes",
        env = "DOLLHOUSE_UPLOAD_MIMETYPES",
        default_values_t = [
            IMAGE_STAR,
            Mime::from_str("video/*").unwrap()
        ],
        value_delimiter = ','
    )]
    upload_mimetypes: Vec<Mime>,
}

#[derive(Debug, Clone)]
struct AppState {
    storage: Arc<StorageHandler>,
    /// Base URL for use when returning public facing links.
    public_base_url: Url,
    /// File mimetypes that are allowed to be uploaded.
    /// Supports type wildcards (e.g. 'image/*', '*/*').
    upload_allowed_mimetypes: Vec<Mime>,
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
        Duration::from(&args.upload_expiry_time),
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
                    args.upload_size_limit
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
            upload_allowed_mimetypes: args.upload_mimetypes,
            auth_tokens: args.tokens,
            persisted_salt: persisted_salt,
        });

    // File expiry background task.
    let storage_clone = Arc::clone(&storage);
    tokio::spawn(async move {
        loop {
            debug!("Running check to find expired files");
            storage_clone.remove_expired_files().unwrap();
            tokio::time::sleep(Duration::from(&args.upload_expiry_interval)).await;
        }
    });

    let tcp_listener = TcpListener::bind(args.address).await?;
    info!(
        "Internal server listening on http://{} and exposed as {}",
        args.address, args.public_url
    );
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// https://github.com/tokio-rs/axum/blob/15917c6dbcb4a48707a20e9cfd021992a279a662/examples/graceful-shutdown/src/main.rs#L55
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
