# Ministore Backend

小卖部小程序后端（Rust + Axum），提供：

- C 端 API：`/api/v1/*`
- 管理端 API：`/api/admin/v1/*`
- API 进程：`axum-server`
- 定时任务进程：`axum-worker`

项目采用 workspace 多 crate + DDD 分层结构（`api` / `application` / `domain` / `infrastructure`）。

## 功能概览

- 认证与用户
  - 微信登录（`code -> openid`，需配置微信参数）
  - 手机号验证码登录
  - 会员状态查询
- C 端业务
  - 门店：附近门店、切换当前门店
  - 商品：类目、列表、搜索、详情
  - 地址：增删改查、设为默认地址
  - 购物车：查询、加购、改数量、删除、清空
  - 商品订单：预览、创建、支付、列表、详情、取消、再来一单
  - 跑腿订单：创建、支付、列表、详情、取消
- 管理端业务
  - 管理员登录（平台/门店角色）
  - 门店/类目/商品管理
  - 全局业务配置读写
  - 商品订单与跑腿订单流转（接单/配送/完成）
- Worker 调度任务
  - 自动关闭超时未支付订单
  - 自动接单（按配置超时触发）

## 技术栈

- Rust 2021, Axum, Tokio
- PostgreSQL + SQLx（离线模式）
- Redis（可选缓存实现）
- Utoipa OpenAPI + Swagger UI

## 目录结构

```text
├── crates/
│   ├── api/            # HTTP 路由 / Handler / 鉴权 / OpenAPI
│   ├── application/    # Service / UseCase / DTO
│   ├── domain/         # Entity / Repository Trait / 领域规则
│   ├── infrastructure/ # Postgres / Redis / 外部服务实现
│   ├── runtime/        # AppState 装配与 worker 调度
│   ├── core-kernel/    # 核心错误与基础类型
│   └── common/         # 统一响应与公共导出
├── bins/
│   ├── server/         # axum-server 入口
│   └── worker/         # axum-worker 入口
├── config/             # default/development/production 配置
└── migrations/         # SQL 迁移
```

## 快速开始

1. 启动依赖

```bash
docker compose up -d
```

2. 准备环境变量（可选）

```bash
cp .env.example .env
```

3. 安装 SQLx CLI（首次）

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

4. 执行迁移

```bash
export DATABASE_URL=postgres://postgres:postgres123@localhost:5432/testdb
cargo sqlx migrate run
```

5. 生成 SQLx 离线元数据

```bash
cargo sqlx prepare --workspace
```

6. 启动 API 服务

```bash
cargo run -p axum-server
```

7. 启动 Worker（新终端，可选）

```bash
cargo run -p axum-worker
```

## 访问入口

- Swagger UI: `http://localhost:3000/swagger-ui`
- OpenAPI JSON: `http://localhost:3000/api-docs/openapi.json`
- Health: `http://localhost:3000/health`

## 接口前缀与鉴权

- C 端公开接口：`/api/v1/auth/*`、`/api/v1/config`、`/api/v1/stores/nearby`、`/api/v1/categories`、`/api/v1/products*`
- C 端鉴权接口：`/api/v1/member*`、`/api/v1/addresses*`、`/api/v1/cart*`、`/api/v1/orders*`、`/api/v1/runner_orders*`、`/api/v1/stores/current|select`
- 管理端公开接口：`/api/admin/v1/auth/login`
- 管理端鉴权接口：`/api/admin/v1/*`（除登录外）

鉴权方式：`Authorization: Bearer <token>`。

## 环境变量

配置加载顺序：`config/default.toml` -> `config/{RUN_MODE}.toml` -> `APP__` 前缀环境变量覆盖。

常用变量：

- `RUN_MODE`：运行环境，默认 `development`
- `DATABASE_URL`：SQLx CLI 使用（迁移/prepare）
- `APP__DATABASE__URL`：运行时数据库连接串覆盖
- `APP__REDIS__URL`：Redis 连接串
- `APP__RUNTIME__CACHE_PROVIDER`：`memory` 或 `redis`
- `APP__AUTH__JWT_SECRET`：JWT 签名密钥
- `APP__WECHAT__APP_ID` / `APP__WECHAT__APP_SECRET`：微信登录参数
- `RUST_LOG`：日志级别

说明：

- 默认短信网关为日志实现，验证码会输出到服务日志中（开发环境）。
- 若未配置微信 `APP_ID/APP_SECRET`，微信登录接口会返回配置错误。

## 常用开发命令

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
```

SQLx 离线检查：

```bash
cargo sqlx prepare --workspace --check
SQLX_OFFLINE=true cargo check --workspace
```

## 相关文档

- 启动与排查清单：`docs/BOOTSTRAP.md`
- 分层与代码规范：`docs/BASE_CONVENTIONS.md`
- 开发指南：`DEVELOPMENT_GUIDE.md`
