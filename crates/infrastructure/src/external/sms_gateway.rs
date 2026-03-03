//! SMS gateway implementation

use async_trait::async_trait;
use axum_common::AppResult;
use axum_domain::SmsGateway;

#[derive(Clone, Default)]
pub struct LogSmsGateway;

#[async_trait]
impl SmsGateway for LogSmsGateway {
    async fn send_login_code(&self, phone: &str, code: &str) -> AppResult<()> {
        tracing::info!(phone = %phone, code = %code, "send sms login code via log gateway");
        Ok(())
    }
}
