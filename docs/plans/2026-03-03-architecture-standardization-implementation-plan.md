# Architecture Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 标准化后端工程架构，提升边界清晰度、可维护性与可重复交付能力，同时保持现有接口行为兼容。

**Architecture:** 采用“先骨架后重构”的渐进式路线：先统一依赖注入、配置装配、工程守门与公共抽象，再推进事务边界、模块拆分与 worker 解耦。全程保持编译与测试基线稳定，避免一次性大改风险。

**Tech Stack:** Rust Workspace, Axum, SQLx, Postgres, Redis, Utoipa, GitHub Actions

### Task 1: 依赖注入标准化（AppState 去服务定位器倾向）

**Files:**
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/router.rs`
- Modify: `crates/api/src/handlers/*.rs`（分批）
- Test: `crates/api/tests/*`

**Step 1: 引入子状态抽象与 FromRef 策略设计**
- 约束：handler 只获取必需依赖。

**Step 2: 迁移一个 vertical slice（如 config/store）验证模式**
- 先小范围迁移，控制回归面。

**Step 3: 扩展至其余 handler 并删除运行时 get_service 分支**

**Step 4: 运行 API tests 验证兼容**

### Task 2: 配置驱动装配标准化（cache/lbs/provider）

**Files:**
- Modify: `crates/infrastructure/src/config.rs`
- Modify: `bins/server/src/bootstrap.rs`
- Modify: `config/*.toml`

**Step 1: 增加 provider 枚举配置（cache/lbs）**

**Step 2: 在 bootstrap 里按配置装配具体实现并 fail-fast**

**Step 3: 增加默认配置和环境覆盖示例**

**Step 4: 运行启动验证与关键测试**

### Task 3: Repository 契约硬化（移除默认未实现）

**Files:**
- Modify: `crates/domain/src/*/repo.rs`
- Modify: `crates/infrastructure/src/postgres/*_repo.rs`
- Modify: `crates/application/src/services/*.rs`

**Step 1: 去除默认实现，拆分最小 trait（读/写/库存）**

**Step 2: 修复所有实现编译错误并补齐接口**

**Step 3: 通过编译期约束替代运行时兜底**

### Task 4: 事务边界标准化（UoW）

**Files:**
- Create: `crates/domain/src/transaction.rs`
- Modify: `crates/application/src/services/order_service.rs`
- Modify: `crates/infrastructure/src/postgres/*`

**Step 1: 定义 TransactionManager/UoW 抽象**

**Step 2: 下单/取消/自动任务关键路径迁移到事务上下文**

**Step 3: 增加并发一致性测试**

### Task 5: 公共 extractor/mapper 标准化（去重复）

**Files:**
- Create: `crates/api/src/extractors/*.rs`
- Create: `crates/api/src/mappers/*.rs`
- Modify: `crates/api/src/handlers/*.rs`

**Step 1: 抽取 ULID 与 Auth 用户解析 extractor**

**Step 2: 抽取 DTO <-> Application 输入映射器**

**Step 3: 替换重复代码并保持语义一致**

### Task 6: 模块拆分与可维护性治理（热点文件瘦身）

**Files:**
- Modify: `crates/application/src/services/order_service.rs`
- Modify: `crates/api/src/handlers/order_handler.rs`
- Create: `.../order/{create,preview,pay,list}.rs`

**Step 1: 按用例拆文件，不跨文件共享隐式状态**

**Step 2: 增加模块级测试，确保拆分前后行为一致**

### Task 7: Worker 解耦（从 API 进程剥离）

**Files:**
- Create: `bins/worker/src/main.rs`
- Modify: `bins/server/src/main.rs`
- Modify: `Cargo.toml` workspace/bin

**Step 1: 将调度逻辑迁移到 worker bin**

**Step 2: API 进程保留纯服务职责**

**Step 3: 增加分布式锁/单实例守门（后续）**

### Task 8: OpenAPI 组织标准化

**Files:**
- Modify: `crates/api/src/openapi.rs`
- Create: `crates/api/src/openapi/*.rs`

**Step 1: 按模块拆分 paths/components 聚合**

**Step 2: 保持 swagger 输出兼容**

### Task 9: 工程守门自动化（规范可执行）

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `rust-toolchain.toml`
- Create: `.cargo/config.toml`（可选）
- Modify: `docs/BOOTSTRAP.md`, `docs/BASE_CONVENTIONS.md`

**Step 1: 增加 SQLx offline 检查与 prepare --check**

**Step 2: 固定工具链版本并分离 offline-check / integration-test**

**Step 3: 文档与 CI 规则对齐**

### Task 10: 依赖边界重构（crate 拆分）

**Files:**
- Create: `crates/core-kernel`, `crates/common-api`, `crates/common-infra`
- Modify: 各 crate Cargo.toml 与导入

**Step 1: 先迁移错误与基础类型到 core-kernel**

**Step 2: 逐步迁移 axum/sqlx 相关到外层 crate**

**Step 3: 用 cargo tree 验证边界达标**

## Done Definition

- 编译通过：`cargo check --workspace`
- 单测通过：`cargo test --workspace`
- 离线检查通过：`SQLX_OFFLINE=true cargo check --workspace`
- CI 通过：fmt/clippy/test/offline-check
- 文档同步：README/BOOTSTRAP/BASE_CONVENTIONS 与代码一致
