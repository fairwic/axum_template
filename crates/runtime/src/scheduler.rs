use std::time::Duration;

use axum_api::state::AppState;

pub fn spawn_order_jobs(_state: AppState, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            print!("job loop");
        }
    })
}
