use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use axum_api::auth::jwt::{encode_token, Claims};
use axum_api::{create_router, AppState};
use axum_application::{
    AdminService, CartService, CategoryService, ProductService, StoreService, UserService,
};
use axum_core_kernel::AppResult;
use axum_domain::admin::entity::Admin;
use axum_domain::admin::repo::AdminRepository;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use axum_domain::product::entity::Product;
use axum_domain::product::repo::ProductRepository;
use axum_domain::store::entity::{Store, StoreStatus};
use axum_domain::store::repo::StoreRepository;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use chrono::Utc;
use serde_json::Value;
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<Ulid, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().find(|item| item.openid == openid).cloned())
    }

    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&user_id).cloned())
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn set_current_store(&self, user_id: Ulid, store_id: Ulid) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        let user = guard.get_mut(&user_id).expect("user must exist");
        user.current_store_id = Some(store_id);
        Ok(user.clone())
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .find(|item| item.phone.as_deref() == Some(phone))
            .cloned())
    }

    async fn bind_phone(&self, user_id: Ulid, phone: String) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        let user = guard
            .get_mut(&user_id)
            .ok_or_else(|| axum_core_kernel::AppError::NotFound("user not found".into()))?;
        user.phone = Some(phone);
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

    async fn find_by_id(&self, store_id: Ulid) -> AppResult<Option<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&store_id.to_string()).cloned())
    }

    async fn update(&self, store: &Store) -> AppResult<Store> {
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

    async fn find_by_id(&self, category_id: ulid::Ulid) -> AppResult<Option<Category>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&category_id.to_string()).cloned())
    }

    async fn update(&self, category: &Category) -> AppResult<Category> {
        let mut guard = self.inner.lock().await;
        guard.insert(category.id.to_string(), category.clone());
        Ok(category.clone())
    }
}

#[derive(Default)]
struct InMemoryCartRepo {
    carts: Mutex<HashMap<(Ulid, Ulid), Cart>>,
}

#[async_trait]
impl CartRepository for InMemoryCartRepo {
    async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<Cart>> {
        let guard = self.carts.lock().await;
        Ok(guard.get(&(user_id, store_id)).cloned())
    }

    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        let mut guard = self.carts.lock().await;
        let cart = Cart::new(user_id, store_id);
        guard.insert((user_id, store_id), cart.clone());
        Ok(cart)
    }

    async fn upsert_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        let cart = guard
            .entry((user_id, store_id))
            .or_insert_with(|| Cart::new(user_id, store_id));
        cart.upsert_item(product_id, qty, price_snapshot);
        Ok(())
    }

    async fn remove_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.remove_item(product_id);
        }
        Ok(())
    }

    async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.items.clear();
        }
        Ok(())
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

    async fn update(&self, product: &Product) -> AppResult<Product> {
        Ok(product.clone())
    }

    async fn find_by_id(
        &self,
        _store_id: ulid::Ulid,
        _product_id: ulid::Ulid,
    ) -> AppResult<Option<Product>> {
        Ok(None)
    }

    async fn find_by_ids(
        &self,
        _store_id: ulid::Ulid,
        _product_ids: &[ulid::Ulid],
    ) -> AppResult<Vec<Product>> {
        Ok(vec![])
    }

    async fn try_lock_stock(&self, _product_id: ulid::Ulid, _qty: i32) -> AppResult<bool> {
        Ok(false)
    }

    async fn release_stock(&self, _product_id: ulid::Ulid, _qty: i32) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeLbs;

#[async_trait]
impl axum_application::services::store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> AppResult<f64> {
        Ok(1.2)
    }
}

fn user_auth_header(user_id: Ulid) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        role: "USER".into(),
        exp: (Utc::now().timestamp() + 3600) as usize,
    };
    let token = encode_token(&claims, "secret").unwrap();
    format!("Bearer {token}")
}

#[tokio::test]
async fn test_stores_nearby() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());

    let user_service = UserService::new(user_repo.clone());
    let admin_service = AdminService::new(admin_repo);
    let lbs: Arc<dyn axum_application::services::store_service::LbsService> =
        Arc::new(FakeLbs::default());
    let store_service = StoreService::new(store_repo.clone(), lbs);
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let category_service = CategoryService::new(category_repo);
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let product_service = ProductService::new(product_repo);
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let cart_service = CartService::new(cart_repo);

    let user = user_repo
        .create(&User::new("openid-store-test".into(), None, None).unwrap())
        .await
        .unwrap();

    let store = Store::new(
        "Store A".into(),
        "Addr".into(),
        30.0,
        120.0,
        "13800000000".into(),
        "9-21".into(),
        StoreStatus::Open,
        3.0,
        0,
        2,
        2,
    )
    .unwrap();
    store_repo.create(&store).await.unwrap();

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
    let auth = user_auth_header(user.id);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/stores/nearby?lat=30.0&lng=120.0")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert_eq!(value["data"][0]["name"], "Store A");

    let select_req = Request::builder()
        .method("POST")
        .uri("/api/v1/stores/select")
        .header("content-type", "application/json")
        .header("authorization", auth.clone())
        .body(Body::from(
            serde_json::json!({
                "store_id": store.id.to_string()
            })
            .to_string(),
        ))
        .unwrap();
    let select_res = app.clone().oneshot(select_req).await.unwrap();
    let select_body = to_bytes(select_res.into_body(), 1024 * 1024).await.unwrap();
    let select_value: Value = serde_json::from_slice(&select_body).unwrap();
    assert_eq!(select_value["success"], true);
    assert_eq!(select_value["data"]["id"], store.id.to_string());

    let current_req = Request::builder()
        .method("GET")
        .uri("/api/v1/stores/current")
        .header("authorization", auth)
        .body(Body::empty())
        .unwrap();
    let current_res = app.oneshot(current_req).await.unwrap();
    let current_body = to_bytes(current_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let current_value: Value = serde_json::from_slice(&current_body).unwrap();
    assert_eq!(current_value["success"], true);
    assert_eq!(current_value["data"]["id"], store.id.to_string());
}
