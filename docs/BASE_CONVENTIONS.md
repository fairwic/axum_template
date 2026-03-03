# 基础约定

**分层职责**
api：HTTP/Handler/路由/中间件，仅做协议转换与参数校验。
application：Service/UseCase DTO，负责业务编排与事务边界。
domain：Entity/Repository Trait/领域规则，不依赖框架。
infrastructure：Repository 实现/数据库/缓存/外部依赖。
core-kernel：核心错误模型与基础类型（跨层共享）。
common：API 侧通用响应结构与兼容导出。

**DTO 分层与目录规范**
- API 层 DTO：放在 `crates/api/src/dtos/`，按业务单独文件（如 `order_dto.rs`、`runner_order_dto.rs`）。
- Application 层输入/输出：放在 `crates/application/src/dtos/`，按业务单独文件（如 `order_dto.rs`）。
- Service 文件只保留业务流程，不内联定义 Input/Output 结构体。
- Handler 负责映射：`api dto <-> application input/output`。
- Application 层禁止依赖 API 层 DTO（只依赖 domain/common）。

**命名规则**
Handler：`{resource}_handler` 或 `{resource}_routes`（如 `user_routes`）。
Service：`XxxService`。
Repository Trait：`XxxRepository`，实现：`PgXxxRepository`。
DTO：`CreateXxxDto`/`UpdateXxxDto`/`XxxResponse`。
API Path：`/api/v1/...`。

**Repository 契约规范**
- Repository trait 禁止提供“默认未实现”方法（如返回 `not implemented`）。
- 新增 trait 方法后，所有实现（含测试内存仓储）必须在编译期补齐。
- 通过编译约束替代运行时兜底，避免线上路径触发隐藏分支。

**进程职责规范**
- `axum-server` 仅承担 HTTP API 职责。
- `axum-worker` 承担定时任务调度职责（自动关单、自动接单等）。
- 两个进程共享 `crates/runtime` 的装配逻辑，避免入口代码分叉。

**错误与响应规范**
统一使用 `ApiResponse` 与 `AppError`。
业务校验失败返回 `success=false`，状态码保持 200，错误码在 body 中体现。
示例响应：
```
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "name is required"
  }
}
```

**SQLx 规范**
所有 SQL 必须使用 `sqlx::query!` / `sqlx::query_as!` 宏。
`.sqlx/` 必须提交，CI 默认使用 `SQLX_OFFLINE=true`。
任何 SQL 变更后必须重新执行 `cargo sqlx prepare --workspace`。

**缓存规范**
使用 cache-aside：先读缓存，未命中查 DB，再回填。
更新/删除后必须清理缓存。
Key 规则示例：`user:{id}`。
默认 TTL 通过配置统一控制。

**测试策略（轻量化 TDD）**
- 优先覆盖“业务交互与状态迁移”路径：认证、下单、支付、取消、自动任务、权限边界。
- 纯 CRUD/通道型测试（仅验证读写成功、无业务规则）默认不新增或逐步删除。
- API 层保留少量冒烟测试，复杂业务规则下沉到 application/domain 层测试。
- 每次改动至少新增或更新一个与业务行为相关的测试用例，而不是堆叠重复 CRUD 用例。
