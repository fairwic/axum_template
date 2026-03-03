# Mini Store Backend V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Sprint 1 backend for 门店/类目/商品/搜索/购物车 + 管理后台最小 API，含微信登录、管理员登录与腾讯位置服务距离计算。

**Architecture:** Keep DDD layering (`domain`/`application`/`infrastructure`/`api`). Add new modules for user(admin/wechcat), store, category, product, cart. API split between `/api/v1` (C端) and `/api/admin/v1` (管理端).

**Tech Stack:** Rust, Axum, SQLx(Postgres), Redis(缓存占位), JWT, bcrypt, reqwest.

**Skills:** @superpowers:test-driven-development, @supabase-postgres-best-practices

---

### Task 1: Add Auth Config + JWT Utilities

**Files:**
- Modify: `crates/infrastructure/src/config.rs`
- Modify: `config/default.toml`
- Modify: `crates/api/src/lib.rs`
- Create: `crates/api/src/auth/mod.rs`
- Create: `crates/api/src/auth/jwt.rs`
- Test: `crates/api/tests/auth_jwt_test.rs`

**Step 1: Write the failing test**
```rust
use axum_api::auth::jwt::{encode_token, decode_token, Claims};

#[test]
fn test_jwt_roundtrip() {
    let claims = Claims { sub: "user_1".into(), role: "USER".into(), exp: 2000000000 };
    let token = encode_token(&claims, "secret").unwrap();
    let decoded = decode_token(&token, "secret").unwrap();
    assert_eq!(decoded.sub, "user_1");
    assert_eq!(decoded.role, "USER");
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test -p axum-api -- tests/auth_jwt_test.rs -v`
Expected: FAIL (module not found)

**Step 3: Write minimal implementation**
```rust
// crates/api/src/auth/jwt.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn encode_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
    Ok(data.claims)
}
```

```rust
// crates/infrastructure/src/config.rs (add fields)
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_ttl_secs")]
    pub jwt_ttl_secs: u64,
}

fn default_jwt_ttl_secs() -> u64 { 7 * 24 * 3600 }
```

```toml
# config/default.toml
[auth]
jwt_secret = "dev-secret"
jwt_ttl_secs = 604800
```

**Step 4: Run test to verify it passes**
Run: `cargo test -p axum-api -- tests/auth_jwt_test.rs -v`
Expected: PASS

**Step 5: Commit**
```bash
git add crates/api/src/auth crates/api/tests/auth_jwt_test.rs crates/infrastructure/src/config.rs config/default.toml
git commit -m "feat(认证): 添加JWT基础配置与工具"
```

---

### Task 2: WeChat User Domain + Repo + Service + API

**Files:**
- Modify: `crates/domain/src/user/entity.rs`
- Modify: `crates/domain/src/user/repo.rs`
- Modify: `crates/domain/src/user/mod.rs`
- Modify: `crates/infrastructure/src/models/user_model.rs`
- Modify: `crates/infrastructure/src/postgres/user_repo.rs`
- Modify: `crates/application/src/dtos/user_dto.rs`
- Modify: `crates/application/src/services/user_service.rs`
- Create: `crates/api/src/handlers/auth_handler.rs`
- Create: `crates/api/src/routes/auth.rs`
- Create: `crates/api/src/handlers/member_handler.rs`
- Create: `crates/api/src/routes/member.rs`
- Test: `crates/application/tests/user_service_test.rs`
- Test: `crates/api/tests/auth_routes_test.rs`

**Step 1: Write failing test (service)**
```rust
#[tokio::test]
async fn test_login_creates_user() {
    let repo = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);
    let user = service.login_with_openid("openid-1".into(), None, None).await.unwrap();
    assert_eq!(user.openid, "openid-1");
    assert!(user.is_member);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test -p axum-application -- tests/user_service_test.rs -v`
Expected: FAIL (method not found)

**Step 3: Implement minimal code**
```rust
// domain user entity
pub struct User {
    pub id: Ulid,
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub is_member: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(openid: String, nickname: Option<String>, avatar: Option<String>) -> Result<Self, DomainError> {
        if openid.trim().is_empty() {
            return Err(DomainError::Validation("openid is required".into()));
        }
        let now = Utc::now();
        Ok(Self { id: Ulid::new(), openid, nickname, avatar, phone: None, is_member: true, created_at: now, updated_at: now })
    }
}
```

```rust
// user repo trait
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn create(&self, user: &User) -> AppResult<User>;
}
```

```rust
// application service
pub async fn login_with_openid(&self, openid: String, nickname: Option<String>, avatar: Option<String>) -> AppResult<User> {
    if let Some(user) = self.repo.find_by_openid(&openid).await? {
        return Ok(user);
    }
    let user = User::new(openid, nickname, avatar)?;
    self.repo.create(&user).await
}
```

**Step 4: Run test to verify it passes**
Run: `cargo test -p axum-application -- tests/user_service_test.rs -v`
Expected: PASS

**Step 5: Commit**
```bash
git add crates/domain/src/user crates/application/src/services/user_service.rs crates/application/src/dtos/user_dto.rs
git commit -m "refactor(用户): 实现微信用户登录服务"
```

---

### Task 3: Admin Domain + Auth Service + API

**Files:**
- Create: `crates/domain/src/admin/*`
- Create: `crates/infrastructure/src/models/admin_model.rs`
- Create: `crates/infrastructure/src/postgres/admin_repo.rs`
- Create: `crates/application/src/services/admin_service.rs`
- Create: `crates/api/src/handlers/admin_auth_handler.rs`
- Create: `crates/api/src/routes/admin_auth.rs`
- Test: `crates/application/tests/admin_service_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_admin_login_success() {
    let repo = Arc::new(InMemoryAdminRepo::default());
    let service = AdminService::new(repo);
    service.create_admin("13800000000".into(), "pass".into(), AdminRole::Platform, None).await.unwrap();
    let admin = service.login("13800000000", "pass").await.unwrap();
    assert_eq!(admin.phone, "13800000000");
}
```

**Step 2: Implement minimal code**
```rust
// admin service
pub async fn login(&self, phone: &str, password: &str) -> AppResult<Admin> {
    let admin = self.repo.find_by_phone(phone).await?
        .ok_or_else(|| AppError::NotFound("admin not found".into()))?;
    if !bcrypt::verify(password, &admin.password_hash).unwrap_or(false) {
        return Err(AppError::Validation("password incorrect".into()));
    }
    Ok(admin)
}
```

**Step 3: Run test**
Run: `cargo test -p axum-application -- tests/admin_service_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/domain/src/admin crates/application/src/services/admin_service.rs crates/api/src/handlers/admin_auth_handler.rs
git commit -m "feat(管理员): 添加登录与服务"
```

---

### Task 4: Store Module (Domain + Repo + Service + API)

**Files:**
- Create: `crates/domain/src/store/*`
- Create: `crates/infrastructure/src/models/store_model.rs`
- Create: `crates/infrastructure/src/postgres/store_repo.rs`
- Create: `crates/application/src/services/store_service.rs`
- Create: `crates/api/src/handlers/store_handler.rs`
- Create: `crates/api/src/routes/store.rs`
- Test: `crates/application/tests/store_service_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_list_stores_sorted_by_distance() {
    let repo = Arc::new(InMemoryStoreRepo::default());
    let lbs = Arc::new(FakeLbs::default());
    let service = StoreService::new(repo, lbs);
    let stores = service.nearby(30.0, 120.0).await.unwrap();
    assert!(stores.len() >= 0);
}
```

**Step 2: Implement minimal entity + service**
```rust
pub struct Store {
    pub id: Ulid,
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: StoreStatus,
    pub delivery_radius_km: f64,
    pub delivery_fee_base: i32,
    pub delivery_fee_per_km: i32,
    pub runner_service_fee: i32,
}
```

**Step 3: Run test**
Run: `cargo test -p axum-application -- tests/store_service_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/domain/src/store crates/application/src/services/store_service.rs crates/api/src/handlers/store_handler.rs
git commit -m "feat(门店): 添加门店模块与附近门店接口"
```

---

### Task 5: Category Module (Domain + Repo + Service + API)

**Files:**
- Create: `crates/domain/src/category/*`
- Create: `crates/infrastructure/src/models/category_model.rs`
- Create: `crates/infrastructure/src/postgres/category_repo.rs`
- Create: `crates/application/src/services/category_service.rs`
- Create: `crates/api/src/handlers/category_handler.rs`
- Create: `crates/api/src/routes/category.rs`
- Test: `crates/application/tests/category_service_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_list_categories_by_store() {
    let repo = Arc::new(InMemoryCategoryRepo::default());
    let service = CategoryService::new(repo);
    let list = service.list_by_store("store1".into()).await.unwrap();
    assert_eq!(list.len(), 0);
}
```

**Step 2: Implement minimal code**
```rust
// domain/category/entity.rs
pub struct Category {
    pub id: Ulid,
    pub store_id: Ulid,
    pub name: String,
    pub sort_order: i32,
    pub status: CategoryStatus,
}

impl Category {
    pub fn new(store_id: Ulid, name: String, sort_order: i32) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("name is required".into()));
        }
        Ok(Self { id: Ulid::new(), store_id, name, sort_order, status: CategoryStatus::On })
    }
}
```

```rust
// domain/category/repo.rs
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>>;
    async fn create(&self, category: &Category) -> AppResult<Category>;
}
```

```rust
// application/category_service.rs
pub async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
    self.repo.list_by_store(store_id).await
}
```

**Step 3: Run test**
Run: `cargo test -p axum-application -- tests/category_service_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/domain/src/category crates/application/src/services/category_service.rs crates/api/src/handlers/category_handler.rs
git commit -m "feat(类目): 添加类目模块"
```

---

### Task 6: Product Module + Search

**Files:**
- Create: `crates/domain/src/product/*`
- Create: `crates/infrastructure/src/models/product_model.rs`
- Create: `crates/infrastructure/src/postgres/product_repo.rs`
- Create: `crates/application/src/services/product_service.rs`
- Create: `crates/api/src/handlers/product_handler.rs`
- Create: `crates/api/src/routes/product.rs`
- Test: `crates/application/tests/product_service_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_search_products_by_keyword() {
    let repo = Arc::new(InMemoryProductRepo::default());
    let service = ProductService::new(repo);
    let list = service.search("store1".into(), "水".into(), 1, 20).await.unwrap();
    assert_eq!(list.items.len(), 0);
}
```

**Step 2: Implement minimal code**
```rust
// domain/product/entity.rs
pub struct Product {
    pub id: Ulid,
    pub store_id: Ulid,
    pub category_id: Ulid,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Vec<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: ProductStatus,
    pub tags: Vec<String>,
}
```

```rust
// application/product_service.rs
pub async fn search(&self, store_id: Ulid, keyword: String, page: i64, page_size: i64) -> AppResult<PagedResponse<Product>> {
    let (items, total) = self.repo.search(store_id, &keyword, page, page_size).await?;
    Ok(PagedResponse::new(items, total, page, page_size))
}
```

**Step 3: Run test**
Run: `cargo test -p axum-application -- tests/product_service_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/domain/src/product crates/application/src/services/product_service.rs crates/api/src/handlers/product_handler.rs
git commit -m "feat(商品): 添加商品模块与搜索"
```

---

### Task 7: Cart Module

**Files:**
- Create: `crates/domain/src/cart/*`
- Create: `crates/infrastructure/src/models/cart_model.rs`
- Create: `crates/infrastructure/src/postgres/cart_repo.rs`
- Create: `crates/application/src/services/cart_service.rs`
- Create: `crates/api/src/handlers/cart_handler.rs`
- Create: `crates/api/src/routes/cart.rs`
- Test: `crates/application/tests/cart_service_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_cart_add_item() {
    let repo = Arc::new(InMemoryCartRepo::default());
    let service = CartService::new(repo);
    service.add_item("user1".into(), "store1".into(), "prod1".into(), 1, 990).await.unwrap();
    let cart = service.get_cart("user1".into(), "store1".into()).await.unwrap();
    assert_eq!(cart.items.len(), 1);
}
```

**Step 2: Implement minimal code**
```rust
// domain/cart/entity.rs
pub struct CartItem {
    pub product_id: Ulid,
    pub qty: i32,
    pub price_snapshot: i32,
}

pub struct Cart {
    pub id: Ulid,
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub items: Vec<CartItem>,
}
```

```rust
// application/cart_service.rs
pub async fn add_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid, qty: i32, price_snapshot: i32) -> AppResult<()> {
    let cart = self.repo.find_or_create(user_id, store_id).await?;
    self.repo.add_item(cart.id, product_id, qty, price_snapshot).await?;
    Ok(())
}
```

**Step 3: Run test**
Run: `cargo test -p axum-application -- tests/cart_service_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/domain/src/cart crates/application/src/services/cart_service.rs crates/api/src/handlers/cart_handler.rs
git commit -m "feat(购物车): 添加购物车模块"
```

---

### Task 8: Tencent LBS Client

**Files:**
- Create: `crates/infrastructure/src/lbs/tencent.rs`
- Modify: `crates/infrastructure/src/lib.rs`
- Modify: `crates/infrastructure/src/config.rs`
- Test: `crates/infrastructure/tests/tencent_lbs_test.rs`

**Step 1: Write failing test**
```rust
#[tokio::test]
async fn test_distance_km() {
    let client = TencentLbs::new("key".into());
    let km = client.distance_km((30.0,120.0),(30.1,120.1)).await.unwrap();
    assert!(km > 0.0);
}
```

**Step 2: Implement minimal code**
```rust
pub struct TencentLbs { key: String }
impl TencentLbs {
    pub fn new(key: String) -> Self { Self { key } }
    pub async fn distance_km(&self, from: (f64,f64), to: (f64,f64)) -> AppResult<f64> {
        // call https://apis.map.qq.com/ws/distance/v1/ (driving=0)
        Ok(0.0)
    }
}
```

**Step 3: Run test**
Run: `cargo test -p axum-infrastructure -- tests/tencent_lbs_test.rs -v`
Expected: PASS

**Step 4: Commit**
```bash
git add crates/infrastructure/src/lbs crates/infrastructure/tests/tencent_lbs_test.rs
git commit -m "feat(LBS): 添加腾讯位置服务客户端"
```

---

### Task 9: Wiring + OpenAPI + Remove Sample User Routes

**Files:**
- Modify: `crates/api/src/router.rs`
- Modify: `crates/api/src/state.rs`
- Modify: `bins/server/src/bootstrap.rs`
- Modify: `crates/api/src/openapi.rs`
- Delete: `crates/api/src/handlers/user_handler.rs`
- Delete: `crates/api/src/routes/user.rs`
- Update tests accordingly

**Steps**
1. Wire AppState with new services.
2. Ensure routes include C端与管理端。
3. Update OpenAPI tags.
4. Run `cargo test`.
5. Commit.

---

### Task 10: Migrations + SQLx Prepare

**Files:**
- Create: `migrations/20260303000100_create_core_tables.up.sql`
- Create: `migrations/20260303000100_create_core_tables.down.sql`
- Update: `.sqlx/`

**Steps**
1. Write migration SQL (tables + indexes).
2. Run `cargo sqlx prepare --workspace`.
3. Commit migrations + `.sqlx/`.

---

### Task 11: Final Verification

**Steps**
1. Run `cargo test`.
2. Run `cargo fmt --check`.
3. Summarize results.

