#!/usr/bin/env bash
set -euo pipefail

: "${DEPLOY_SSH_USER:?DEPLOY_SSH_USER is required}"
: "${DEPLOY_SSH_HOST:?DEPLOY_SSH_HOST is required}"
: "${SERVER_APP_PATH:?SERVER_APP_PATH is required}"

compose_file="${DEPLOY_COMPOSE_FILE:-docker-compose.yml}"
rollback_service="${DEPLOY_ROLLBACK_SERVICE:-${DEPLOY_STABLE_SERVICE:-axum-server}}"
rollback_image="${DEPLOY_ROLLBACK_IMAGE:-}"

ssh "${DEPLOY_SSH_USER}@${DEPLOY_SSH_HOST}" \
  bash -s -- \
  "${SERVER_APP_PATH}" \
  "${compose_file}" \
  "${rollback_service}" \
  "${rollback_image}" <<'REMOTE'
set -euo pipefail

server_app_path="$1"
compose_file="$2"
service="$3"
manual_image="$4"

cd "${server_app_path}"
mkdir -p .deploy

if ! docker compose -f "${compose_file}" config --services | grep -Fxq "${service}"; then
  echo "service '${service}' is not defined in ${compose_file}"
  exit 1
fi

if [ -n "${manual_image}" ]; then
  target_image="${manual_image}"
else
  snapshot_file=".deploy/${service}.previous_image"
  if [ ! -f "${snapshot_file}" ]; then
    echo "rollback snapshot missing: ${snapshot_file}"
    exit 1
  fi
  target_image="$(cat "${snapshot_file}")"
fi

override_file=".deploy/${service}.rollback.override.yml"
cat > "${override_file}" <<EOF
services:
  ${service}:
    image: ${target_image}
EOF

docker compose -f "${compose_file}" -f "${override_file}" pull "${service}" || true
docker compose -f "${compose_file}" -f "${override_file}" up -d --no-deps "${service}"
docker compose -f "${compose_file}" -f "${override_file}" ps "${service}"
REMOTE

