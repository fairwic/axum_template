# skills.md — BJD Rust 后端开发规则（AI 快速理解版 · 生产级）

必要原则：**所有代码都必须能够接受生产环境的考验**  
架构核心：**DDD 四层 + 依赖倒置 + 可观测 + 可交付 + 轻量 TDD**
项目规范：** 业务边界清晰（DDD/模块化）可靠性（SLO/降级/容灾/一致性）可观测性（log/metric/trace/告警/runbook）安全（鉴权、数据、审计、供应链）性能与成本（压测、容量、缓存、成本指标）质量体系（测试策略 + 质量门禁）交付闭环（灰度/回滚/复盘）
团队可持续（ADR/文档/评审/上手成本）**

---

## 0. 总目标（AI 写代码时必须遵守）

- ✅ 代码可上线：无 `unwrap/expect/dbg!/panic!/todo!`（除非启动阶段明确可控）
- ✅ 分层清晰：Interfaces → Application → Domain → Infrastructure
- ✅ 依赖倒置：Domain 定义接口（trait），Infrastructure 实现
- ✅ 可观测：关键路径 `#[instrument]` + 结构化日志
- ✅ 可交付：CI 门禁（fmt/clippy/test）必过
- ✅ 轻量 TDD：先列 **<=10 条关键测试用例**，确认后再写测试与实现；实现以通过测试为准

---

## 1. Workspace 与目录结构（强制）

### 1.1 Workspace 结构（必须）

- `crates/` 分层
- `bins/server/` 为入口（启动、装配、路由挂载、tracing 初始化）

推荐成员：

- `crates/api/`：Interfaces（axum handlers/routes、API DTO、OpenAPI）
- `crates/application/`：Application（Service、用例 DTO、事务边界、权限检查入口）
- `crates/domain/`：Domain（Entity/VO、DomainError、Ports Traits）
- `crates/infrastructure/`：Infrastructure（repo/cache/mq 实现、sqlx models、配置/观测落地）
- `core-kernel`：核心错误模型与基础类型（跨层共享）。
- `crates/common/`（可选，极克制）：分页结构、通用错误基类、时间/ID trait

### 1.2 模块声明规则（必须）

- **禁止**使用 `mod.rs`
- 所有子模块在 `lib.rs` 内联声明
- 单文件模块用 `.rs`
- 目录模块使用：
  - `pub mod user { pub mod entity; }`

### 1.3 DTO 文件聚合规则（必须）

- API DTO：`crates/api/src/dtos/<business>_dto.rs`
- Application 用例 DTO：`crates/application/src/dtos/<business>_dto.rs`
- 同一业务 DTO 必须聚合在一个 DTO 文件中
- **禁止**在 handler/service 内联定义 DTO

### 1.4 进程职责规

- `axum-server` 仅承担 HTTP API 职责。
- `axum-worker` 承担定时任务调度职责（自动关单、自动接单等）。
- 两个进程共享 `crates/runtime` 的装配逻辑，避免入口代码分叉。

---

## 2. 分层规则（强制）

### 2.1 Interfaces (API) 层

- 只做：参数提取、DTO 映射、调用单一 Service、错误映射、OpenAPI 注解
- **禁止**业务逻辑
- **禁止**直接调用 Repository
- Handler 参数提取顺序（必须）：
  - `State → Extension → Path → Query → Body`
- 必须使用 `ValidatedJson<T>` 自动校验请求体
- 必须添加 `#[utoipa::path]`（中文描述，标注所有响应码）
- protected 路由必须 `security(("jwt_auth" = []))`

#### State 提取（必须）

- ✅ `State(service): State<Arc<ConcreteXxxService>>`
- ❌ `State(state): State<AppState>`（禁止）
- Handler 只能依赖单一 Service（多个依赖必须在 Service 层协调）

### 2.2 Application (Service) 层

- 只做：用例编排、权限/前置条件、事务边界、缓存策略、调用 domain/ports
- **必须**泛型约束依赖 Domain trait（ports）
- **必须**通过 `Arc<R>` 持有 repo（或 ports）
- **禁止**依赖 Infrastructure 的具体实现
- **禁止**写领域验证（领域验证在 Entity 内）

构造函数参数顺序（必须）：

1. 核心 Repository/Ports（按依赖顺序）
2. 基础设施服务（cache/logger 等）
3. 配置参数

业务方法（必须）：

- `#[instrument(skip(self, dto))]`（敏感参数要 skip）
- 结构化日志：`info!(user_id = %id, "xxx")`
- 方法开头检查权限与前置条件
- **禁止** `unwrap()`，必须 `?` 传播错误

缓存（必须）：Cache-Aside

- 查询：cache → miss 查 DB → 写 cache
- 更新：DB 成功 → **删除缓存**
- 必须：TTL 常量、key 前缀常量、空值缓存、TTL 抖动防雪崩

### 2.3 Domain 层

- 只做：领域模型与规则、不变性、Ports traits、DomainError
- **禁止**依赖外部框架（Axum/SQLx 等）
- Entity（聚合根）必须：
  - 包含所有领域验证逻辑
  - 构造函数保证不变性（`new(...) -> Result<Self, DomainError>`）
  - 提供 `validate_xxx()` 方法（按需）
  - **禁止**暴露可变引用给外部

Repository Trait（必须）：

- `trait XxxRepository: Send + Sync`
- 返回 `impl Future<Output = AppResult<T>> + Send`
- 分页必须返回 `(Vec<T>, i64)`（数据 + 总数）
- **禁止**包含实现细节

领域错误（必须）：

- `DomainError`（thiserror）
- 错误分类清晰：`Validation / State / NotFound`
- 错误消息必须中文

### 2.4 Infrastructure 层

- 只做：实现 Domain ports、DB/Redis/MQ/外部系统落地、Model 转换
- **必须**实现 Domain trait
- **必须**通过 Model <-> Entity 转换
- **禁止** Entity 直接做 DB 操作
- **禁止**业务逻辑

SQLx（必须）：

- 只能用 `query! / query_as!`（编译期检查）
- **禁止**拼接 SQL 字符串
- **禁止**循环 insert（用 `QueryBuilder`）
- upsert 必须 `ON CONFLICT DO UPDATE`

Model（必须）：

- `#[derive(sqlx::FromRow)]`
- 字段类型严格匹配 DB
- Ulid 存 `String`（VARCHAR）
- Enum 存 `String`（VARCHAR）
- 业务数据量小表禁止使用 Ulid
- 必须提供 `from_entity()` 与 `into_entity()`

类型转换（必须）：

- Ulid：`id.to_string()` / `Ulid::from_string(&str)?`
- Enum：实现 `From<String>` 与 `ToString`
- DateTime：使用 `DateTime<Utc>`

---

## 3. 命名规范（强制）

- 文件/模块：`snake_case`
- Struct/Enum/Trait：`PascalCase`
- 函数/方法：`snake_case`
- 常量：`SCREAMING_SNAKE_CASE`

DTO 命名（强制）：

- Request：`动词 + 实体 + Dto`
- Response：`实体 + Response`
- Query：`List + 实体 + Query`

Repository 方法命名（强制）：

- 单个：`find_by_xxx -> Option<T>`
- 列表：`find_xxx -> Vec<T>`
- 存在：`exists_by_xxx -> bool`
- 计数：`count_by_xxx -> i64`
- 保存：`save`（insert/upsert）
- 更新：`update`
- 删除：`delete`（物理删除）

---

## 4. DTO 规则（强制）

Request DTO（必须）：

- 必须有字段涵义注释
- `derive(Validate, ToSchema)`
- 有 `#[validate(...)]`，中文错误消息
- Handler 必须 `ValidatedJson<T>`

Response DTO（必须）：

- 必须有字段涵义注释
- `derive(Serialize, ToSchema)`
- 必须提供 `from_domain()` 聚合多个 Entity（需要时）
- 简单可 `impl From<Entity>`
- **禁止**暴露内部 ID（对外使用 `id.to_string()`）

DTO 转换边界（强制）：

- API DTO → App Input：Handler 映射
- App Input → Entity：Service 手动构造（保证不变性）
- Entity → App Output：Service 聚合
- App Output → API DTO：Handler 映射
- **禁止** Application 依赖 API DTO

---

## 5. 认证授权（强制）

JWT Claims（必须）：

- 字段：`sub(user_id)`, `exp`, `role`, `identity_type`
- `sub` 使用 Ulid 字符串
- **禁止**存敏感信息

认证中间件（必须）：

- 从 Header 提取 token
- 验证签名与过期
- Claims 注入 `request.extensions`
- **禁止**中间件查 DB

路由划分（必须）：

- `public_routes()` / `protected_routes()`
- protected 必须挂 `auth_middleware`

Extractor（必须）：

- 从 `extensions` 提取 Claims
- 不完整/失败返回 `AppError::Unauthorized`

---

## 6. 错误处理（强制）

- **禁止** `unwrap()`
- **禁止** `expect()`（除非 100% 不失败且写明原因）
- 必须 `?` 传播错误
- 在合适位置转换错误类型（domain/app/api 各自边界清晰）

日志分级（必须）：

- 4xx：`warn!`
- 5xx：`error!`
- **禁止**把内部错误细节暴露给客户端

---

## 7. 数据库规范（强制）

表设计（必须）：

- 重要表：`is_deleted`, `deleted_at`
- 所有表：`created_at`, `updated_at`
- 所有字段中文注释
- 禁止外键
- WHERE 条件字段必须有索引

查询（强制）：

- **禁止**任何 join（考虑后续缓存/拆分）
- 分页：`MAX_PAGE_SIZE = 100`
- **必须**先查总数，再查数据

> join 替代策略（建议）：

- 两次查询 + application 聚合（配合缓存）
- 冗余字段（反范式）
- 同步表/物化视图（异步维护，用于列表/检索）

---

## 8. 缓存规范（强制）

Redis Key 命名（必须）：

- 实体：`{prefix}:{id}`
- 列表：`{prefix}:list:{suffix}`
- 锁：`lock:{operation}`
- 验证码：`{type}:{purpose}:{identifier}`

缓存策略（必须）：

- Cache-Aside
- 合理 TTL
- 更新必须清缓存
- 缓存穿透：空值缓存
- TTL 抖动防雪崩

热点击穿（建议）：

- 热点 key 互斥锁（短 TTL lock）
- 或 singleflight（同 key 合并请求）

---

## 9. 性能规范（强制/建议）

- 连接池：`max_connections = (cpu_cores * 2) + disk_spindles`
- **禁止**阻塞操作；I/O 必须 async
- 文件操作：`tokio::fs` 或 `spawn_blocking`
- 批量插入：`QueryBuilder`
- **禁止**在循环中执行 DB 查询

---

## 10. 安全规范（强制）

- 密码/密钥：必须 `secrecy::Secret`
- 所有用户输入：必须 `validator` 校验
- **禁止**把内部错误暴露给客户端
- **禁止**日志记录敏感信息（密码、Token、验证码、私钥）
- **禁止**Claims 存密码/敏感字段

---

## 11. 日志与可观测（强制）

- 关键操作必须 `#[instrument(skip(self, 敏感参数))]`
- 必须结构化日志（带字段）
- 错误必须记录上下文（含 error_code/trace_id/user_id）
- 不记录敏感信息

建议统一字段：

- `trace_id`, `user_id`, `req_id`, `path`, `method`, `status`, `latency_ms`, `error_code`

---

## 12. Clippy 配置（强制）

```toml
[lints.clippy]
pedantic = "warn"
unwrap_used = "deny"
expect_used = "deny"
dbg_macro = "deny"
panic = "deny"
todo = "warn"
```

## 13. 工程交付链路（CI 必做）

PR 门禁（必须）：
cargo fmt --check
cargo clippy -- -D warnings
cargo test

## 14. 测试规范（强制）

核心业务逻辑：必须单元测试

Repository 实现：必须集成测试

API Handler：建议端到端测试

覆盖率建议 > 70%（以关键域行为覆盖为准）

### 15. 提交前检查清单（强制）

无跨层直接调用（Handler 不碰 Repo）
Request DTO：Validate + ToSchema；Response DTO：Serialize + ToSchema
无 unwrap/expect/dbg!/panic!/todo!（按规则）
关键路径有 #[instrument] + 结构化日志
查询有缓存策略；更新清缓存
多表操作使用事务（如适用）
公共 API 有 OpenAPI 注解（中文 + 全响应码 + security）
通过 cargo fmt/clippy/test
核心逻辑有单元测试，repo 有集成测试

## 16 【补充】错误模型推荐形态（生产级：稳定错误码 + 分层适配）

> 目标：内层（domain/application）完全框架无关；外层（api/infra）做适配。
> 约束：对外 message 中文、稳定；内部 detail 仅日志；支持 trace_id 贯穿。

---

### 1) 错误模型分层（必须）

- **DomainError**（domain）
  - 表达：领域规则失败/不变式被破坏/状态不合法
  - 不包含：HTTP/DB/框架类型
- **AppError**（application）
  - 表达：用例层错误（含权限、资源不存在、冲突、外部依赖失败等）
  - 可包装 DomainError
  - 不包含：axum/sqlx 类型
- **Adapter Errors**（外层适配）
  - api：`AppError -> (StatusCode, ApiResponse)`
  - infra：`sqlx::Error/redis/mq -> AppError`

---

### 2) 推荐字段（必须）

#### 2.1 稳定错误码（对外）

- `code: &'static str` 或 `enum ErrorCode`
- **必须稳定**（可用于前端/客户端逻辑与告警聚合）
- 示例：`OK`, `VALIDATION_FAILED`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT`, `INTERNAL`, `UPSTREAM_TIMEOUT`, `DB_ERROR`

#### 2.2 对外消息（中文）

- `message: Cow<'static, str>` 或 `String`
- **必须中文**
- **禁止**暴露内部堆栈/SQL/连接信息

#### 2.3 内部详情（仅日志）

- `detail: Option<String>`
- 记录：底层错误、上下文（比如 sqlx 原始错误）
- **禁止**返回给客户端（除非明确 debug 环境并做脱敏）

#### 2.4 可观测字段

- `trace_id: Option<String>`（可从 request/span 获取）
- `ctx: Vec<(key, value)>` 或结构化字段（用于日志）
