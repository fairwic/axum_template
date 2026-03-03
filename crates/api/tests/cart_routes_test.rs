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
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use ulid::Ulid;

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
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
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
        _store_id: Ulid,
        _category_id: Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        Ok((vec![], 0))
    }

    async fn search(
        &self,
        _store_id: Ulid,
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
struct FakeLbs;

#[async_trait]
impl axum_application::services::store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> AppResult<f64> {
        Ok(0.0)
    }
}

#[tokio::test]
async fn test_cart_flow() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());

    let user_service = UserService::new(user_repo);
    let admin_service = AdminService::new(admin_repo);
    let lbs: Arc<dyn axum_application::services::store_service::LbsService> =
        Arc::new(FakeLbs::default());
    let store_service = StoreService::new(store_repo, lbs);
    let category_service = CategoryService::new(category_repo);
    let product_service = ProductService::new(product_repo);
    let cart_service = CartService::new(cart_repo);

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

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product_id = Ulid::new();

    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/cart?store_id={}", store_id))
        .header("x-user-id", user_id.to_string())
        .body(Body::empty())
        .unwrap();
    let get_res = app.clone().oneshot(get_req).await.unwrap();
    let get_body = to_bytes(get_res.into_body(), 1024 * 1024).await.unwrap();
    let get_value: Value = serde_json::from_slice(&get_body).unwrap();
    assert_eq!(get_value["success"], true);
    assert_eq!(get_value["data"]["items"].as_array().unwrap().len(), 0);

    let add_payload = json!({
        "store_id": store_id.to_string(),
        "product_id": product_id.to_string(),
        "qty": 1,
        "price_snapshot": 990
    });
    let add_req = Request::builder()
        .method("POST")
        .uri("/api/v1/cart/add")
        .header("content-type", "application/json")
        .header("x-user-id", user_id.to_string())
        .body(Body::from(add_payload.to_string()))
        .unwrap();
    let add_res = app.clone().oneshot(add_req).await.unwrap();
    let add_body = to_bytes(add_res.into_body(), 1024 * 1024).await.unwrap();
    let add_value: Value = serde_json::from_slice(&add_body).unwrap();
    assert_eq!(add_value["success"], true);
    assert_eq!(add_value["data"]["items"][0]["qty"], 1);

    let update_payload = json!({
        "store_id": store_id.to_string(),
        "product_id": product_id.to_string(),
        "qty": 2
    });
    let update_req = Request::builder()
        .method("POST")
        .uri("/api/v1/cart/update_qty")
        .header("content-type", "application/json")
        .header("x-user-id", user_id.to_string())
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    let update_res = app.clone().oneshot(update_req).await.unwrap();
    let update_body = to_bytes(update_res.into_body(), 1024 * 1024).await.unwrap();
    let update_value: Value = serde_json::from_slice(&update_body).unwrap();
    assert_eq!(update_value["success"], true);
    assert_eq!(update_value["data"]["items"][0]["qty"], 2);

    let remove_payload = json!({
        "store_id": store_id.to_string(),
        "product_id": product_id.to_string()
    });
    let remove_req = Request::builder()
        .method("POST")
        .uri("/api/v1/cart/remove")
        .header("content-type", "application/json")
        .header("x-user-id", user_id.to_string())
        .body(Body::from(remove_payload.to_string()))
        .unwrap();
    let remove_res = app.clone().oneshot(remove_req).await.unwrap();
    let remove_body = to_bytes(remove_res.into_body(), 1024 * 1024).await.unwrap();
    let remove_value: Value = serde_json::from_slice(&remove_body).unwrap();
    assert_eq!(remove_value["success"], true);
    assert_eq!(remove_value["data"]["items"].as_array().unwrap().len(), 0);

    let add_req = Request::builder()
        .method("POST")
        .uri("/api/v1/cart/add")
        .header("content-type", "application/json")
        .header("x-user-id", user_id.to_string())
        .body(Body::from(add_payload.to_string()))
        .unwrap();
    let _ = app.clone().oneshot(add_req).await.unwrap();

    let clear_payload = json!({"store_id": store_id.to_string()});
    let clear_req = Request::builder()
        .method("POST")
        .uri("/api/v1/cart/clear")
        .header("content-type", "application/json")
        .header("x-user-id", user_id.to_string())
        .body(Body::from(clear_payload.to_string()))
        .unwrap();
    let clear_res = app.oneshot(clear_req).await.unwrap();
    let clear_body = to_bytes(clear_res.into_body(), 1024 * 1024).await.unwrap();
    let clear_value: Value = serde_json::from_slice(&clear_body).unwrap();
    assert_eq!(clear_value["success"], true);
    assert_eq!(clear_value["data"]["items"].as_array().unwrap().len(), 0);
}
