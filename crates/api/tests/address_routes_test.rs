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
    AddressService, AdminService, CartService, CategoryService, ProductService, StoreService,
    UserService,
};
use axum_common::AppResult;
use axum_domain::address::entity::Address;
use axum_domain::address::repo::AddressRepository;
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
use chrono::Utc;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryAddressRepo {
    inner: Mutex<HashMap<Ulid, Address>>,
}

#[async_trait]
impl AddressRepository for InMemoryAddressRepo {
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Option<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .get(&address_id)
            .cloned()
            .filter(|item| item.user_id == user_id))
    }

    async fn create(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn update(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if guard
            .get(&address_id)
            .map(|item| item.user_id == user_id)
            .unwrap_or(false)
        {
            guard.remove(&address_id);
        }
        Ok(())
    }
}

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
struct InMemoryStoreRepo;

#[async_trait]
impl StoreRepository for InMemoryStoreRepo {
    async fn list(&self) -> AppResult<Vec<Store>> {
        Ok(vec![])
    }

    async fn create(&self, store: &Store) -> AppResult<Store> {
        Ok(store.clone())
    }
}

#[derive(Default)]
struct InMemoryCategoryRepo;

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepo {
    async fn list_by_store(&self, _store_id: Ulid) -> AppResult<Vec<Category>> {
        Ok(vec![])
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
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
struct InMemoryCartRepo;

#[async_trait]
impl CartRepository for InMemoryCartRepo {
    async fn get_cart(&self, _user_id: Ulid, _store_id: Ulid) -> AppResult<Option<Cart>> {
        Ok(None)
    }

    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        Ok(Cart::new(user_id, store_id))
    }

    async fn upsert_item(
        &self,
        _user_id: Ulid,
        _store_id: Ulid,
        _product_id: Ulid,
        _qty: i32,
        _price_snapshot: i32,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn remove_item(
        &self,
        _user_id: Ulid,
        _store_id: Ulid,
        _product_id: Ulid,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn clear(&self, _user_id: Ulid, _store_id: Ulid) -> AppResult<()> {
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

fn create_test_app() -> axum::Router {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let address_repo: Arc<dyn AddressRepository> = Arc::new(InMemoryAddressRepo::default());

    let state = AppState::new(
        UserService::new(user_repo),
        AdminService::new(admin_repo),
        StoreService::new(store_repo, Arc::new(FakeLbs::default())),
        CategoryService::new(category_repo),
        ProductService::new(product_repo),
        CartService::new(cart_repo),
        "secret".into(),
        3600,
        300,
    )
    .with_address_service(AddressService::new(address_repo));

    create_router(state)
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
async fn test_address_crud_flow() {
    let app = create_test_app();
    let user_id = Ulid::new();
    let auth = user_auth_header(user_id);

    let create_req = Request::builder()
        .method("POST")
        .uri("/api/v1/addresses")
        .header("content-type", "application/json")
        .header("authorization", auth.clone())
        .body(Body::from(
            json!({
                "name":"张三",
                "phone":"13800000000",
                "detail":"A区101",
                "lat":30.0,
                "lng":120.0,
                "is_default":false
            })
            .to_string(),
        ))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let create_body = to_bytes(create_res.into_body(), 1024 * 1024).await.unwrap();
    let create_value: Value = serde_json::from_slice(&create_body).unwrap();
    assert_eq!(create_value["success"], true);
    let address_id = create_value["data"]["address_id"]
        .as_str()
        .unwrap()
        .to_string();

    let list_req = Request::builder()
        .method("GET")
        .uri("/api/v1/addresses")
        .header("authorization", auth.clone())
        .body(Body::empty())
        .unwrap();
    let list_res = app.clone().oneshot(list_req).await.unwrap();
    let list_body = to_bytes(list_res.into_body(), 1024 * 1024).await.unwrap();
    let list_value: Value = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list_value["success"], true);
    assert_eq!(list_value["data"].as_array().unwrap().len(), 1);

    let set_default_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/addresses/{}/set_default", address_id))
        .header("authorization", auth)
        .body(Body::empty())
        .unwrap();
    let set_default_res = app.clone().oneshot(set_default_req).await.unwrap();
    let set_default_body = to_bytes(set_default_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let set_default_value: Value = serde_json::from_slice(&set_default_body).unwrap();
    assert_eq!(set_default_value["success"], true);
}
