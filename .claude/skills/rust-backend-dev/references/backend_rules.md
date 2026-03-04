# rust-backend-dev backend rules（axum_template 对齐）

本文件是技能补充规则。若与仓库内文档冲突，以仓库文档为准。

## 0. 事实优先级

1. `docs/BASE_CONVENTIONS.md`
2. `README.md`
3. `docs/BOOTSTRAP.md`
4. 本补充文件

## 1. Workspace 与分层

- `api`：HTTP 协议层（handler/router/auth/extractor/openapi）。
- `application`：业务编排（service + use-case DTO）。
- `domain`：实体 + 仓储 trait + 领域规则，不依赖 Axum/SQLx。
- `infrastructure`：仓储实现、SQLx model、缓存、外部依赖。
- `runtime`：依赖装配（AppState 构建与 worker 组装）。
- `core-kernel`：统一错误与基础类型。
- `common-api`：响应封套。
- `common-infra`：基础设施公共适配。

## 2. API 层约定

- Handler 放在 `crates/api/src/handlers/`，Route 放在 `crates/api/src/routes/`。
- Handler 使用 `State(state): State<AppState>`（当前项目约定），从 `state` 访问 service。
- Handler 只做参数提取、DTO 映射、调用 service；避免在 handler 内写大段业务编排。
- 返回类型使用 `crate::error::ApiResult<ApiResponse<T>>`。
- API DTO 放在 `crates/api/src/dtos/`，按业务文件划分。
- 需要鉴权时从自定义 extractor（如 `AuthUser`）读取用户上下文。

## 3. Application 层约定

- Service 放在 `crates/application/src/services/`。
- 使用 `Arc<dyn XxxRepository>` 依赖 domain trait，避免依赖具体基础设施实现。
- 输入输出 DTO 放在 `crates/application/src/dtos/`。
- Application 层禁止依赖 API DTO。
- 可选缓存策略使用 cache-aside：读缓存 miss 回源，写库成功后删缓存。

## 4. Domain 层约定

- Repository trait 使用 `async_trait` + `async fn` 风格。
- Entity 保持纯领域对象，不打基础设施注解（如 `sqlx::FromRow`）。
- Domain 层不依赖 Axum/SQLx/Redis 客户端等框架库。

## 5. Infrastructure 层约定

- Postgres 实现放在 `crates/infrastructure/src/postgres/`。
- Model 放在 `crates/infrastructure/src/models/`，负责 `Model <-> Entity` 转换。
- SQL 必须使用 `sqlx::query!` / `sqlx::query_as!`，不拼接字符串 SQL。
- 统一通过 `common-infra` 做 SQLx 错误映射（如唯一键冲突映射）。

## 6. 错误、日志、状态码

- 禁止 `unwrap()` / `expect()`（除非常明确的初始化常量场景）。
- 优先使用 `?` 传播错误，在层边界做错误映射。
- 状态码遵循 REST 风格，常见使用：`400/401/403/404/409/422/500`。
- 敏感参数在日志中 `skip`，不要记录 token、验证码、密码等内容。

## 7. SQLx 离线流程（必须）

SQL 变更后必须执行：

```bash
cargo sqlx migrate run
cargo sqlx prepare --workspace
cargo sqlx prepare --workspace --check
SQLX_OFFLINE=true cargo check --workspace
```

`.sqlx/` 目录必须随代码提交。

## 8. 测试建议

- 优先覆盖业务规则与状态迁移，而非重复 CRUD 冒烟。
- API 层保留少量关键路由用例，复杂规则下沉到 application/domain 测试。
- 每次改动至少补一个业务行为测试。

## 9. Rust 开发规范（通用补充）

### 9.1 Toolchain 与质量门禁

- 使用仓库根目录 `rust-toolchain.toml` 的版本与组件。
- 提交前至少执行：
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets --all-features -D warnings`
  - `cargo check --workspace`
  - `cargo test --workspace`

### 9.2 编码风格

- 单函数单职责，避免超长函数；复杂流程拆为私有方法。
- 优先 early return，减少多层嵌套。
- 避免过度泛型与过早抽象，先满足当前业务。
- 公共函数签名保持稳定，避免无必要 breaking changes。

### 9.3 所有权与并发

- 优先借用，减少无意义 `clone()`。
- 跨任务共享使用 `Arc<T>`；可变共享需显式同步原语并限制作用域。
- trait 对象默认补齐 `Send + Sync`。

### 9.4 错误与日志

- 使用 `AppResult<T>` 贯穿应用路径；边界处映射 `AppError`。
- 业务错误与系统错误分离，避免将内部错误直接返回客户端。
- 日志要有上下文字段（如 `user_id`、`order_id`、`store_id`）。
- 敏感参数必须脱敏或不记录（token、验证码、手机号全量等）。

### 9.5 异步实践

- async 代码路径禁止阻塞操作；必要时 `tokio::task::spawn_blocking`。
- 外部调用设置超时，避免长时间挂起。
- 并发分支使用 `try_join!`/`join!` 时要评估失败传播与取消语义。

### 9.6 测试与回归

- 业务改动必须新增或更新测试。
- 缺陷修复先补失败用例，再修复实现。
- 对关键状态流转（创建/支付/取消/完成）保持回归覆盖。

## 10. Rust 严格规范（Strict Profile）

适用于核心链路、发布前加固和高风险改动。

### 10.1 强制检查命令

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

### 10.2 `unsafe` 约束

- 默认禁止新增 `unsafe`。
- 必须使用时，要求：
  - 代码块最小化并局部封装；
  - 添加 `// SAFETY:` 注释说明前置条件与不变量；
  - 补充边界与并发场景测试。

### 10.3 并发与锁纪律

- 禁止在持锁期间执行网络/数据库/文件 IO。
- 限制锁粒度，避免全局锁串行化热点路径。
- 并发组合操作需明确失败传播、超时与取消语义。

### 10.4 契约与迁移纪律

- API 契约默认向后兼容，破坏性变更必须给出迁移窗口。
- 数据库迁移要求可回滚或具备等价补偿方案。
- 结构体序列化字段改动需评估前端、任务进程、历史数据兼容性。

### 10.5 依赖治理

- 新增依赖要做必要性说明与体积/风险评估。
- 禁止引入未知来源依赖，关键依赖需固定版本策略。
- 使用 `deny.toml` 持续检查漏洞、许可证、重复依赖。

### 10.6 性能基线

- 热路径避免重复分配与不必要 clone。
- 数据访问需避免 N+1 与无索引扫描。
- 关键接口在发布前至少一次压测或基准对比。

### 10.7 发布门禁

- 必须提供：观测指标、告警阈值、回滚触发条件、回滚步骤。
- 任一门禁未满足，不允许发布。
