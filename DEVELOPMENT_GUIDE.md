# 开发指南

本模板仅保留最小 `user` CRUD 业务示例，适合作为新项目起点。

## 启动前提

依赖：`Rust`、`Postgres`、`Redis`、`sqlx-cli`（迁移与 `.sqlx` 生成）。

如需完整清单与排查流程，见 `docs/BOOTSTRAP.md`。

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

### 3. SQLx 离线流程

`.sqlx/` 必须提交到仓库，CI 默认使用离线模式。

当 SQL 有变更时，必须重新执行：

```bash
cargo sqlx prepare --workspace
```

无数据库环境编译示例：

```bash
export SQLX_OFFLINE=true
cargo build
```

### 4. 启动服务

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

完整规范见 `docs/BASE_CONVENTIONS.md`。

## 常见问题与排查

1. `DATABASE_URL` 未设置：先 `export DATABASE_URL=...`。
2. SQLx 宏提示无缓存：执行 `cargo sqlx prepare --workspace`。
3. 连接失败：确认 Postgres/Redis 已启动并监听端口。
