# Notes: 架构标准化改造（阶段一）

## 审查结论（非业务维度）

- 架构边界：domain 通过 common 间接耦合 axum/sqlx。
- 注入模型：AppState 聚合 + Option 服务，运行时失败面扩大。
- 仓储契约：trait 默认未实现方法导致编译期约束弱。
- 工程基线：SQLx 离线缓存与查询不完全一致，CI 未强制 offline 守门。
- 配置治理：配置存在但部分运行时未按配置装配（cache/lbs）。
- 可维护性：handler/service 重复解析与映射逻辑较多，热点文件偏大。
- 可扩展性：定时任务耦合 API 进程，横向扩容存在重复调度风险。
- 文档组织：OpenAPI 手工集中维护，规模增大后易漏项。

## 本轮落地优先级

1. 注入与配置标准化（2/5/7）
2. 去重复与 API 入口标准化（8/11）
3. 工程守门与规范自动化（12）
4. 预留后续结构改造点（1/3/4/9/10）

## 风险控制

- 保持接口路径与 DTO 字段兼容。
- 大型结构变更分阶段，避免单次 PR 过大。
- 每阶段都保证编译与测试可验证。

## 当前落地结果（本轮）

- 已完成：注入入口收敛、公共 extractor、OpenAPI 结构化、cache/lbs 配置驱动。
- 已完成：worker 独立进程（`axum-worker`）与 runtime 共享装配（`crates/runtime`）。
- 已完成：Repository trait 去默认未实现，所有测试内存仓储补齐编译期约束。
- 已完成：CI 增加 SQLx offline 守门、`cargo sqlx prepare --check`、固定工具链与文档同步。
- 已完成：测试轻量化，移除纯 CRUD/通道型测试，保留业务交互主路径测试集。
- 已完成：事务边界扩展到 goods/runner 的 create/cancel/auto_close/auto_accept 关键路径。
- 已完成：引入 `crates/core-kernel` 承载统一错误模型，`domain` 直接依赖 core-kernel，common 保持兼容导出。
