---
name: rust-backend-dev
description: Use when 在当前 axum_template/ministore Rust 后端中创建或修改 API Handler、Service、Repository、DTO、认证鉴权、缓存、SQLx 查询，或按同构架快速初始化新项目脚手架。
---

# Rust 后端开发技能（axum_template 对齐版）

提供当前项目 Rust 后端开发规范和最佳实践指导。

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

## 当前项目事实（基线）

- Workspace 主要目录：`crates/api`、`crates/application`、`crates/domain`、`crates/infrastructure`、`crates/runtime`、`crates/core-kernel`、`crates/common-api`、`crates/common-infra`。
- API Handler 目录是 `crates/api/src/handlers/`，路由在 `crates/api/src/routes/`。
- 本项目 Handler 默认提取 `State(state): State<AppState>`，从 `state` 访问 service。
- Repository trait 在 domain 层使用 `async_trait` + `async fn` 风格定义。
- SQL 必须使用 `sqlx::query!` / `sqlx::query_as!`，并维护 `.sqlx/` 离线元数据。
- DTO 分层仍然强制：`api/src/dtos` 与 `application/src/dtos` 分离。

## 脚手架初始化

当用户要求“按当前项目架构快速起一个新项目骨架”时，执行以下流程：

1. 读取目录模板文档：`references/architecture_scaffold.md`
2. 运行脚本生成目录与基础文件：

```bash
bash "${CODEX_HOME:-$HOME/.codex}/skills/rust-backend-dev/scripts/init_project_scaffold.sh" <target_dir>
```

例如：

```bash
bash "${CODEX_HOME:-$HOME/.codex}/skills/rust-backend-dev/scripts/init_project_scaffold.sh" /tmp/new_axum_project
```

3. 进入目标目录后执行：

```bash
cargo fmt --all
cargo check --workspace
```

脚手架只做“标准结构 + 最小可编译入口”，业务代码按具体需求补充。

## DTO 分层约定（强制）

- API DTO 放在 `crates/api/src/dtos/`，按业务单文件维护（如 `order_dto.rs`）。
- Application 用例 Input/Output 放在 `crates/application/src/dtos/`，按业务单文件维护。
- Service 文件只保留业务逻辑，禁止内联定义 Input/Output 结构体。
- Handler 显式做映射：`api dto <-> application input/output`。
- Application 层禁止依赖 API 层 DTO。

## Rust 开发规范（新增）

### 1. 工具链与代码质量门禁

- 统一使用仓库 `rust-toolchain.toml` 指定版本，不私自切换 toolchain。
- 提交前必须执行：
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets --all-features -D warnings`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 涉及 SQL 变更时必须额外执行：
  - `cargo sqlx prepare --workspace`
  - `cargo sqlx prepare --workspace --check`

### 2. 代码风格与可读性

- 函数保持单一职责；超过约 80 行优先拆分私有辅助函数。
- 优先返回早退出（guard clauses），减少深层嵌套。
- 公共 API、复杂业务分支添加简短注释，说明“为什么”而不是“做了什么”。
- 命名语义化，避免 `data`/`tmp`/`foo` 这类弱语义名称。

### 3. 所有权与内存语义

- 避免不必要 `clone()`；先用借用（`&T` / `&mut T`）。
- 跨线程共享使用 `Arc<T>`；可变共享状态优先封装到服务层，不向外泄漏内部可变性。
- trait 对象默认要求 `Send + Sync`，以满足 async 运行时并发约束。

### 4. 错误处理规范

- 禁止 `unwrap()` / `expect()`（除初始化时明确不可失败且注释原因）。
- 统一使用 `AppResult<T>` 与 `AppError` 在层边界做错误映射。
- 用户可见错误保持稳定、可理解；内部细节只进日志，不直接透出给客户端。

### 5. 异步与并发规范

- 在 async 路径中禁止阻塞调用（如同步 IO/重 CPU 计算）；必要时使用 `spawn_blocking`。
- 并发任务必须可取消、可超时（如 `tokio::time::timeout`）。
- 外部依赖调用（DB/Redis/HTTP）按需配置超时，避免悬挂请求。

### 6. 可观测性规范

- 关键 service 方法建议使用 `#[instrument]`，并对敏感字段 `skip`。
- 错误日志至少包含：错误类型、关键业务 ID（如 `user_id`/`order_id`）、调用上下文。
- 避免记录敏感信息：token、验证码、手机号全量、身份证号等。

### 7. 测试规范

- 新增/修改业务逻辑必须配套测试，优先 application/domain 层行为测试。
- 一个测试只验证一个业务行为；测试名称明确输入条件与期望结果。
- Bug 修复必须先补复现用例，再修实现，防止回归。

## Rust 严格规范（Strict Profile）

用于核心链路、上线前加固、或高风险改动。默认比常规规范更严格。

### A. 强制门禁（必须全通过）

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -D warnings
cargo test --workspace
cargo test --workspace --release
cargo check --workspace
cargo check --workspace --all-features
cargo sqlx prepare --workspace --check
SQLX_OFFLINE=true cargo check --workspace
cargo deny check
```

### B. `unsafe` 与并发约束

- 禁止新增 `unsafe` 代码块；确有必要时必须满足：
  - 仅封装在最小私有模块内；
  - 附带 `// SAFETY:` 注释说明不变量；
  - 提供覆盖边界条件的单元测试与文档说明。
- 禁止在 async 任务间共享非线程安全可变状态。
- 锁持有时间最小化，禁止在持锁期间执行外部 IO。

### C. API 与数据契约稳定性

- 对外 API 字段只增不减；删除/改名必须走兼容窗口与迁移说明。
- JSON 字段命名保持稳定，禁止同语义多命名并存。
- 数据库 schema 变更必须包含可回滚迁移策略。

### D. 依赖与供应链安全

- 新增依赖必须说明用途与替代评估，避免“只为一个小函数引入大库”。
- 禁止使用来源不明的 git 依赖与未固定版本的关键依赖。
- 通过 `deny.toml` 执行许可证、漏洞、重复依赖检查并保持清洁。

### E. 性能与资源约束

- 热路径禁止重复分配；优先复用缓冲区与预分配容量。
- 大对象避免无意义拷贝，必要时传引用或 `Arc`。
- N+1 查询、无索引过滤、全表扫描必须在评审前消除或给出豁免理由。

### F. 发布与回滚纪律

- 关键改动必须提供“上线观察指标 + 回滚条件 + 回滚步骤”。
- 不满足可观测性与回滚条件的改动，不得进入发布分支。

## 详细规范

优先读取项目内文档（以仓库事实为准）：

- `docs/BASE_CONVENTIONS.md`
- `README.md`
- `docs/BOOTSTRAP.md`

技能内补充参考：

- `references/backend_rules.md`
- `references/architecture_scaffold.md`

## 使用流程

### 1. 创建 API 接口

在 `crates/api/src/handlers/` 创建 Handler：

```rust
use axum::{extract::State, Json};
use axum_common_api::ApiResponse;

#[utoipa::path(
    post,
    path = "/auth/sms/send_code",
    request_body = SendSmsCodeDto,
    responses((status = 200, body = ApiResponse<SendSmsCodeResponse>)),
    tag = "Auth"
)]
pub async fn send_sms_code(
    State(state): State<AppState>,
    Json(payload): Json<SendSmsCodeDto>,
) -> crate::error::ApiResult<ApiResponse<SendSmsCodeResponse>> {
    state.user_service.send_login_sms_code(payload.phone).await?;
    Ok(ApiResponse::success(SendSmsCodeResponse {
        expires_in_secs: state.sms_code_ttl_secs,
    }))
}
```

在 `crates/api/src/routes/` 注册路由。

### 2. 实现业务逻辑

在 `crates/application/src/services/` 创建 Service：

```rust
#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
    cache: Arc<dyn CacheService>,
}

impl UserService {
    pub async fn login_with_phone_sms(
        &self,
        phone: String,
        sms_code: String,
        wechat_code: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> AppResult<User> {
        self.verify_sms_code(&phone, &sms_code).await?;
        let user = self
            .login_with_wechat_code(wechat_code, nickname, avatar)
            .await?;
        self.repo.bind_phone(user.id, phone).await
    }
}
```

### 3. 定义领域模型

在 `crates/domain/src/` 定义 Entity 和 Repository Trait：

```rust
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>>;
    async fn create(&self, user: &User) -> AppResult<User>;
}
```

### 4. 实现数据访问

在 `crates/infrastructure/src/` 实现 Repository：

```rust
pub struct PgUserRepository {
    pool: PgPool,
}

impl UserRepository for PgUserRepository {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            FROM users
            WHERE openid = $1
            "#,
            openid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(|model| model.into_entity()).transpose()
    }
}
```

### 5. 设计 DTO

在 `crates/application/src/dtos/` 定义：

```rust
pub struct CreateGoodsOrderInput {
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub delivery_type: DeliveryType,
    pub items: Vec<GoodsOrderItem>,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<JsonValue>,
    pub store_snapshot: Option<JsonValue>,
    pub remark: Option<String>,
}
```

API DTO 在 `crates/api/src/dtos/`，并由 Handler 做映射（例如 `CreateOrderRequest -> CreateGoodsOrderInput`）。

## 快速参考

### 关键路径

- Handler：`crates/api/src/handlers/*`
- Route：`crates/api/src/routes/*`
- API DTO：`crates/api/src/dtos/*`
- Application DTO：`crates/application/src/dtos/*`
- Service：`crates/application/src/services/*`
- Domain 实体与仓储：`crates/domain/src/*`
- Repository 实现：`crates/infrastructure/src/postgres/*`

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
- 422: 业务规则冲突/状态不合法
- 500: 服务器错误

### 错误处理

- **禁止** `unwrap()`
- **必须**使用 `?` 传播错误
- 客户端错误用 `warn!`
- 服务器错误用 `error!`

### SQLx 离线流程

```bash
cargo sqlx migrate run
cargo sqlx prepare --workspace
cargo sqlx prepare --workspace --check
SQLX_OFFLINE=true cargo check --workspace
```

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
    State(state): State<AppState>,  // 正确（当前项目约定）
) -> AppResult<Json<UserResponse>> {
    let user = state.user_service.get_by_id(id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

### ❌ 在 Handler 写复杂业务编排

```rust
// Handler 内大量业务判断/聚合/事务控制
pub async fn handler(
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    // 过多业务逻辑...
    // 过多跨服务编排...
    unimplemented!()
}
```

**✅ 正确做法**：Handler 只做参数提取/DTO 映射，把业务编排下沉到 Service

```rust
pub async fn handler(
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    let result = state.order_service_ref()?.create(input).await?;
    Ok(Json(to_response(result)))
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

## 关键注意事项

- sql 语句必须使用 sqlx 宏
- 缓存更新时必须清除缓存
- Application 层禁止依赖 API 层 DTO
- SQL 变更后必须执行 `cargo sqlx prepare --workspace`
- 敏感参数必须在日志中 `skip`
