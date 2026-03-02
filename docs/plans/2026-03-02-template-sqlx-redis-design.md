# Template SQLx Macro + Redis Cache Design (2026-03-02)

## Goals
- Use SQLx macros (`query!`/`query_as!`) for all SQL queries.
- Enable `SQLX_OFFLINE` and keep `.sqlx/` in repo so CI can build without DB.
- Restore Redis caching and provide a minimal cache-aside example in `UserService`.

## Non-goals
- Implement advanced cache invalidation or distributed locking.
- Add additional business modules beyond `user` CRUD.
- Expand API beyond existing minimal endpoints.

## Scope & Architecture
- Keep current multi-crate layout.
- Infrastructure adds `redis/cache.rs` with `RedisCacheService`.
- Application `UserService` depends on `UserRepository + CacheService`.
- API remains unchanged, still uses `UserService` via `AppState`.

## SQLx Macro Mode
- Replace all SQL calls in `PgUserRepository` with `sqlx::query!`/`sqlx::query_as!`.
- Keep `.sqlx/` and document `cargo sqlx prepare --workspace`.
- `SQLX_OFFLINE=true` supported for CI builds.

## Redis Cache-Aside (Minimal)
- Cache key: `user:{id}`.
- `get_user`: check cache first, fallback to DB, then set cache.
- `create/update/delete`: write to DB, then delete cache key.
- Cache errors should not fail the request; log and degrade to DB.

## Config & Docker
- Restore Redis config in `config/*.toml` and `AppConfig`.
- Update `docker-compose.yml` to include Redis service.
- `.env.example` updated with Redis URL and cache TTL.

## Tests
- Add a simple in-memory cache stub to `UserService` tests.
- Keep API test as-is.

## Deliverables
- Macro-based SQLx queries.
- Redis cache module + wiring.
- Updated config, env, docker-compose, docs, and `.sqlx/` usage notes.
