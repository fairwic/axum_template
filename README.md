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

## 开发规范

### DDD 分层约束

严格遵循依赖方向，上层依赖下层，禁止反向依赖与跨层调用：

```
bins (server/worker)
  ↓
runtime (AppState 装配与生命周期)
  ↓
api (HTTP routes/handlers/middleware)
  ↓
application (Service 业务编排与 DTO)
  ↓
domain (Entity/Repo trait/领域逻辑) + infra (Repo 实现/外部适配)
  ↓
core-kernel (AppError/DomainError)
```

**禁止事项**：
- API 层不得直接调用 infra 仓储实现（必须通过 application Service）
- domain 层不得依赖 infra 具体实现（只能依赖 trait 抽象）
- application 层不得依赖 axum/utoipa 运行时（DTO 定义放 api 层）
- 任何层不得绕过 core-kernel 自定义错误类型

### 文件大小策略

**目标 ≤ 1000 行，硬上限 2000 行**（Rust/.ts/.tsx/.js/.jsx 等源码文件）。

- 触碰超过 2000 行的历史文件时，**先拆分再改动**（按领域逻辑或业务拆到小模块）。
- 无法一次拆到限制内时，在 commit message / PR 里记录 blocker 和拆分计划。
- 运行检查脚本：`scripts/dev/check_code_file_line_limit.sh`（根目录执行，超 1000 行 WARN，超 2000 行硬失败）。

### 编码规范

1. **注释要求**：
   - 每个新增 `struct`/`enum`/`pub fn` 必须加文档注释（`///` 或 `//!`）。
   - 内部业务复杂的函数写清楚逻辑分支与前置条件。
   - **先写代码并测试通过，再补注释**（避免注释与实现不一致）。

2. **错误处理**：
   - 业务错误用 `AppError`/`DomainError`（core-kernel 定义），不用 `anyhow` 或裸 `String`。
   - infra 层 sqlx 错误映射统一用 `infra-common::db::{map_sqlx_error, map_unique_violation}`。
   - 不得使用 `.unwrap()`/`.expect()`（clippy `unwrap_used = "deny"` 门禁）。

3. **异步与 trait**：
   - 仓储 trait 使用 `#[async_trait]`（domain 定义 trait，infra 实现）。
   - Service 方法直接 `async fn`，无需 async_trait（application 层具体类型）。

4. **SQLx 离线模式**：
   - 本地开发改动查询后运行 `cargo sqlx prepare --workspace` 更新 `.sqlx/` 缓存。
   - CI 通过 `SQLX_OFFLINE=true` 校验，不连真实数据库。

5. **依赖引入**：
   - 优先复用 workspace 已有依赖（见根 `Cargo.toml` `[workspace.dependencies]`）。
   - 新增依赖前评估必要性，避免功能重叠（如已有 `reqwest` 不再引入 `ureq`）。

### 测试要求

1. **单元测试**：
   - Service 层每个公开方法需单测（mock 仓储依赖）。
   - domain 层复杂业务逻辑需单测（纯函数优先）。
   - infra 层适配器可选集成测试（需外部依赖时用 `#[ignore]` 标记）。

2. **集成测试**：
   - API routes 写 `crates/api/tests/` 集成测试（mock AppState 或真实仓储）。
   - 参考现有 `address_service_test.rs`、`auth_jwt_test.rs` 模式。

3. **编译门禁**：
   ```bash
   cargo check --workspace --all-targets   # 必须通过
   cargo clippy --all-targets              # 零警告
   cargo test --workspace                  # 全通过
   cargo fmt --all --check                 # 格式一致
   ```

### Git 与提交规范

1. **提交前检查**：
   ```bash
   cargo clippy --workspace --all-targets --all-features
   cargo fmt --all
   cargo test --workspace
   ```

2. **Commit message**：
   - 格式：`<type>: <简短描述>`（如 `feat: 新增地址管理 API`、`fix: 修正 JWT 过期校验`、`refactor: 拆分 address handler`）。
   - type 可选：`feat`/`fix`/`refactor`/`test`/`docs`/`chore`。

3. **禁止提交**（默认排除，除非明确需要）：
   - 日志文件、pid、截图、`.DS_Store`、`__pycache__`。
   - `*.md`/`*.sql`（文档与 SQL 脚本需显式 `git add`）。
   - `.env`（敏感配置，只提交 `.env.example`）。

4. **SQLx 离线缓存提交**：
   - 改动涉及 `sqlx::query!` 宏时，提交时必须包含 `.sqlx/*.json` 更新。

5. **CI 流程**：
   - GitHub Actions 自动跑 `cargo check`/`clippy`/`test`/`fmt`（见 `.github/workflows/ci.yml`）。
   - SQLx 离线模式校验、依赖审计（`cargo deny check`）、覆盖率基线（见 `scripts/ci/`）。

### 架构演进原则

- **最小权责**：每个 Service/Handler 只做一件事，复杂编排拆到独立 Service。
- **依赖注入**：通过构造函数注入依赖（`Arc<dyn Trait>`），不在方法里直接 `new` 仓储。
- **领域纯度**：domain 层不依赖框架（axum/sqlx/serde），Entity 只保留业务字段与验证逻辑。
- **向后兼容**：API 契约变更需版本化（如 `/api/v2`），已发布接口不破坏性修改。

## 代码提交制约（快速参考）

确保合并节点未被挂起，触发前核心执行安全要求：

```bash
cargo clippy --workspace --all-targets --all-features
cargo fmt --all
cargo test --workspace
```

严格保持领域向下依赖约束：API 层只能请求 Application，不可越阶跨越。
基础 SQL 查询存在跨级绑定要求，离线检查在 CI 中通过环境变量 `SQLX_OFFLINE=true` 控制。
