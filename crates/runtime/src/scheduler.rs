use std::time::Duration;

use axum_api::state::AppState;

/// 启动后台任务调度循环。
///
/// 脚手架占位实现：按固定间隔触发心跳，业务任务在此处按需接入。
pub fn spawn_background_jobs(_state: AppState, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            tracing::debug!("background job tick");
        }
    })
}
