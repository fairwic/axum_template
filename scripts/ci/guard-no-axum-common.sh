#!/usr/bin/env bash
set -euo pipefail

has_violation=0

echo "[guard] check any Cargo.toml dependency on axum-common"
dep_hits="$(rg -n '^\s*axum-common(\.workspace\s*=\s*true|\s*=)' --glob '**/Cargo.toml' . || true)"
if [[ -n "${dep_hits}" ]]; then
  echo "Found forbidden axum-common dependency:"
  echo "${dep_hits}"
  has_violation=1
fi

echo "[guard] check Rust code usage of axum_common::"
code_hits="$(rg -n '\baxum_common::' crates bins --glob '*.rs' || true)"
if [[ -n "${code_hits}" ]]; then
  echo "Found forbidden axum_common:: usage:"
  echo "${code_hits}"
  has_violation=1
fi

echo "[guard] check axum-common-api dependency boundary"
common_api_hits="$(rg -n '^\s*axum-common-api(\.workspace\s*=\s*true|\s*=)' crates/*/Cargo.toml bins/*/Cargo.toml || true)"
if [[ -n "${common_api_hits}" ]]; then
  bad_common_api_hits="$(echo "${common_api_hits}" | grep -v '^crates/api/Cargo.toml:' || true)"
  if [[ -n "${bad_common_api_hits}" ]]; then
    echo "Found forbidden axum-common-api dependency (only crates/api allowed):"
    echo "${bad_common_api_hits}"
    has_violation=1
  fi
fi

echo "[guard] check axum-common-infra dependency boundary"
common_infra_hits="$(rg -n '^\s*axum-common-infra(\.workspace\s*=\s*true|\s*=)' crates/*/Cargo.toml bins/*/Cargo.toml || true)"
if [[ -n "${common_infra_hits}" ]]; then
  bad_common_infra_hits="$(echo "${common_infra_hits}" | grep -v '^crates/infrastructure/Cargo.toml:' || true)"
  if [[ -n "${bad_common_infra_hits}" ]]; then
    echo "Found forbidden axum-common-infra dependency (only crates/infrastructure allowed):"
    echo "${bad_common_infra_hits}"
    has_violation=1
  fi
fi

echo "[guard] check application layer does not depend on utoipa"
if rg -n '^\s*utoipa(\.workspace\s*=\s*true|\s*=)' crates/application/Cargo.toml >/dev/null; then
  echo "Found forbidden utoipa dependency in crates/application/Cargo.toml"
  has_violation=1
fi

echo "[guard] check application runtime does not depend on serde_json"
if awk '
  /^\[dependencies\]/{in_dep=1; next}
  /^\[/{in_dep=0}
  in_dep && $0 ~ /^[[:space:]]*serde_json(\.workspace[[:space:]]*=[[:space:]]*true|[[:space:]]*=)/ {found=1}
  END {exit found ? 0 : 1}
' crates/application/Cargo.toml; then
  echo "Found forbidden serde_json runtime dependency in crates/application/Cargo.toml"
  has_violation=1
fi

if [[ -d "crates/common" ]]; then
  echo "Found forbidden compatibility crate: crates/common"
  has_violation=1
fi

if [[ "${has_violation}" -ne 0 ]]; then
  echo "axum-common has been removed. Use axum-core-kernel / axum-common-api / axum-common-infra."
  exit 1
fi

echo "[guard] pass"
