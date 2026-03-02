# SQLx Macro + Redis Cache Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Switch all SQL to SQLx macros with offline `.sqlx` support and add a minimal Redis cache-aside example in `UserService`.

**Architecture:** Keep current multi-crate structure. Add a `CacheService` trait in domain, implement `RedisCacheService` in infrastructure, inject into `UserService`, and update server bootstrap/config. Convert `PgUserRepository` queries to `query!`/`query_as!` and generate `.sqlx` metadata.

**Tech Stack:** Rust, Axum, SQLx (Postgres, offline mode), Redis (fred), Tokio.

---

### Task 1: Add Cache Trait + Redis Module + Dependencies

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/Cargo.toml`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/domain/Cargo.toml`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/Cargo.toml`
- Create: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/domain/src/cache.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/domain/src/lib.rs`
- Create: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/src/redis/cache.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/src/lib.rs`

**Step 1: Write failing unit test for cache trait usage in application (stub)**
Add to `crates/application/tests/user_service_test.rs`:
```rust
#[tokio::test]
async fn test_get_user_reads_cache_first() { /* cache hit returns without DB */ }
```

**Step 2: Run test to verify it fails**
Run: `cargo test -p axum-application test_get_user_reads_cache_first`
Expected: FAIL (cache trait not implemented yet).

**Step 3: Add CacheService trait**
Create `cache.rs`:
```rust
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>>;
    async fn set_string(&self, key: &str, value: &str, ttl_secs: u64) -> AppResult<()>;
    async fn delete(&self, key: &str) -> AppResult<()>;
}
```
Export in `domain/src/lib.rs`.

**Step 4: Add Redis cache implementation (fred)**
Create `infrastructure/src/redis/cache.rs` with `RedisCacheService::new(url, max_connections)` and methods implementing the trait. Use `serde_json` in application, not here.

**Step 5: Update dependencies**
- Workspace: add `fred = { version = "9", features = ["enable-rustls", "partial-tracing"] }`.
- Infrastructure: add `fred.workspace = true`.

**Step 6: Commit**
```bash
git add Cargo.toml crates/domain/src crates/infrastructure/src

git commit -m "feat(cache): 添加缓存接口与Redis实现"
```

---

### Task 2: Wire Redis Config + Bootstrap

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/src/config.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/bins/server/src/bootstrap.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/bins/server/src/main.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/config/default.toml`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/config/development.toml`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/config/production.toml`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/.env.example`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/docker-compose.yml`

**Step 1: Add redis + cache config**
Add:
```toml
[redis]
url = "redis://:password@localhost:6379"
max_connections = 10

[cache]
default_ttl_secs = 300
```

**Step 2: Update AppConfig**
Add `RedisConfig` + `CacheConfig` and expose in `AppConfig`.

**Step 3: Update bootstrap/main**
- Create `RedisCacheService` from config.
- Pass into `UserService`.

**Step 4: Update docker-compose**
Restore `redis` service (password optional; keep consistent with config).

**Step 5: Commit**
```bash
git add bins/server/src config .env.example docker-compose.yml crates/infrastructure/src/config.rs

git commit -m "feat(config): 恢复Redis配置与启动注入"
```

---

### Task 3: Update UserService for Cache-Aside

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/application/src/services/user_service.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/application/tests/user_service_test.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/application/Cargo.toml`

**Step 1: Write failing test**
Add cache hit test and cache miss test (uses in-memory repo + cache stub). Expected cache hit to skip DB.

**Step 2: Run test**
Run: `cargo test -p axum-application test_get_user_reads_cache_first`
Expected: FAIL.

**Step 3: Implement cache-aside**
- Inject `Arc<dyn CacheService>` into `UserService::new`.
- `get_user`: check cache; on hit, return; on miss, query DB then set cache.
- `create/update/delete`: delete cache key.
- Use `serde_json` to serialize/deserialize `User`.

**Step 4: Run tests**
Run: `cargo test -p axum-application`
Expected: PASS.

**Step 5: Commit**
```bash
git add crates/application/src crates/application/tests crates/application/Cargo.toml

git commit -m "feat(application): 增加User缓存示例"
```

---

### Task 4: Convert SQL to SQLx Macros

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/src/postgres/user_repo.rs`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/crates/infrastructure/src/models/user_model.rs`

**Step 1: Replace queries**
Use `sqlx::query_as!` / `sqlx::query!` for all SQL:
```rust
let row = sqlx::query_as!(UserModel, "SELECT ...", ...).fetch_one(&self.pool).await?;
```

**Step 2: Run tests**
Run: `cargo test -p axum-infrastructure`
Expected: PASS.

**Step 3: Commit**
```bash
git add crates/infrastructure/src/postgres/user_repo.rs crates/infrastructure/src/models/user_model.rs

git commit -m "refactor(sqlx): 切换为宏查询"
```

---

### Task 5: Generate .sqlx + Docs

**Files:**
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/README.md`
- Modify: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/DEVELOPMENT_GUIDE.md`
- Create: `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/.sqlx/*` (generated)

**Step 1: Start Postgres for prepare**
Run: `docker compose up -d postgres`

**Step 2: Prepare SQLx metadata**
Run:
```bash
export DATABASE_URL=postgres://postgres:postgres123@localhost:5432/testdb
cargo install sqlx-cli --no-default-features --features postgres
cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare --workspace
```
Expected: `.sqlx/` created.

**Step 3: Update docs**
Add instructions for `SQLX_OFFLINE=true` and `cargo sqlx prepare --workspace`.

**Step 4: Commit**
```bash
git add .sqlx README.md DEVELOPMENT_GUIDE.md

git commit -m "docs(sqlx): 补充离线宏模式说明"
```

---

Plan complete and saved to `/Users/mac2/onions/axum_template/.worktrees/feature/sqlx-redis/docs/plans/2026-03-02-sqlx-macro-redis-implementation-plan.md`.

Two execution options:

1. Subagent-Driven (this session)
2. Parallel Session (separate)

Which approach?
