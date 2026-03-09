#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use nibe_exporter::{
    config::Config,
    metrics::{self, MetricsStore},
    myuplink::MyUplinkClient,
    server::AppState,
    server::build_router,
    telemetry,
};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let mut config = Config::load();
    config.validate()?;

    // Initialize telemetry
    telemetry::init_telemetry(&config.log_level, config.log_json);

    info!("Starting nibe-exporter");
    info!("API version: {}", config.api_version);
    info!("Listen: {}", config.listen_addr);
    info!("Poll interval: {}s", config.poll_interval);

    // Create myUplink client
    let client = Arc::new(MyUplinkClient::new(
        config.client_id().to_string(),
        config.client_secret().to_string(),
        config.api_version.clone(),
    )?);

    info!("myUplink client configured");

    // Create metrics store
    let metrics_store = Arc::new(MetricsStore::new());

    // Spawn polling loop
    let poll_loop = metrics::spawn_poll_loop(
        metrics_store.clone(),
        client.clone(),
        config.device_id().map(std::string::ToString::to_string),
        config.poll_interval,
    );

    info!("Metrics polling loop spawned");

    // Build HTTP server
    let app_state = AppState {
        metrics_store: metrics_store.clone(),
    };
    let router = build_router(app_state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    info!("HTTP server listening on {}", config.listen_addr);

    // Run server with graceful shutdown
    let server = axum::serve(listener, router);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    match graceful.await {
        Ok(()) => info!("HTTP server shut down gracefully"),
        Err(e) => error!("HTTP server error: {}", e),
    }

    // Cancel polling loop
    poll_loop.abort();

    info!("nibe-exporter shut down");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        #[allow(clippy::expect_used)]
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        #[allow(clippy::expect_used)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => info!("Received Ctrl+C"),
        () = terminate => info!("Received SIGTERM"),
    }
}
