#!/usr/bin/env bash
set -euo pipefail

: "${COVERAGE_MIN_LINES:=70}"
: "${COVERAGE_MIN_FUNCTIONS:=60}"

export PATH="${HOME}/.cargo/bin:${PATH}"
cargo_cmd="cargo"
if command -v rustup >/dev/null 2>&1; then
  rustup_cargo="$(rustup which cargo 2>/dev/null || true)"
  if [ -n "${rustup_cargo}" ]; then
    cargo_cmd="${rustup_cargo}"
  fi
fi

echo "[coverage] baseline for axum-application + axum-domain"
echo "[coverage] min lines=${COVERAGE_MIN_LINES}, min functions=${COVERAGE_MIN_FUNCTIONS}"

"${cargo_cmd}" llvm-cov clean --workspace
"${cargo_cmd}" llvm-cov \
  --package axum-application \
  --package axum-domain \
  --all-features \
  --summary-only \
  --fail-under-lines "${COVERAGE_MIN_LINES}" \
  --fail-under-functions "${COVERAGE_MIN_FUNCTIONS}"
