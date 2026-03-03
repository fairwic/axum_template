# BJD 后端开发规则 (Rust)

必要原则：所有代码都必须能够接受生产环境的考验

## 目录

- [项目架构](#项目架构)
  - [四层架构规则](#四层架构规则)
  - [Workspace 结构](#workspace-结构)
- [命名规范](#命名规范)
  - [必须遵循](#必须遵循)
  - [DTO 命名](#dto-命名)
  - [Repository 方法命名](#repository-方法命名)
- [Interfaces (API) 层规则](#interfaces-api-层规则)
  - [Handler 函数](#handler-函数)
  - [State 提取规范](#state-提取规范)
  - [OpenAPI 注解](#openapi-注解)
  - [状态码使用](#状态码使用)
- [Application (Service) 层规则](#application-service-层规则)
  - [Service 结构](#service-结构)
  - [构造函数参数顺序](#构造函数参数顺序)
  - [业务方法](#业务方法)
  - [缓存规则](#缓存规则)
  - [私有辅助方法](#私有辅助方法)
- [Domain 层规则](#domain-层规则)
  - [Entity (聚合根)](#entity-聚合根)
  - [Repository Trait](#repository-trait)
  - [领域错误](#领域错误)
- [Infrastructure 层规则](#infrastructure-层规则)
  - [Repository 实现](#repository-实现)
  - [Model 结构](#model-结构)
  - [SQLx 使用](#sqlx-使用)
  - [类型转换](#类型转换)
- [DTO 设计规则](#dto-设计规则)
  - [Request DTO](#request-dto)
  - [Response DTO](#response-dto)
  - [DTO 转换](#dto-转换)
- [认证授权规则](#认证授权规则)
  - [JWT Claims](#jwt-claims)
  - [认证中间件](#认证中间件)
  - [路由划分](#路由划分)
  - [Extractor](#extractor)
- [错误处理规则](#错误处理规则)
  - [错误传播](#错误传播)
  - [错误日志](#错误日志)
- [数据库规范](#数据库规范)
  - [表设计](#表设计)
  - [查询规范](#查询规范)
- [缓存规范](#缓存规范)
  - [Redis 键命名](#redis-键命名)
  - [缓存策略](#缓存策略)
- [性能规范](#性能规范)
- [安全规范](#安全规范)
- [测试规范](#测试规范)
- [日志规范](#日志规范)
- [Clippy 配置](#clippy-配置)
- [代码提交前检查清单](#代码提交前检查清单)

---

## 项目架构

### 四层架构规则
- **必须**遵循 DDD 四层架构: Interfaces → Application → Domain → Infrastructure
- **必须**使用依赖倒置: Domain 层定义接口，Infrastructure 层实现
- **禁止**跨层直接调用（如 Handler 直接调用 Repository）
- **禁止** Domain 层依赖外部框架（Axum、SQLx 等）

### Workspace 结构

- **必须**使用 `crates/` 分离各层，`bins/server/` 为入口
- **禁止**使用 `mod.rs`，所有子模块在 `lib.rs` 内联声明
- 单文件模块直接用 `.rs` 文件
- 目录模块使用 `pub mod user { pub mod entity; }`
- DTO 目录必须分层：
  - API DTO：`crates/api/src/dtos/<business>_dto.rs`
  - Application 用例输入输出：`crates/application/src/dtos/<business>_dto.rs`
- 同一业务功能的 DTO 必须聚合在一个 DTO 文件中，禁止在 handler/service 内联定义

---

## 命名规范

### 必须遵循

- 文件/模块: `snake_case` (如 `user_service.rs`)
- Struct/Enum/Trait: `PascalCase` (如 `UserRepository`)
- 函数/方法: `snake_case` (如 `find_by_id`)
- 常量: `SCREAMING_SNAKE_CASE` (如 `MAX_PAGE_SIZE`)

### DTO 命名

- Request DTO: `动词 + 实体 + Dto` (如 `RegisterUserDto`)
- Response DTO: `实体 + Response` (如 `UserResponse`)
- Query DTO: `List + 实体 + Query` (如 `ListUsersQuery`)

### Repository 方法命名

- 查询单个: `find_by_xxx` (返回 `Option<T>`)
- 查询列表: `find_xxx` (返回 `Vec<T>`)
- 检查存在: `exists_by_xxx` (返回 `bool`)
- 计数: `count_by_xxx` (返回 `i64`)
- 保存: `save` (insert or upsert)
- 更新: `update`
- 删除: `delete` (物理删除)

---

## Interfaces (API) 层规则

### Handler 函数

- **必须**按顺序提取参数: State → Extension → Path → Query → Body
- **必须**使用 `ValidatedJson<T>` 自动校验请求体
- **必须**添加 `#[utoipa::path]` OpenAPI 注解
- **禁止**包含业务逻辑
- **禁止**直接调用 Repository
- 路径命名原则（使用snake_case、路径包含函数名、使用{id}等）
- 命名模式（列表/创建/获取/更新/删除的标准格式）
- 正确与错误示例对比
- 路由文件组织规范
- 成功返回使用标准示例： Ok(ApiResponse::success(*)

### State 提取规范

- **必须**直接提取具体 Service: `State(service): State<Arc<ConcreteXxxService>>`
- **禁止**提取整个 AppState 再导航: `State(state): State<AppState>`
- Handler 只能依赖单一 Service，不能访问多个 Service
- 依赖关系必须在 Service 层协调，不在 Handler 层

### OpenAPI 注解

- **必须**使用中文描述
- **必须**标注所有可能的响应状态码
- **必须**为受保护路由添加 `security(("jwt_auth" = []))`
- Decimal 类型**必须**标注 `#[schema(value_type = f64)]`

### 状态码使用

- 200: 查询、更新成功
- 201: 创建成功 (使用 `StatusCode::CREATED`)
- 204: 删除成功 (使用 `StatusCode::NO_CONTENT`)
- 400: 参数验证失败 (`AppError::Validation`)
- 401: 未认证 (`AppError::Unauthorized`)
- 403: 无权限 (`AppError::Forbidden`)
- 404: 资源不存在 (`AppError::NotFound`)
- 409: 资源冲突 (`AppError::Conflict`)
- 500: 服务器错误 (`AppError::Internal`)

---

## Application (Service) 层规则

### Service 结构

- **必须**使用泛型约束依赖 Repository Trait
- **必须**通过 `Arc<R>` 持有 Repository
- **禁止**直接依赖 Infrastructure 层实现
- **禁止**包含领域验证逻辑（应在 Entity 中）

### 构造函数参数顺序

1. 核心 Repository (按依赖顺序)
2. 基础设施服务 (cache, logger 等)
3. 配置参数

### 业务方法

- **必须**使用 `#[instrument(skip(self, dto))]` 追踪（敏感参数用 `skip`）
- **必须**使用结构化日志 (如 `info!(user_id = %id, "操作成功")`)
- **必须**在方法开头检查权限和前置条件
- **禁止**使用 `unwrap()`，**必须**使用 `?` 传播错误

### 缓存规则

- **必须**使用 Cache-Aside 模式
- 查询操作: 先查缓存 → 未命中查数据库 → 写入缓存
- 更新操作: 更新数据库 → 清除缓存
- **必须**定义缓存 TTL 常量
- **必须**定义缓存键前缀常量

### 私有辅助方法

- 长业务流程**必须**拆分为私有方法
- 重复逻辑**必须**提取为私有方法
- 私有方法放在 `// ====== 私有辅助方法 ======` 分隔符后

---

## Domain 层规则

### Entity (聚合根)

- **必须**包含所有领域验证逻辑
- **必须**提供 `validate_xxx()` 验证方法
- **必须**使用构造函数保证不变性
- **禁止**暴露可变引用给外部
- **禁止**依赖外部框架

### Repository Trait

- **必须**使用 `trait UserRepository: Send + Sync`
- **必须**返回 `impl Future<Output = AppResult<T>> + Send`
- 分页查询**必须**返回 `(Vec<T>, i64)` (数据 + 总数)
- **禁止**包含实现细节

### 领域错误

- **必须**使用 `DomainError` (thiserror)
- **必须**有明确的错误类型: `Validation`, `State`, `NotFound`
- 错误消息**必须**使用中文

---

## Infrastructure 层规则

### Repository 实现

- **必须**实现 Domain 层定义的 Trait
- **必须**通过 Model 转换 Entity
- **禁止**将 Entity 直接用于数据库操作
- **禁止**包含业务逻辑

### Model 结构

- **必须**使用 `#[derive(sqlx::FromRow)]`
- 字段类型**必须**精确匹配数据库列类型
- Ulid 存储为 `String` (VARCHAR)
- Enum 存储为 `String` (VARCHAR)
- 业务数据量小的表禁止使用Ulid
- **必须**提供 `from_entity()` 和 `into_entity()` 方法

### SQLx 使用

- **必须**使用 `sqlx::query!` 或 `sqlx::query_as!` (编译时检查)
- **禁止**使用原始 SQL 字符串拼接
- **禁止**在循环中执行 INSERT (使用 `QueryBuilder`)
- Upsert **必须**使用 `ON CONFLICT DO UPDATE`

### 类型转换

- Ulid: `id.to_string()` / `Ulid::from_string(&str)?`
- Enum: 实现 `From<String>` 和 `ToString`
- DateTime: 直接使用 `DateTime<Utc>` (sqlx 自动处理)

---

## DTO 设计规则

### Request DTO

- **必须** `derive(Validate, ToSchema)`
- **必须**添加 `#[validate(...)]` 注解
- **必须**使用中文错误消息
- Handler **必须**使用 `ValidatedJson<T>` 提取

### Response DTO

- **必须** `derive(Serialize, ToSchema)`
- **必须**提供 `from_domain()` 方法聚合多个 Entity
- 简单转换可实现 `From<Entity>`
- **禁止**暴露内部 ID (使用 `id.to_string()`)

### DTO 转换

- API DTO → Application Input：Handler 层负责映射
- Application Input → Entity：Service 层手动构造
- Entity → Application Output：Service 层聚合
- Application Output → API DTO：Handler 层负责映射
- **禁止** Application 层依赖 API 层 DTO

---

## 认证授权规则

### JWT Claims

- **必须**包含字段: `sub` (user_id), `exp`, `role`, `identity_type`
- **必须**使用 Ulid 字符串作为 `sub`
- **禁止**在 Claims 中存储敏感信息

### 认证中间件

- **必须**从 Header 提取 Token
- **必须**验证签名和过期时间
- **必须**将 Claims 注入到 `request.extensions`
- **禁止**在中间件中查询数据库

### 路由划分

- **必须**分为 `public_routes()` 和 `protected_routes()`
- `protected_routes()` **必须**应用 `auth_middleware`
- **禁止**为公开路由添加认证中间件

### Extractor

- **必须**从 `request.extensions` 提取 Claims
- **必须**验证 Claims 完整性
- 验证失败**必须**返回 `AppError::Unauthorized`

---

## 错误处理规则

### 错误传播

- **禁止**使用 `unwrap()` (Clippy: `unwrap_used = "deny"`)
- **禁止**使用 `expect()` (除非 100% 确定不会失败)
- **必须**使用 `?` 传播错误
- **必须**在合适位置转换错误类型

### 错误日志

- 客户端错误 (4xx): 使用 `warn!`
- 服务器错误 (5xx): 使用 `error!`
- **禁止**将内部错误详情暴露给客户端

---

## 数据库规范

### 表设计

- 重要业务表**必须**有 `is_deleted`, `deleted_at` 字段
- 所有表**必须**有 `created_at`, `updated_at` 字段
- 所有字段**必须**有中文注释
- 禁止使用外键
- WHERE 条件字段**必须**建立索引

### 查询规范
- 禁止任何语句使用join,因为不排除我join的表后续会使用缓存
- 分页查询**必须**限制 `MAX_PAGE_SIZE = 100`
- **必须**先查总数，再查数据
---
## 缓存规范
### Redis 键命名

- 实体: `{prefix}:{id}` (如 `user:01ARZ3NDEKTSV4RRFFQ69G5FAV`)
- 列表: `{prefix}:list:{suffix}` (如 `user:list:active`)
- 锁: `lock:{operation}` (如 `lock:order_create`)
- 验证码: `{type}:{purpose}:{identifier}` (如 `sms_code:login:13800138000`)

### 缓存策略

- **必须**使用 Cache-Aside 模式
- **必须**设置合理的 TTL
- **必须**在更新时清除缓存
- **必须**处理缓存穿透（空值缓存）
- **必须**使用 TTL 抖动防止雪崩

---

## 性能规范

- 连接池: `max_connections = (cpu_cores * 2) + disk_spindles`
- **禁止**阻塞操作，I/O **必须**使用 async
- 文件操作**必须**使用 `tokio::fs` 或 `spawn_blocking`
- 批量插入**必须**使用 `QueryBuilder`
- **禁止**在循环中执行数据库查询

---

## 安全规范

- 敏感数据（密码、密钥）**必须**使用 `secrecy::Secret`
- 所有用户输入**必须**使用 `validator` 校验
- **禁止**将内部错误暴露给客户端
- **禁止**在日志中记录敏感信息
- **禁止**在 Claims 中存储密码

---

## 测试规范

- 核心业务逻辑**必须**有单元测试
- Repository 实现**必须**有集成测试
- API Handler **建议**有端到端测试
- 测试覆盖率**建议** > 70%

---

## 日志规范

- 关键操作**必须**使用 `#[instrument(skip(self, 敏感参数))]`
- **必须**使用结构化日志: `info!(user_id = %id, "操作成功")`
- 错误**必须**记录完整上下文
- **禁止**记录敏感信息（密码、Token）

---

## Clippy 配置

```toml
[lints.clippy]
pedantic = "warn"
unwrap_used = "deny"
```

---

## 代码提交前检查清单

提交前**必须**确认:

- [ ] 没有跨层直接调用
- [ ] Request DTO 有 `Validate`，Response DTO 有 `ToSchema`
- [ ] 没有 `unwrap()`，使用 `?` 传播错误
- [ ] 关键操作有 `#[instrument]` 和结构化日志
- [ ] 查询使用了缓存，更新清除了缓存
- [ ] 多表操作使用了事务
- [ ] 公开 API 有 OpenAPI 注解
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 核心逻辑有单元测试
