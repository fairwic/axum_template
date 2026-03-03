# 启动清单

**环境要求**
`Rust`、`Postgres`、`Redis`。`sqlx-cli` 用于迁移与 `.sqlx` 生成。

**一键启动清单**
1. 启动依赖：`docker compose up -d`
2. 安装 SQLx CLI（首次）：`cargo install sqlx-cli --no-default-features --features postgres`
3. 设置数据库地址并迁移：
```
export DATABASE_URL=postgres://postgres:postgres123@localhost:5432/testdb
cargo sqlx migrate run
```
4. 生成离线元数据：`cargo sqlx prepare --workspace`
5. 启动服务：`cargo run -p axum-server`

**访问入口**
Swagger UI：`http://localhost:3000/swagger-ui`
Health：`http://localhost:3000/health`

**SQLX_OFFLINE 固定流程**
`.sqlx/` 必须提交到仓库，CI 默认使用离线模式。
当 SQL 有变更时，必须重新执行：`cargo sqlx prepare --workspace`。
无数据库环境编译示例：
```
export SQLX_OFFLINE=true
cargo build
```

**模板复制后必改项清单**
1. `Cargo.toml` 的项目描述与仓库地址。
2. `bins/server/Cargo.toml` 的 crate 名称与描述。
3. `README.md` 标题与示例命令中的项目名。
4. `docker-compose.yml` 的 service/container 名称与端口。
5. `.env.example` 的前缀与默认端口。
6. 全局搜索替换关键词：`axum_template`、`axum-`、`testdb`、`postgres123`。

**常见问题与修复**
1. 提示 `DATABASE_URL` 未设置：先 `export DATABASE_URL=...`。
2. 提示 `no cached data for this query`：执行 `cargo sqlx prepare --workspace`。
3. 连接失败：确认 Postgres/Redis 已启动并监听对应端口。

**健康与可观测性建议**
最小健康检查使用 `/health`。如需 DB/Redis 就绪性检查，可新增 `/ready`。
建议设置：`RUST_LOG=info,axum_api=debug,axum_application=debug,axum_infrastructure=debug,sqlx=warn`。
