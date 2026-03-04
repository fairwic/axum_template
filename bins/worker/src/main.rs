use std::time::Duration;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use axum_infra::AppConfig;
use axum_runtime::{build_app_state, spawn_order_jobs};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    tracing::info!("Starting worker process...");

    let config = AppConfig::load().context("Failed to load configuration")?;
    tracing::info!(host = %config.server.host, port = %config.server.port, "Configuration loaded");

    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.database.max_lifetime_secs))
        .connect(config.database.url())
        .await
        .context("Failed to connect to database")?;

    let state = build_app_state(pool, &config).await?;
    let worker_handle = spawn_order_jobs(state, 30);

    tracing::info!("Worker scheduler loop started (interval=30s)");
    shutdown_signal().await;

    worker_handle.abort();
    let _ = worker_handle.await;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "info,axum_api=debug,axum_application=debug,axum_infrastructure=debug,sqlx=warn".into()
        }))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_span_events(
                    tracing_subscriber::fmt::format::FmtSpan::NEW
                        | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
                ),
        )
        .init();
}

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

    tracing::info!("Signal received, shutting down");
}
