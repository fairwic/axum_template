//! Backend Template Server

mod bootstrap;

use std::time::Duration;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use axum_api::create_router;
use axum_api::state::AppState;
use axum_infrastructure::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    tracing::info!("Starting backend template server...");

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

    // sqlx::migrate!("../../migrations")
    //     .run(&pool)
    //     .await
    //     .context("Failed to run database migrations")?;

    let state = bootstrap::build_app_state(pool, &config).await?;
    spawn_order_jobs(state.clone());
    let app = create_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    tracing::info!("Server running on http://{}", addr);
    tracing::info!("Swagger UI: http://{}/swagger-ui", addr);
    tracing::info!("Health: http://{}/health", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    Ok(())
}

fn spawn_order_jobs(state: AppState) {
    let job_state = state.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(30));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            if let Some(order_service) = job_state.order_service.clone() {
                match order_service
                    .auto_close_unpaid_orders(job_state.biz_config.pay_timeout_secs)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::info!(closed_goods_orders = count, "Closed unpaid goods orders");
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!(error = ?err, "Failed to close unpaid goods orders");
                    }
                }

                match order_service
                    .auto_accept_pending_orders(job_state.biz_config.auto_accept_secs)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::info!(
                            auto_accepted_goods_orders = count,
                            "Auto accepted goods orders"
                        );
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!(error = ?err, "Failed to auto accept goods orders");
                    }
                }
            }

            if let Some(runner_order_service) = job_state.runner_order_service.clone() {
                match runner_order_service
                    .auto_close_unpaid_orders(job_state.biz_config.pay_timeout_secs)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::info!(closed_runner_orders = count, "Closed unpaid runner orders");
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!(error = ?err, "Failed to close unpaid runner orders");
                    }
                }

                match runner_order_service
                    .auto_accept_pending_orders(job_state.biz_config.auto_accept_secs)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::info!(
                            auto_accepted_runner_orders = count,
                            "Auto accepted runner orders"
                        );
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!(error = ?err, "Failed to auto accept runner orders");
                    }
                }
            }
        }
    });
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,axum_api=debug,axum_application=debug,axum_infrastructure=debug,tower_http=debug,sqlx=warn"
                    .into()
            }),
        )
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
