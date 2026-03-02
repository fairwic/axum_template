---
name: rust-backend-dev
description: Rust 后端开发技能，基于 DDD 四层架构。用于开发 BJD 项目的 Rust 后端服务。触发场景：(1) 创建或修改 API Handler、Service、Repository 代码；(2) 设计数据库表结构和 Entity；(3) 实现认证授权逻辑；(4) 处理错误和日志；(5) 编写 DTO 和数据验证；(6) 实现缓存策略；(7) 所有涉及 Rust 后端代码的任务。
---

# Rust 后端开发技能 (BJD 项目)

提供 BJD 项目 Rust 后端开发规范和最佳实践指导。

## 核心架构

采用 DDD 四层架构：

- **Interfaces (API)**: Handler、Router、Middleware
- **Application**: Service 层，协调业务流程
- **Domain**: Entity、Repository Trait、领域逻辑
- **Infrastructure**: Repository 实现、Model、数据库操作

**关键原则**：

- 依赖倒置：Domain 定义接口，Infrastructure 实现
- 禁止跨层调用
- Domain 层不依赖外部框架

## 详细规范

详细开发规范参考：[references/backend_rules.md](references/backend_rules.md)

包含：

- 四层架构规则和 Workspace 结构
- 命名规范 (文件、DTO、Repository 方法)
- Handler/Service/Repository 实现规则
- DTO 设计和数据验证
- 认证授权 (JWT、Claims、Middleware)
- 错误处理和日志规范
- 数据库设计和查询规范
- 缓存策略 (Redis、Cache-Aside)
- 性能和安全规范
- 代码提交前检查清单

## 使用流程

### 1. 创建 API 接口

在 `crates/api/handlers/` 创建 Handler：

```rust
/// 验证旧手机号 (换绑第一步)
#[utoipa::path(
    post,
    path = "/api/v1/users/verify_old_phone",
    request_body = VerifyOldPhoneDto,
    responses(
        (status = 200, description = "验证成功"),
    ),
    tag = AUTH_TAG,
    security(("jwt_auth" = []))
)]
pub async fn verify_old_phone(
    State(service): State<Arc<ConcreteUserService>>,
    auth_user: AuthUser,
    ValidatedJson(dto): ValidatedJson<VerifyOldPhoneDto>,
) -> AppResult<ApiResponse<()>> {
    service.verify_old_phone(auth_user.user_id, dto).await?;
    Ok(ApiResponse::success(()))
}
```

在 `routes/` 注册路由。

### 2. 实现业务逻辑

在 `crates/application/services/` 创建 Service：

```rust
pub struct UserService<R: UserRepository> {
    user_repo: Arc<R>,
    cache: Arc<Cache>,
}

impl<R: UserRepository> UserService<R> {
    #[instrument(skip(self, dto))]
    pub async fn create_user(
        &self,
        dto: CreateUserDto,
        claims: &Claims,
    ) -> AppResult<User> {
        // 权限检查
        if claims.role != Role::Admin {
            return Err(AppError::Forbidden("需要管理员权限".into()));
        }

        // 业务逻辑
        let user = User::new(dto.username, dto.email)?;
        let saved = self.user_repo.save(user).await?;

        // 清除缓存
        self.cache.delete(&format!("user:list")).await?;

        info!(user_id = %saved.id, "用户创建成功");
        Ok(saved)
    }
}
```

### 3. 定义领域模型

在 `crates/domain/` 定义 Entity 和 Repository Trait：

```rust
// Entity
pub struct User {
    pub id: Ulid,
    pub username: String,
    pub email: String,
}

impl User {
    pub fn new(username: String, email: String) -> AppResult<Self> {
        Self::validate_username(&username)?;
        Self::validate_email(&email)?;
        Ok(Self { id: Ulid::new(), username, email })
    }

    fn validate_username(username: &str) -> AppResult<()> {
        if username.len() < 3 {
            return Err(AppError::Validation("用户名至少3个字符".into()));
        }
        Ok(())
    }
}

// Repository Trait
pub trait UserRepository: Send + Sync {
    fn save(&self, user: User) -> impl Future<Output = AppResult<User>> + Send;
    fn find_by_id(&self, id: &Ulid) -> impl Future<Output = AppResult<Option<User>>> + Send;
}
```

### 4. 实现数据访问

在 `crates/infrastructure/` 实现 Repository：

```rust
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: User) -> AppResult<User> {
        let model = UserModel::from_entity(&user);
        sqlx::query!(
            "INSERT INTO users (id, username, email) VALUES ($1, $2, $3)",
            model.id, model.username, model.email
        )
        .execute(&self.pool)
        .await?;
        Ok(user)
    }
}

#[derive(sqlx::FromRow)]
struct UserModel {
    id: String,
    username: String,
    email: String,
}

impl UserModel {
    fn from_entity(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
        }
    }

    fn into_entity(self) -> AppResult<User> {
        Ok(User {
            id: Ulid::from_string(&self.id)?,
            username: self.username,
            email: self.email,
        })
    }
}
```

### 5. 设计 DTO

在 `crates/application/dtos/` 定义：

```rust
#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 50, message = "用户名长度3-50字符"))]
    pub username: String,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }
    }
}
```

## 快速参考

### Handler 参数顺序

```rust
State → Extension → Path → Query → Body
```

### Repository 方法命名

- `find_by_xxx` → `Option<T>`
- `find_xxx` → `Vec<T>`
- `exists_by_xxx` → `bool`
- `count_by_xxx` → `i64`

### 状态码使用

- 200: 查询/更新成功
- 201: 创建成功
- 204: 删除成功
- 400: 参数验证失败
- 401: 未认证
- 403: 无权限
- 404: 资源不存在
- 409: 资源冲突
- 500: 服务器错误

### 错误处理

- **禁止** `unwrap()`
- **必须**使用 `?` 传播错误
- 客户端错误用 `warn!`
- 服务器错误用 `error!`

## 常见错误模式

### ❌ 跨层调用

```rust
// Handler 直接调用 Repository
pub async fn handler(
    State(repo): State<Arc<dyn UserRepository>>,  // 错误！
) -> AppResult<Json<UserResponse>> {
    let user = repo.find_by_id(&id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

**✅ 正确做法**：Handler 调用 Service

```rust
pub async fn handler(
    State(service): State<Arc<UserService>>,  // 正确
) -> AppResult<Json<UserResponse>> {
    let user = service.get_user(&id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

### ❌ 提取整个 AppState

```rust
// 提取整个 AppState 再导航
pub async fn handler(
    State(state): State<AppState>,  // 错误！
) -> AppResult<Json<UserResponse>> {
    let user = state.user_service.get_user(&id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

**✅ 正确做法**：直接提取具体 Service

```rust
pub async fn handler(
    State(service): State<Arc<UserService>>,  // 正确
) -> AppResult<Json<UserResponse>> {
    let user = service.get_user(&id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

### ❌ 使用 unwrap()

```rust
// 使用 unwrap 会导致 panic
let id = Ulid::from_string(&id_str).unwrap();  // 错误！
```

**✅ 正确做法**：使用 ? 传播错误

```rust
let id = Ulid::from_string(&id_str)?;  // 正确
```

### ❌ Domain 层依赖外部框架

```rust
// Entity 依赖 sqlx
#[derive(sqlx::FromRow)]  // 错误！
pub struct User {
    pub id: Ulid,
}
```

**✅ 正确做法**：Infrastructure 层的 Model 依赖框架

```rust
// Domain Entity - 纯领域对象
pub struct User {
    pub id: Ulid,
}

// Infrastructure Model - 数据库映射
#[derive(sqlx::FromRow)]  // 正确
struct UserModel {
    id: String,
}
```

### ❌ 查询使用 JOIN

```rust
// 使用 JOIN 查询
sqlx::query!(
    "SELECT u.*, p.name as profile_name
     FROM users u JOIN profiles p ON u.id = p.user_id"  // 错误！
)
```

**✅ 正确做法**：分别查询，Service 层聚合

```rust
// Repository 分别查询
let user = user_repo.find_by_id(&id).await?;
let profile = profile_repo.find_by_user_id(&id).await?;

// Service 层聚合
UserWithProfileResponse::from_domain(user, profile)
```

## 关键注意事项

- Handler 只能依赖单一 Service
- 查询禁止使用 JOIN (后续可能缓存)
- sql 语句必须使用 sqlx 宏
- 重要业务表必须有软删除字段
- 业务数据量小的表禁止使用 Ulid
- 缓存更新时必须清除缓存
- 敏感参数必须在日志中 `skip`
