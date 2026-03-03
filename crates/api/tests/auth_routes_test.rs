use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use axum_api::{create_router, AppState};
use axum_application::{
    AdminService, CartService, CategoryService, ProductService, StoreService, UserService,
};
use axum_common::AppResult;
use axum_domain::admin::entity::Admin;
use axum_domain::admin::repo::AdminRepository;
use axum_domain::auth::{SmsGateway, WechatAuthClient, WechatSession};
use axum_domain::cache::CacheService;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use axum_domain::product::entity::Product;
use axum_domain::product::repo::ProductRepository;
use axum_domain::store::entity::Store;
use axum_domain::store::repo::StoreRepository;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use serde_json::Value;
use tokio::sync::Mutex;
use tower::util::ServiceExt;
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
}

#[derive(Default)]
struct InMemoryAdminRepo {
    inner: Mutex<HashMap<String, Admin>>,
}

#[async_trait]
impl AdminRepository for InMemoryAdminRepo {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(phone).cloned())
    }

    async fn create(&self, admin: &Admin) -> AppResult<Admin> {
        let mut guard = self.inner.lock().await;
        guard.insert(admin.phone.clone(), admin.clone());
        Ok(admin.clone())
    }
}

#[derive(Default)]
struct InMemoryStoreRepo {
    inner: Mutex<HashMap<String, Store>>,
}

#[async_trait]
impl StoreRepository for InMemoryStoreRepo {
    async fn list(&self) -> AppResult<Vec<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().cloned().collect())
    }

    async fn create(&self, store: &Store) -> AppResult<Store> {
        let mut guard = self.inner.lock().await;
        guard.insert(store.id.to_string(), store.clone());
        Ok(store.clone())
    }
}

#[derive(Default)]
struct InMemoryCategoryRepo {
    inner: Mutex<HashMap<String, Category>>,
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepo {
    async fn list_by_store(&self, store_id: ulid::Ulid) -> AppResult<Vec<Category>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.store_id == store_id)
            .cloned()
            .collect())
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
        let mut guard = self.inner.lock().await;
        guard.insert(category.id.to_string(), category.clone());
        Ok(category.clone())
    }
}

#[derive(Default)]
struct InMemoryProductRepo;

#[async_trait]
impl ProductRepository for InMemoryProductRepo {
    async fn list_by_category(
        &self,
        _store_id: ulid::Ulid,
        _category_id: ulid::Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        Ok((vec![], 0))
    }

    async fn search(
        &self,
        _store_id: ulid::Ulid,
        _keyword: &str,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        Ok((vec![], 0))
    }

    async fn create(&self, product: &Product) -> AppResult<Product> {
        Ok(product.clone())
    }
}

#[derive(Default)]
struct InMemoryCartRepo;

#[async_trait]
impl CartRepository for InMemoryCartRepo {
    async fn get_cart(
        &self,
        _user_id: ulid::Ulid,
        _store_id: ulid::Ulid,
    ) -> AppResult<Option<Cart>> {
        Ok(None)
    }

    async fn create_cart(&self, user_id: ulid::Ulid, store_id: ulid::Ulid) -> AppResult<Cart> {
        Ok(Cart::new(user_id, store_id))
    }

    async fn upsert_item(
        &self,
        _user_id: ulid::Ulid,
        _store_id: ulid::Ulid,
        _product_id: ulid::Ulid,
        _qty: i32,
        _price_snapshot: i32,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn remove_item(
        &self,
        _user_id: ulid::Ulid,
        _store_id: ulid::Ulid,
        _product_id: ulid::Ulid,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn clear(&self, _user_id: ulid::Ulid, _store_id: ulid::Ulid) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeLbs;

#[async_trait]
impl axum_application::services::store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> AppResult<f64> {
        Ok(0.0)
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

struct TestContext {
    app: axum::Router,
    sms_gateway: Arc<FakeSmsGateway>,
}

async fn create_test_app() -> TestContext {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let wechat_client = Arc::new(FakeWechatAuthClient::default());
    let cache: Arc<dyn CacheService> = Arc::new(InMemoryCache::default());
    let sms_gateway = Arc::new(FakeSmsGateway::default());

    wechat_client
        .code_to_openid
        .lock()
        .await
        .insert("wx-code-1".into(), "openid-1".into());

    let service = UserService::new(repo).with_auth(wechat_client, cache, sms_gateway.clone(), 300);
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let admin_service = AdminService::new(admin_repo);
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let lbs: Arc<dyn axum_application::services::store_service::LbsService> =
        Arc::new(FakeLbs::default());
    let store_service = StoreService::new(store_repo, lbs);
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let category_service = CategoryService::new(category_repo);
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let product_service = ProductService::new(product_repo);
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let cart_service = CartService::new(cart_repo);
    let state = AppState::new(
        service,
        admin_service,
        store_service,
        category_service,
        product_service,
        cart_service,
        "secret".into(),
        3600,
        300,
    );
    TestContext {
        app: create_router(state),
        sms_gateway,
    }
}

#[tokio::test]
async fn test_wechat_login_returns_token() {
    let ctx = create_test_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/wechat_login")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"wx-code-1","nickname":"Alice","avatar":null}"#,
        ))
        .unwrap();

    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert!(value["data"]["token"].as_str().unwrap_or("").len() > 0);
    assert_eq!(value["data"]["user"]["openid"], "openid-1");
}

#[tokio::test]
async fn test_phone_sms_login_links_with_same_wechat_account() {
    let ctx = create_test_app().await;

    let wechat_login_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/wechat_login")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"wx-code-1","nickname":"Alice","avatar":null}"#,
        ))
        .unwrap();

    let wechat_res = ctx.app.clone().oneshot(wechat_login_req).await.unwrap();
    let wechat_body = to_bytes(wechat_res.into_body(), 1024 * 1024).await.unwrap();
    let wechat_value: Value = serde_json::from_slice(&wechat_body).unwrap();
    let wechat_user_id = wechat_value["data"]["user"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let send_code_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/sms/send_code")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"phone":"13800000000"}"#))
        .unwrap();
    let send_code_res = ctx.app.clone().oneshot(send_code_req).await.unwrap();
    let send_code_body = to_bytes(send_code_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let send_code_value: Value = serde_json::from_slice(&send_code_body).unwrap();
    assert_eq!(send_code_value["success"], true);

    let sms_code = ctx
        .sms_gateway
        .last_code_by_phone
        .lock()
        .await
        .get("13800000000")
        .cloned()
        .unwrap();

    let phone_login_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/phone_sms_login")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"phone":"13800000000","sms_code":"{}","wechat_code":"wx-code-1","nickname":"Alice","avatar":null}}"#,
            sms_code
        )))
        .unwrap();
    let phone_login_res = ctx.app.clone().oneshot(phone_login_req).await.unwrap();
    let phone_login_body = to_bytes(phone_login_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let phone_login_value: Value = serde_json::from_slice(&phone_login_body).unwrap();

    assert_eq!(phone_login_value["success"], true);
    assert_eq!(phone_login_value["data"]["user"]["id"], wechat_user_id);
    assert_eq!(
        phone_login_value["data"]["user"]["phone"].as_str(),
        Some("13800000000")
    );
}
