# Axum Template 架构脚手架（目录与子目录）

以下为当前项目标准目录骨架，用于快速初始化：

```text
.
├── .github/
│   └── workflows/
├── bins/
│   ├── server/
│   │   └── src/
│   └── worker/
│       └── src/
├── config/
├── crates/
│   ├── api/
│   │   ├── src/
│   │   │   ├── auth/
│   │   │   ├── dtos/
│   │   │   ├── extractors/
│   │   │   ├── handlers/
│   │   │   ├── openapi/
│   │   │   └── routes/
│   │   └── tests/
│   ├── application/
│   │   ├── src/
│   │   │   ├── dtos/
│   │   │   └── services/
│   │   └── tests/
│   ├── common-api/
│   │   └── src/
│   ├── common-infra/
│   │   └── src/
│   ├── core-kernel/
│   │   └── src/
│   ├── domain/
│   │   └── src/
│   │       ├── address/
│   │       ├── admin/
│   │       ├── auth/
│   │       ├── cart/
│   │       ├── category/
│   │       ├── order/
│   │       ├── product/
│   │       ├── runner_order/
│   │       ├── store/
│   │       └── user/
│   ├── infrastructure/
│   │   └── src/
│   │       ├── external/
│   │       ├── memory/
│   │       ├── models/
│   │       ├── postgres/
│   │       └── redis/
│   └── runtime/
│       └── src/
├── data/
├── docs/
├── migrations/
└── scripts/
    ├── ci/
    └── deploy/
```

## 分层职责（初始化时必须保持）

- `api`: HTTP 接口层、鉴权、请求响应 DTO、路由。
- `application`: 用例编排与事务边界，不直接依赖具体基础设施实现。
- `domain`: 领域模型与仓储 trait，禁止依赖外部框架。
- `infrastructure`: DB/缓存/外部服务实现，负责 trait 落地。
- `runtime`: 组装依赖与进程运行入口装配。
- `common-api`: API 响应封套、分页等跨接口通用模型。
- `common-infra`: 基础设施通用适配（例如 SQLx 错误映射适配层）。
- `core-kernel`: 错误模型、基础类型、跨层公共约束。

## 初始化建议

1. 先落目录和最小文件（可用脚本自动执行）。
2. 再补 `Cargo.toml` 依赖与 crate 互引。
3. 最后按业务逐层扩展，避免一次性把所有模块写“重”。
