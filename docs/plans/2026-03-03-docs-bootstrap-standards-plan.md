# Docs Bootstrap & Base Conventions Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve scaffold usability by adding a clear one-pass startup checklist, fixed SQLX offline workflow, health/observability notes, template rename guidance, and a base conventions document.

**Architecture:** Documentation-only changes. Add two docs under `docs/`, update `README.md` and `DEVELOPMENT_GUIDE.md` to reference them, and align wording with existing routes/configs. No new scripts or command entry points.

**Tech Stack:** Markdown docs only.

---

### Task 1: Add Base Conventions Doc

**Files:**
- Create: `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BASE_CONVENTIONS.md`

**Step 1: Write the failing test**
Not applicable (documentation-only change).

**Step 2: Run test to verify it fails**
Not applicable.

**Step 3: Write minimal implementation**
Create `docs/BASE_CONVENTIONS.md` with sections:
- 分层职责（api/application/domain/infrastructure/common）
- 命名规则（handler/service/repo/dto）
- 错误与响应规范（引用现有统一响应结构）
- SQLx 规范（仅宏 + 必须准备 `.sqlx`）
- 缓存规范（cache-aside、更新后清除、key 规则）

**Step 4: Run tests to verify pass**
Not applicable.

**Step 5: Commit**
```bash
git add /Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BASE_CONVENTIONS.md

git commit -m "docs规范): 添加基础约定说明"
```

---

### Task 2: Add Bootstrap Doc (One-Pass Startup Checklist)

**Files:**
- Create: `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BOOTSTRAP.md`

**Step 1: Write the failing test**
Not applicable.

**Step 2: Run test to verify it fails**
Not applicable.

**Step 3: Write minimal implementation**
Create `docs/BOOTSTRAP.md` with:
- 环境要求（Rust、Postgres、Redis）
- 一键启动清单（docker compose up → migrate → sqlx prepare → run）
- SQLX_OFFLINE 固定流程（`.sqlx` 必须提交）
- 常见错误与修复（.sqlx 缺失/SQLx 宏错误）
- 健康/可观测性建议（/health、/ready 可选、RUST_LOG 示例）

**Step 4: Run tests to verify pass**
Not applicable.

**Step 5: Commit**
```bash
git add /Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BOOTSTRAP.md

git commit -m "docs(引导): 增加启动清单与离线流程"
```

---

### Task 3: Update README Quick Start + Links

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/README.md`

**Step 1: Write the failing test**
Not applicable.

**Step 2: Run test to verify it fails**
Not applicable.

**Step 3: Write minimal implementation**
- Replace quick start with concise 3–5 step checklist.
- Link to `docs/BOOTSTRAP.md` and `docs/BASE_CONVENTIONS.md`.
- Keep existing Swagger/health endpoints consistent.

**Step 4: Run tests to verify pass**
Not applicable.

**Step 5: Commit**
```bash
git add /Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/README.md

git commit -m "docs(readme): 强化快速开始与文档链接"
```

---

### Task 4: Update Development Guide

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/DEVELOPMENT_GUIDE.md`

**Step 1: Write the failing test**
Not applicable.

**Step 2: Run test to verify it fails**
Not applicable.

**Step 3: Write minimal implementation**
- Add “启动前提/依赖说明”
- Add SQLX_OFFLINE 说明与 `.sqlx` 生成流程
- Add 常见问题与排查（无 docker/缺 .sqlx/DB 连接）
- Reference `docs/BOOTSTRAP.md` and `docs/BASE_CONVENTIONS.md`

**Step 4: Run tests to verify pass**
Not applicable.

**Step 5: Commit**
```bash
git add /Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/DEVELOPMENT_GUIDE.md

git commit -m "docs(指南): 补充启动与离线说明"
```

---

### Task 5: Add Template Rename Checklist

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BOOTSTRAP.md`

**Step 1: Write the failing test**
Not applicable.

**Step 2: Run test to verify it fails**
Not applicable.

**Step 3: Write minimal implementation**
Add “模板复制后必改项清单” section in `docs/BOOTSTRAP.md`:
- Cargo workspace/package 名称
- README 标题与仓库地址
- docker-compose 容器名
- `.env.example` 前缀与默认端口
- 搜索替换关键词建议

**Step 4: Run tests to verify pass**
Not applicable.

**Step 5: Commit**
```bash
git add /Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/BOOTSTRAP.md

git commit -m "docs(模板): 增加改名清单"
```

---

Plan complete and saved to `/Users/mac2/onions/axum_template/.worktrees/feature/docs-bootstrap/docs/plans/2026-03-03-docs-bootstrap-standards-plan.md`.

Two execution options:

1. Subagent-Driven (this session)
2. Parallel Session (separate)

Which approach?
