//! User service

use std::sync::Arc;

use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use axum_domain::auth::{SmsGateway, WechatAuthClient};
use axum_domain::cache::CacheService;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use rand::Rng;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
    wechat_auth: Arc<dyn WechatAuthClient>,
    sms_gateway: Arc<dyn SmsGateway>,
    cache: Arc<dyn CacheService>,
    sms_code_ttl_secs: u64,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self {
            repo,
            wechat_auth: Arc::new(NoopWechatAuthClient),
            sms_gateway: Arc::new(NoopSmsGateway),
            cache: Arc::new(NoopCacheService),
            sms_code_ttl_secs: 300,
        }
    }

    pub fn with_auth(
        mut self,
        wechat_auth: Arc<dyn WechatAuthClient>,
        cache: Arc<dyn CacheService>,
        sms_gateway: Arc<dyn SmsGateway>,
        sms_code_ttl_secs: u64,
    ) -> Self {
        self.wechat_auth = wechat_auth;
        self.cache = cache;
        self.sms_gateway = sms_gateway;
        self.sms_code_ttl_secs = sms_code_ttl_secs;
        self
    }

    pub async fn login_with_openid(
        &self,
        openid: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> AppResult<User> {
        if let Some(user) = self.repo.find_by_openid(&openid).await? {
            return Ok(user);
        }
        let user = User::new(openid, nickname, avatar)?;
        self.repo.create(&user).await
    }

    pub async fn login_with_wechat_code(
        &self,
        code: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> AppResult<User> {
        let code = code.trim();
        if code.is_empty() {
            return Err(AppError::Validation("code 不能为空".into()));
        }

        let session = self.wechat_auth.code2session(code).await?;
        self.login_with_openid(session.openid, nickname, avatar)
            .await
    }

    pub async fn send_login_sms_code(&self, phone: String) -> AppResult<()> {
        let phone = Self::normalize_phone(&phone)?;
        let code = Self::generate_sms_code();
        let cache_key = Self::sms_cache_key(&phone);

        self.cache
            .set_string(&cache_key, &code, self.sms_code_ttl_secs)
            .await?;

        if let Err(err) = self.sms_gateway.send_login_code(&phone, &code).await {
            let _ = self.cache.delete(&cache_key).await;
            return Err(err);
        }

        Ok(())
    }

    pub async fn login_with_phone_sms(
        &self,
        phone: String,
        sms_code: String,
        wechat_code: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> AppResult<User> {
        let phone = Self::normalize_phone(&phone)?;
        self.verify_sms_code(&phone, &sms_code).await?;

        let mut user = self
            .login_with_wechat_code(wechat_code, nickname, avatar)
            .await?;

        if let Some(existing_phone_user) = self.repo.find_by_phone(&phone).await? {
            if existing_phone_user.id != user.id {
                return Err(AppError::Conflict("手机号已绑定其他账号".into()));
            }
            return Ok(existing_phone_user);
        }

        user = self.repo.bind_phone(user.id, phone).await?;
        Ok(user)
    }

    fn normalize_phone(phone: &str) -> AppResult<String> {
        let phone = phone.trim();
        let valid = phone.len() == 11
            && phone.starts_with('1')
            && phone.chars().all(|char| char.is_ascii_digit());
        if !valid {
            return Err(AppError::Validation("手机号格式不正确".into()));
        }
        Ok(phone.to_string())
    }

    fn generate_sms_code() -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1_000_000))
    }

    fn sms_cache_key(phone: &str) -> String {
        format!("auth:sms:login:{phone}")
    }

    async fn verify_sms_code(&self, phone: &str, sms_code: &str) -> AppResult<()> {
        let sms_code = sms_code.trim();
        if sms_code.is_empty() {
            return Err(AppError::Validation("短信验证码不能为空".into()));
        }

        let cache_key = Self::sms_cache_key(phone);
        let cached_code = self.cache.get_string(&cache_key).await?;
        match cached_code {
            Some(code) if code == sms_code => {
                self.cache.delete(&cache_key).await?;
                Ok(())
            }
            _ => Err(AppError::Validation("短信验证码错误或已过期".into())),
        }
    }
}

struct NoopWechatAuthClient;

#[async_trait]
impl WechatAuthClient for NoopWechatAuthClient {
    async fn code2session(&self, _code: &str) -> AppResult<axum_domain::WechatSession> {
        Err(AppError::Internal(
            "wechat auth client not configured".into(),
        ))
    }
}

struct NoopSmsGateway;

#[async_trait]
impl SmsGateway for NoopSmsGateway {
    async fn send_login_code(&self, _phone: &str, _code: &str) -> AppResult<()> {
        Err(AppError::Internal("sms gateway not configured".into()))
    }
}

struct NoopCacheService;

#[async_trait]
impl CacheService for NoopCacheService {
    async fn get_string(&self, _key: &str) -> AppResult<Option<String>> {
        Err(AppError::Internal("cache service not configured".into()))
    }

    async fn set_string(&self, _key: &str, _value: &str, _ttl_secs: u64) -> AppResult<()> {
        Err(AppError::Internal("cache service not configured".into()))
    }

    async fn delete(&self, _key: &str) -> AppResult<()> {
        Err(AppError::Internal("cache service not configured".into()))
    }
}
