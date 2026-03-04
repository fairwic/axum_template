#!/usr/bin/env bash
set -euo pipefail

target_dir="${1:-}"
if [ -z "${target_dir}" ]; then
  echo "usage: $0 <target_dir>"
  exit 1
fi

mkdir -p "${target_dir}"
cd "${target_dir}"

mkdir -p \
  .github/workflows \
  bins/server/src \
  bins/worker/src \
  config \
  data \
  docs \
  migrations \
  scripts/ci \
  scripts/deploy \
  crates/api/src/auth \
  crates/api/src/dtos \
  crates/api/src/extractors \
  crates/api/src/handlers \
  crates/api/src/openapi \
  crates/api/src/routes \
  crates/api/tests \
  crates/application/src/dtos \
  crates/application/src/services \
  crates/application/tests \
  crates/common-api/src \
  crates/common-infra/src \
  crates/core-kernel/src \
  crates/domain/src/address \
  crates/domain/src/admin \
  crates/domain/src/auth \
  crates/domain/src/cart \
  crates/domain/src/category \
  crates/domain/src/order \
  crates/domain/src/product \
  crates/domain/src/runner_order \
  crates/domain/src/store \
  crates/domain/src/user \
  crates/infrastructure/src/external \
  crates/infrastructure/src/memory \
  crates/infrastructure/src/models \
  crates/infrastructure/src/postgres \
  crates/infrastructure/src/redis \
  crates/runtime/src

touch \
  .github/workflows/ci.yml \
  docs/.gitkeep \
  data/.gitkeep \
  migrations/.gitkeep

cat > Cargo.toml <<'EOF'
[workspace]
resolver = "2"
members = ["crates/*", "bins/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["your-team"]
license = "MIT"

[workspace.dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
ulid = { version = "1", features = ["serde"] }
EOF

cat > rust-toolchain.toml <<'EOF'
[toolchain]
channel = "1.88.0"
components = ["rustfmt", "clippy"]
profile = "minimal"
EOF

cat > README.md <<'EOF'
# Axum Template Scaffold

Generated from rust-backend-dev skill scaffold.
EOF

cat > crates/api/Cargo.toml <<'EOF'
[package]
name = "axum-api"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/application/Cargo.toml <<'EOF'
[package]
name = "axum-application"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/common-api/Cargo.toml <<'EOF'
[package]
name = "axum-common-api"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/common-infra/Cargo.toml <<'EOF'
[package]
name = "axum-common-infra"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/core-kernel/Cargo.toml <<'EOF'
[package]
name = "axum-core-kernel"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/domain/Cargo.toml <<'EOF'
[package]
name = "axum-domain"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/infra/Cargo.toml <<'EOF'
[package]
name = "axum-infra"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/runtime/Cargo.toml <<'EOF'
[package]
name = "axum-runtime"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > bins/server/Cargo.toml <<'EOF'
[package]
name = "axum-server"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > bins/worker/Cargo.toml <<'EOF'
[package]
name = "axum-worker"
version.workspace = true
edition.workspace = true
license.workspace = true
EOF

cat > crates/api/src/lib.rs <<'EOF'
pub fn init_api_layer() {}
EOF

cat > crates/application/src/lib.rs <<'EOF'
pub fn init_application_layer() {}
EOF

cat > crates/common-api/src/lib.rs <<'EOF'
pub fn init_common_api_layer() {}
EOF

cat > crates/common-infra/src/lib.rs <<'EOF'
pub fn init_common_infra_layer() {}
EOF

cat > crates/core-kernel/src/lib.rs <<'EOF'
pub fn init_core_kernel() {}
EOF

cat > crates/domain/src/lib.rs <<'EOF'
pub fn init_domain_layer() {}
EOF

cat > crates/infrastructure/src/lib.rs <<'EOF'
pub fn init_infrastructure_layer() {}
EOF

cat > crates/runtime/src/lib.rs <<'EOF'
pub fn init_runtime_layer() {}
EOF

cat > bins/server/src/main.rs <<'EOF'
fn main() {
    println!("axum-server scaffold ready");
}
EOF

cat > bins/worker/src/main.rs <<'EOF'
fn main() {
    println!("axum-worker scaffold ready");
}
EOF

echo "scaffold created at: ${target_dir}"
