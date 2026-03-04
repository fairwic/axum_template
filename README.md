# Axum Backend Template

最小化后端模板，保留多 crate 分层结构与一个 `user` CRUD 示例，适合作为新项目脚手架。

## 特性
- 多 crate 分层（`core-kernel` / `common` / `domain` / `application` / `infrastructure` / `api` / `runtime`）
- Postgres + SQLx
- Utoipa OpenAPI + Swagger UI
- 统一错误与响应格式

## 目录结构
```
├── crates/
│   ├── api/            # HTTP 层
│   ├── application/    # 应用层
│   ├── domain/         # 领域层
│   ├── infrastructure/ # 基础设施层
│   ├── runtime/        # 进程装配与调度
│   ├── core-kernel/    # 核心错误与基础类型
│   └── common/         # 通用类型
├── bins/
│   ├── server/         # API 服务入口
│   └── worker/         # 调度任务入口
├── config/             # 配置
└── migrations/         # 数据库迁移
```

## 快速开始

1. 启动依赖：`docker compose up -d`
2. 安装 SQLx CLI（首次）：`cargo install sqlx-cli --no-default-features --features postgres`
3. 迁移：`cargo sqlx migrate run`
4. 生成离线元数据：`cargo sqlx prepare --workspace`
5. 启动 API：`cargo run -p axum-server`
6. 启动 Worker（新终端）：`cargo run -p axum-worker`

详细说明见 `docs/BOOTSTRAP.md`。

访问：
- Swagger UI: `http://localhost:3000/swagger-ui`
- Health: `http://localhost:3000/health`

## 示例请求
```bash
# 创建用户
curl -X POST http://localhost:3000/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# 获取用户
curl http://localhost:3000/api/v1/users/<id>
```

## 环境变量
参考 `.env.example`。
错误响应采用统一 REST 风格状态码（如 400/404/409/422）。

## 项目约定
基础规范见 `docs/BASE_CONVENTIONS.md`。
