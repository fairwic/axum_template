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

**Phase 1 已收敛完成** - `事务边界(UoW)` 已覆盖订单关键路径；`crate 边界重构` 已完成 `core-kernel` 落地，下一步进入 axum/sqlx 进一步外移阶段。
