use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::UserService;
use axum_core_kernel::AppResult;
use axum_domain::auth::{SmsGateway, WechatAuthClient, WechatSession};
use axum_domain::cache::CacheService;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryUserRepo {
    by_openid: Mutex<HashMap<String, User>>,
    by_phone: Mutex<HashMap<String, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let guard = self.by_openid.lock().await;
        Ok(guard.get(openid).cloned())
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        let guard = self.by_phone.lock().await;
        Ok(guard.get(phone).cloned())
    }

    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>> {
        let guard = self.by_openid.lock().await;
        Ok(guard.values().find(|item| item.id == user_id).cloned())
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let mut openid_guard = self.by_openid.lock().await;
        openid_guard.insert(user.openid.clone(), user.clone());
        drop(openid_guard);

        if let Some(phone) = &user.phone {
            let mut phone_guard = self.by_phone.lock().await;
            phone_guard.insert(phone.clone(), user.clone());
        }
        Ok(user.clone())
    }

    async fn bind_phone(&self, user_id: Ulid, phone: String) -> AppResult<User> {
        let mut openid_guard = self.by_openid.lock().await;
        let entry = openid_guard
            .values_mut()
            .find(|item| item.id == user_id)
            .expect("user should exist");
        entry.phone = Some(phone.clone());
        let updated = entry.clone();
        drop(openid_guard);

        let mut phone_guard = self.by_phone.lock().await;
        phone_guard.insert(phone, updated.clone());
        Ok(updated)
    }

    async fn set_current_store(&self, user_id: Ulid, store_id: Ulid) -> AppResult<User> {
        let mut openid_guard = self.by_openid.lock().await;
        let entry = openid_guard
            .values_mut()
            .find(|item| item.id == user_id)
            .ok_or_else(|| axum_core_kernel::AppError::NotFound("user not found".into()))?;
        entry.current_store_id = Some(store_id);
        let updated = entry.clone();
        drop(openid_guard);

        if let Some(phone) = &updated.phone {
            let mut phone_guard = self.by_phone.lock().await;
            phone_guard.insert(phone.clone(), updated.clone());
        }
        Ok(updated)
    }
}

#[derive(Default)]
struct FakeWechatAuthClient {
    code_to_openid: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl WechatAuthClient for FakeWechatAuthClient {
    async fn code2session(&self, code: &str) -> AppResult<WechatSession> {
        let guard = self.code_to_openid.lock().await;
        let openid = guard.get(code).cloned().unwrap_or_else(|| code.to_string());
        Ok(WechatSession {
            openid,
            session_key: "session-key".into(),
            unionid: None,
        })
    }
}

#[derive(Default)]
struct InMemoryCache {
    inner: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(key).cloned())
    }

    async fn set_string(&self, key: &str, value: &str, _ttl_secs: u64) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        guard.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        guard.remove(key);
        Ok(())
    }
}

#[derive(Default)]
struct FakeSmsGateway {
    last_code_by_phone: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl SmsGateway for FakeSmsGateway {
    async fn send_login_code(&self, phone: &str, code: &str) -> AppResult<()> {
        let mut guard = self.last_code_by_phone.lock().await;
        guard.insert(phone.to_string(), code.to_string());
        Ok(())
    }
}

#[tokio::test]
async fn test_wechat_code_login_creates_user() {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let wechat = Arc::new(FakeWechatAuthClient::default());
    let cache: Arc<dyn CacheService> = Arc::new(InMemoryCache::default());
    let sms = Arc::new(FakeSmsGateway::default());

    wechat
        .code_to_openid
        .lock()
        .await
        .insert("wx-code-1".into(), "openid-1".into());

    let service = UserService::new(repo).with_auth(wechat, cache, sms, 300);
    let user = service
        .login_with_wechat_code("wx-code-1".into(), Some("Alice".into()), None)
        .await
        .unwrap();

    assert_eq!(user.openid, "openid-1");
}

#[tokio::test]
async fn test_phone_sms_login_binds_same_wechat_user() {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let wechat = Arc::new(FakeWechatAuthClient::default());
    let cache: Arc<dyn CacheService> = Arc::new(InMemoryCache::default());
    let sms = Arc::new(FakeSmsGateway::default());

    wechat
        .code_to_openid
        .lock()
        .await
        .insert("wx-code-2".into(), "openid-2".into());

    let service = UserService::new(repo).with_auth(wechat.clone(), cache, sms.clone(), 300);

    let wechat_user = service
        .login_with_wechat_code("wx-code-2".into(), None, None)
        .await
        .unwrap();

    service
        .send_login_sms_code("13800000000".into())
        .await
        .unwrap();

    let code = sms
        .last_code_by_phone
        .lock()
        .await
        .get("13800000000")
        .cloned()
        .unwrap();

    let phone_user = service
        .login_with_phone_sms(
            "13800000000".into(),
            code,
            "wx-code-2".into(),
            Some("Alice".into()),
            None,
        )
        .await
        .unwrap();

    assert_eq!(wechat_user.id, phone_user.id);
    assert_eq!(phone_user.phone.as_deref(), Some("13800000000"));
}
