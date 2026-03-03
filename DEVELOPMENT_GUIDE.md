# 开发指南

本模板仅保留最小 `user` CRUD 业务示例，适合作为新项目起点。

## 本地开发

### 1. 启动数据库

```bash
docker compose up -d
```

### 2. 迁移

```bash
cargo install sqlx-cli --no-default-features --features postgres
cargo sqlx migrate run
```

### 3. 启动服务

```bash
cargo run -p axum-server
```

## 常用命令

```bash
cargo check
cargo test
cargo fmt
cargo clippy
```

## 分层约定

- `api`：HTTP/Handler
- `application`：Service/DTO
- `domain`：Entity/Repo Trait
- `infrastructure`：Postgres 实现
