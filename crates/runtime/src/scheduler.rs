use std::time::Duration;

use axum_api::state::AppState;

pub fn spawn_order_jobs(state: AppState, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            let config = state.biz_config.read().await.clone();

            if let Some(order_service) = state.order_service.clone() {
                match order_service
                    .auto_close_unpaid_orders(config.pay_timeout_secs)
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
                    .auto_accept_pending_orders(config.auto_accept_secs)
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

            if let Some(runner_order_service) = state.runner_order_service.clone() {
                match runner_order_service
                    .auto_close_unpaid_orders(config.pay_timeout_secs)
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
                    .auto_accept_pending_orders(config.auto_accept_secs)
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
    })
}
