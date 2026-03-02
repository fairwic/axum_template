# Axum Backend Template

最小化后端模板，保留多 crate 分层结构与一个 `user` CRUD 示例，适合作为新项目脚手架。

## 特性
- 多 crate 分层（`common` / `domain` / `application` / `infrastructure` / `api`）
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
│   └── common/         # 通用类型
├── bins/
│   └── server/         # 服务入口
├── config/             # 配置
└── migrations/         # 数据库迁移
```

## 快速开始

### 1. 准备数据库
```bash
docker compose up -d
```

### 2. 迁移
```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

### 3. 启动服务
```bash
cargo run -p axum-server
```

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
