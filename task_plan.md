# Task Plan: 架构标准化改造（阶段一）

## Goal

在不改变核心业务语义的前提下，完成架构边界、工程规范、可维护性与可观测性的标准化升级，优先落地用户确认的 1/2/3/4/5/7/8/9/10/11/12 项。

## Phases

- [x] Phase 1: 现状审查与问题归类（架构/工程/模式/维护性）
- [x] Phase 2: 方案设计与改造蓝图（分阶段）
- [x] Phase 3: 第一批代码改造（依赖注入、配置装配、去重复、OpenAPI 组织）
- [x] Phase 4: 工程守门改造（CI/SQLx 离线/工具链固定/规范自动化）
- [x] Phase 5: 结构性重构（worker 解耦、仓储契约硬化）
- [x] Phase 6: 验证与交付（check/test/offline 全通过）

## Key Questions

1. 是否允许分阶段交付（先落地工程与结构骨架，再重构事务与模块）？
2. 是否在本阶段引入新 crate（core-kernel/common-api/common-infra）？
3. worker 是否拆到独立 bin 并保留向后兼容入口？

## Decisions Made

- 本轮优先落地“标准化骨架”：注入/配置/规范/去重复/文档组织。
- 高风险项（事务边界、crate 拆分、大文件拆分）进入后续阶段，避免一次性大爆炸改造。
- 保持 API 语义兼容，优先做内部结构升级。

## Errors Encountered

- `openskills read planning-with-files` 在当前路径不可用，改用 superpowers 对应技能加载。
- 当前仓库 SQLx 在线/离线基线不一致，需在工程守门阶段修复。

## Status

**Phase 1 已收敛完成** - `事务边界(UoW)` 已覆盖订单关键路径；`crate 边界重构` 已完成 `core-kernel` 落地。  
**Phase 2（进行中）** - 已完成 `common-api` 拆分并迁移 API 响应模型；业务层错误类型已直接依赖 `core-kernel`；新增 `common-infra` 作为基础设施公共适配层并完成 SQLx 映射落地；`common` 兼容层已移除并由 CI 防回流守门；`application` 已移除对 `common-api`/`utoipa` 运行时依赖，并通过 domain 类型别名消除 `serde_json` 运行时依赖。

## 治理轮次（脚手架瘦身，2026-07-21）

裁剪重业务依赖，让脚手架回归最小可运行 DDD 骨架，同时修复被 aws-sdk MSRV 卡死的编译：

- **移除消息与存储重依赖**：删除 `aws-config`/`aws-sdk-s3`/`scylla`/`rdkafka`/`async-nats` 五个依赖及对应实现（`s3_cold_store`/`scylla_hot_store`/`kafka_bus`/`nats_bus`），一并解决 rustc 1.91.1 < aws MSRV 1.94.1 的编译失败。
- **snapshot 能力保留**：新增 `InMemoryHotStore` 作为默认 `HotStore` 实现（参照 `memory/cache.rs`），`EventPublisher` 沿用 `NoopEventPublisher`；`IngestService` 与 API 契约不变。删除无调用方的 `ColdStore` port。
- **删除死代码聚合**：移除 `domain/src/{store,user}` 及 `lib.rs` 导出。
- **清理失效测试**：删除引用已删类型的 9 个测试文件（store/user/order/admin 系列），保留 `address_service_test`、`auth_jwt_test`。
- **清理 AppState 业务污染**：移除 `BizConfig`（跑腿字段）与无调用方的 `with_jwt_config`。
- **worker 占位收敛**：`spawn_order_jobs` → `spawn_background_jobs`，`print!` 改为 `tracing::debug!`。
- **验收**：`cargo check --workspace --all-targets`、`cargo clippy --all-targets`（零警告）、`cargo test --workspace`（全通过）、`cargo fmt --check` 全绿；残留 grep 清零。
