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
use axum_domain::product::entity::{Product, ProductStatus};
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
struct InMemoryProductRepo {
    inner: Mutex<HashMap<Ulid, Product>>,
}

#[async_trait]
impl ProductRepository for InMemoryProductRepo {
    async fn list_by_category(
        &self,
        store_id: Ulid,
        category_id: Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|p| p.store_id == store_id && p.category_id == category_id)
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|p| p.store_id == store_id && p.title.contains(keyword))
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
async fn test_products_list_and_search() {
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
    let product_service = ProductService::new(product_repo.clone());
    let cart_service = CartService::new(cart_repo);

    let store_id = Ulid::new();
    let category_id = Ulid::new();
    let product = Product::new(
        store_id,
        category_id,
        "椰子水".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        10,
        ProductStatus::On,
        vec!["new".into()],
    )
    .unwrap();
    product_repo.create(&product).await.unwrap();

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

    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/products?store_id={}&category_id={}&page=1&page_size=20",
            store_id, category_id
        ))
        .body(Body::empty())
        .unwrap();

    let list_res = app.clone().oneshot(list_req).await.unwrap();
    let list_body = to_bytes(list_res.into_body(), 1024 * 1024).await.unwrap();
    let list_value: Value = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list_value["success"], true);

    let search_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/products/search?store_id={}&keyword=椰子&page=1&page_size=20",
            store_id
        ))
        .body(Body::empty())
        .unwrap();

    let search_res = app.oneshot(search_req).await.unwrap();
    let search_body = to_bytes(search_res.into_body(), 1024 * 1024).await.unwrap();
    let search_value: Value = serde_json::from_slice(&search_body).unwrap();
    assert_eq!(search_value["success"], true);
    assert_eq!(search_value["data"]["items"][0]["title"], "椰子水");
}
