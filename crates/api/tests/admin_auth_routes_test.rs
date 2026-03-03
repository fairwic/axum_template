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
use axum_domain::admin::entity::{Admin, AdminRole};
use axum_domain::admin::repo::AdminRepository;
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

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<String, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(openid).cloned())
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.openid.clone(), user.clone());
        Ok(user.clone())
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

#[tokio::test]
async fn test_admin_login_returns_token() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let user_service = UserService::new(user_repo);
    let admin_service = AdminService::new(admin_repo.clone());
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

    admin_service
        .create_admin(
            "13800000000".into(),
            "pass".into(),
            AdminRole::Platform,
            None,
        )
        .await
        .unwrap();

    let state = AppState::new(
        user_service,
        admin_service,
        store_service,
        category_service,
        product_service,
        cart_service,
        "secret".into(),
        3600,
        300,
    );
    let app = create_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/admin/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"phone":"13800000000","password":"pass"}"#))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert!(value["data"]["token"].as_str().unwrap_or("").len() > 0);
    assert_eq!(value["data"]["admin"]["phone"], "13800000000");
}
