use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
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
use serde_json::Value;
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

    async fn find_by_id(&self, category_id: Ulid) -> AppResult<Option<Category>> {
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
struct InMemoryProductRepo {
    inner: Mutex<HashMap<Ulid, Product>>,
}

#[async_trait]
impl ProductRepository for InMemoryProductRepo {
    async fn list_by_category(
        &self,
        _store_id: Ulid,
        _category_id: Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|item| item.store_id == _store_id && item.category_id == _category_id)
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn search(
        &self,
        _store_id: Ulid,
        _keyword: &str,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|item| item.store_id == _store_id && item.title.contains(_keyword))
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn create(&self, product: &Product) -> AppResult<Product> {
        let mut guard = self.inner.lock().await;
        guard.insert(product.id, product.clone());
        Ok(product.clone())
    }

    async fn find_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Option<Product>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .get(&product_id)
            .filter(|item| item.store_id == store_id)
            .cloned())
    }

    async fn update(&self, product: &Product) -> AppResult<Product> {
        let mut guard = self.inner.lock().await;
        guard.insert(product.id, product.clone());
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
async fn test_admin_catalog_manage_flow() {
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

    let create_store_req = Request::builder()
        .method("POST")
        .uri("/api/admin/v1/stores")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "name": "北区店",
                "address": "北区 1 号",
                "lat": 30.0,
                "lng": 120.0,
                "phone": "13800000000",
                "business_hours": "08:00-22:00",
                "status": "OPEN",
                "delivery_radius_km": 3.0,
                "delivery_fee_base": 0,
                "delivery_fee_per_km": 100,
                "runner_service_fee": 200
            })
            .to_string(),
        ))
        .unwrap();

    let create_store_res = app.clone().oneshot(create_store_req).await.unwrap();
    assert_eq!(create_store_res.status(), StatusCode::OK);
    let create_store_body = to_bytes(create_store_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_store_value: Value = serde_json::from_slice(&create_store_body).unwrap();

    assert_eq!(create_store_value["success"], true);
    let store_id = create_store_value["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(create_store_value["data"]["name"], "北区店");

    let update_store_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/admin/v1/stores/{store_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "name": "北区店-更新",
                "address": "北区 2 号",
                "lat": 31.0,
                "lng": 121.0,
                "phone": "13900000000",
                "business_hours": "09:00-23:00",
                "status": "OPEN",
                "delivery_radius_km": 4.0,
                "delivery_fee_base": 100,
                "delivery_fee_per_km": 120,
                "runner_service_fee": 300
            })
            .to_string(),
        ))
        .unwrap();
    let update_store_res = app.clone().oneshot(update_store_req).await.unwrap();
    assert_eq!(update_store_res.status(), StatusCode::OK);
    let update_store_body = to_bytes(update_store_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let update_store_value: Value = serde_json::from_slice(&update_store_body).unwrap();
    assert_eq!(update_store_value["success"], true);
    assert_eq!(update_store_value["data"]["name"], "北区店-更新");
    assert_eq!(update_store_value["data"]["delivery_radius_km"], 4.0);

    let list_store_req = Request::builder()
        .method("GET")
        .uri("/api/admin/v1/stores")
        .body(Body::empty())
        .unwrap();
    let list_store_res = app.clone().oneshot(list_store_req).await.unwrap();
    assert_eq!(list_store_res.status(), StatusCode::OK);
    let list_store_body = to_bytes(list_store_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let list_store_value: Value = serde_json::from_slice(&list_store_body).unwrap();
    assert_eq!(list_store_value["success"], true);
    assert_eq!(list_store_value["data"][0]["id"], store_id);

    let create_category_req = Request::builder()
        .method("POST")
        .uri("/api/admin/v1/categories")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "store_id": store_id,
                "name": "饮料",
                "sort_order": 1,
                "status": "ON"
            })
            .to_string(),
        ))
        .unwrap();
    let create_category_res = app.clone().oneshot(create_category_req).await.unwrap();
    assert_eq!(create_category_res.status(), StatusCode::OK);
    let create_category_body = to_bytes(create_category_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_category_value: Value = serde_json::from_slice(&create_category_body).unwrap();
    assert_eq!(create_category_value["success"], true);
    let category_id = create_category_value["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_category_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/admin/v1/categories/{category_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "name": "饮料-更新",
                "sort_order": 2,
                "status": "OFF"
            })
            .to_string(),
        ))
        .unwrap();
    let update_category_res = app.clone().oneshot(update_category_req).await.unwrap();
    assert_eq!(update_category_res.status(), StatusCode::OK);
    let update_category_body = to_bytes(update_category_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let update_category_value: Value = serde_json::from_slice(&update_category_body).unwrap();
    assert_eq!(update_category_value["success"], true);
    assert_eq!(update_category_value["data"]["name"], "饮料-更新");
    assert_eq!(update_category_value["data"]["status"], "OFF");

    let create_product_req = Request::builder()
        .method("POST")
        .uri("/api/admin/v1/products")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "store_id": store_id,
                "category_id": category_id,
                "title": "椰子水",
                "subtitle": "冷饮",
                "cover_image": "cover.png",
                "images": ["a.png", "b.png"],
                "price": 990,
                "original_price": 1190,
                "stock": 10,
                "status": "ON",
                "tags": ["new", "hot"]
            })
            .to_string(),
        ))
        .unwrap();
    let create_product_res = app.clone().oneshot(create_product_req).await.unwrap();
    assert_eq!(create_product_res.status(), StatusCode::OK);
    let create_product_body = to_bytes(create_product_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_product_value: Value = serde_json::from_slice(&create_product_body).unwrap();
    assert_eq!(create_product_value["success"], true);
    let product_id = create_product_value["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_product_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/admin/v1/products/{product_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "store_id": store_id,
                "category_id": category_id,
                "title": "椰子水-更新",
                "subtitle": "冰镇",
                "cover_image": "cover-new.png",
                "images": ["c.png"],
                "price": 1090,
                "original_price": 1290,
                "stock": 20,
                "status": "OFF",
                "tags": ["discount"]
            })
            .to_string(),
        ))
        .unwrap();
    let update_product_res = app.clone().oneshot(update_product_req).await.unwrap();
    assert_eq!(update_product_res.status(), StatusCode::OK);
    let update_product_body = to_bytes(update_product_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let update_product_value: Value = serde_json::from_slice(&update_product_body).unwrap();
    assert_eq!(update_product_value["success"], true);
    assert_eq!(update_product_value["data"]["title"], "椰子水-更新");
    assert_eq!(update_product_value["data"]["stock"], 20);
    assert_eq!(update_product_value["data"]["status"], "OFF");
}
