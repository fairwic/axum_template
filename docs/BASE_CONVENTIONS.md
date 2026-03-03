# 基础约定

**分层职责**
api：HTTP/Handler/路由/中间件，仅做协议转换与参数校验。
application：Service/UseCase DTO，负责业务编排与事务边界。
domain：Entity/Repository Trait/领域规则，不依赖框架。
infrastructure：Repository 实现/数据库/缓存/外部依赖。
common：统一错误、响应结构、通用类型。

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
