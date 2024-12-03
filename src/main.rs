mod elapsed;
mod middleware;
mod routes;
mod storage;

use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    handler::Handler,
    middleware as axum_middleware,
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use clap_duration::duration_range_value_parse;
use dotenvy::dotenv;
use duration_human::{DurationHuman, DurationHumanValidator};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use storage::StorageHandler;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer,
    normalize_path::NormalizePathLayer,
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::{debug, info, Level};
use tracing_subscriber::EnvFilter;
use url::Url;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Arguments {
    /// The socket address that the server should be exposed on.
    #[arg(
        long = "address",
        env = "DOLLHOUSE_ADDRESS",
        default_value = "127.0.0.1:8731"
    )]
    address: SocketAddr,

    /// The public url that this server will be exposed as to the internet.
    ///
    /// This only impacts what base is used when links are sent to users.
    /// You'll need to handle the reverse proxy yourself.
    #[arg(
        long = "public-url",
        env = "DOLLHOUSE_PUBLIC_URL",
        default_value = "http://127.0.0.1:8731"
    )]
    public_url: Url,

    /// The bearer token to use when interacting with authenticated endpoints.
    #[clap(long = "token", env = "DOLLHOUSE_TOKEN")]
    token: String,

    /// The amount of time since last access that can elapse before a file is automatically purged from storage.
    #[clap(long = "expiry-time", env = "DOLLHOUSE_EXPIRY_TIME", default_value="31 days", value_parser = duration_range_value_parse!(min: 1min, max: 500years))]
    expiry_time: DurationHuman,

    /// The interval to run the expiry check on.
    ///
    /// This may be an intensive operation if you store thousands of files with long expiry times.
    #[clap(long = "expiry-interval", env = "DOLLHOUSE_EXPIRY_INTERVAL", default_value="60 min", value_parser = duration_range_value_parse!(min: 1min, max: 1day))]
    expiry_interval: DurationHuman,

    /// Where all uploads should be stored locally.
    ///
    /// This directory should ONLY contain uploads as it is automatically purged and exposed to the internet.
    #[clap(
        long = "uploads-path", 
        env = "DOLLHOUSE_UPLOADS_PATH",
        default_value = dirs::data_local_dir().unwrap().join("dollhouse").join("uploads").into_os_string()
    )]
    uploads_path: PathBuf,

    /// The maximum size of file that can be uploaded in bytes.
    #[clap(
        long = "upload-limit", 
        env = "DOLLHOUSE_UPLOAD_LIMIT",
        default_value_t = 50 * 1000 * 1000
    )]
    upload_limit_bytes: usize,

    /// Whether to enforce uploads be of either the `image/*` or `video/*` MIME type.
    ///
    /// MIME types are determined by the magic numbers of uploaded content.
    /// This process is not perfect but will fail-closed on unknown media types.
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
    public_url: Url,
    limit_to_media: bool,
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .with_thread_ids(true)
        .init();
    let args = Arguments::parse();

    let storage =
        Arc::new(StorageHandler::new(&args.uploads_path, Duration::from(&args.expiry_time)).await?);

    let router = Router::new()
        .route("/", get(routes::index_handler))
        .route("/health", get(routes::health_handler))
        .route(
            "/api/upload",
            post(
                routes::uploads::create_upload_response
                    .layer(DefaultBodyLimit::max(args.upload_limit_bytes)),
            ),
        )
        .route(
            "/api/upload/:id",
            delete(routes::uploads::delete_image_handler),
        )
        .nest_service("/uploads", ServeDir::new(&args.uploads_path))
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
            public_url: args.public_url.clone(),
            limit_to_media: args.limit_to_media,
            token: args.token,
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
