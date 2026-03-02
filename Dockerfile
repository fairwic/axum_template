# syntax=docker/dockerfile:1.7

# --- Chef Base ---
FROM rust:1.88-slim AS chef
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev clang lld ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked

ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld" \
    SQLX_OFFLINE=true \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    CARGO_TARGET_DIR=/app/target

# --- Planner: 生成依赖配方（只要 Cargo.toml/Cargo.lock 不变，这层就稳） ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Cacher: 编译依赖层（依赖层缓存命中率极高） ---
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --recipe-path recipe.json
# cargo chef cook --release --recipe-path recipe.json
# --- Builder: 编译你的二进制（只编你改动的那部分） ---
FROM chef AS builder
COPY . .

# 暂时使用 debug 模式加速编译（恢复时改回 --release 和 target/release）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build -p axum-server --bin axum-server && \
    install -m 0755 /app/target/debug/axum-server /app/axum-server
# cargo build --release -p axum-server --bin axum-server && \
# install -m 0755 /app/target/release/axum-server /app/axum-server

# --- Runtime ---
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app

COPY --from=builder /app/axum-server ./axum-server
COPY --from=builder /app/config ./config
# 需要自动迁移就打开
# COPY --from=builder /app/migrations ./migrations

# ✅ HTTPS / AWS SDK 必备：CA 证书
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

# ✅ 可选：如果你用 chrono::Local / 需要正确时区（建议保留，几乎不占空间）
ENV TZ=Asia/Shanghai
COPY --from=builder /usr/share/zoneinfo/Asia/Shanghai /usr/share/zoneinfo/Asia/Shanghai

ENV RUN_MODE=production
EXPOSE 8080
ENTRYPOINT ["./axum-server"]
