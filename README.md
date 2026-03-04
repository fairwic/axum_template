# Rust Web Backend Scaffold

基于 Rust 和 Axum 构建的 Web 后端脚手架，采用 DDD（领域驱动设计）架构与 Workspace 多模块工程组织形式。

## 特性

- 框架层: Axum + Tokio
- 架构层: Domain-Driven Design (DDD) 四层分层架构
- 工程组织: Cargo Workspace 多包管理
- 数据库: PostgreSQL (SQLx 构建，支持编译期查询分析)
- 缓存组件: Redis (Fred)
- 接口文档: Utoipa OpenAPI + Swagger UI 集成
- 安全认证: JWT 解析与拦截控制
- 错误管理: 统一业务系统模型定义与 API 封套转换

## 目录分布

```text
├── crates/
│   ├── api/            # HTTP Controller 路由拦截、接口签名定义及鉴权
│   ├── application/    # 业务层 (Service)、领域对象调度与 DTO 格式转化
│   ├── domain/         # 原子逻辑、实体模型 (Entity) 及仓库依赖抽象 (Trait)
│   ├── infra/          # Db/Cache 及第三方服务驱动层的向下实现
│   ├── runtime/        # 生命周期 AppState 全局依赖编排与上下文装配
│   ├── core-kernel/    # 异常错误总线设计等全局基建
│   ├── api-common/     # 接口外壳与端侧结构化响应结构规范 (ApiResponse)
│   ├── infra-common/   # 基础设施共用支持代码库
├── bins/
│   ├── server/         # HTTP 网关入口与 API 进程启动执行
│   └── worker/         # 异步作业与调度后台守护进程
├── config/             # 多环境隔离配置文件存放
└── migrations/         # 版本化演进 SQL 结构迁移数据
```

## 本地启动

### 依靠组件构建

拉起服务栈容器环境：

```bash
docker compose up -d
```

初始化基础环境变量描述文件：

```bash
cp .env.example .env
```

### 数据库化解

保证 SQLx CLI 开发扩展可用：

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

连接应用所需载体执行结构：

```bash
export DATABASE_URL=postgres://postgres:postgres123@localhost:5432/testdb
cargo sqlx migrate run
```

同步 SQLx 检查层离线映射：

```bash
cargo sqlx prepare --workspace
```

### 进程唤醒

前置守护进程节点启动（API）：

```bash
cargo run -p axum-server
```

旁路任务节点启动（后台任务）：

```bash
cargo run -p axum-worker
```

## 测试地址

- **Swagger UI 交互端**: `http://localhost:3000/swagger-ui`
- **通用健康流检测**: `http://localhost:3000/health`
- **解析 JSON 原档**: `http://localhost:3000/api-docs/openapi.json`

## 代码提交制约

确保合并节点未被挂起，触发前核实执行安全要求：

```bash
cargo clippy --workspace --all-targets --all-features
cargo fmt --all
```

严格保持领域向下依赖约束：API 层只能请求 Application，不可越阶跨越。
基础 SQL 查询存在跨级绑定要求，离线检查在 CI 中通过环境变量 `SQLX_OFFLINE=true` 控制。
