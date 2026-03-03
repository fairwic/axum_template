//! Auth integration traits

use async_trait::async_trait;

use axum_core_kernel::AppResult;

#[derive(Debug, Clone)]
pub struct WechatSession {
    pub openid: String,
    pub session_key: String,
    pub unionid: Option<String>,
}

#[async_trait]
pub trait WechatAuthClient: Send + Sync {
    async fn code2session(&self, code: &str) -> AppResult<WechatSession>;
}

#[async_trait]
pub trait SmsGateway: Send + Sync {
    async fn send_login_code(&self, phone: &str, code: &str) -> AppResult<()>;
}
