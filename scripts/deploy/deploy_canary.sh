#!/usr/bin/env bash
set -euo pipefail

: "${DEPLOY_SSH_USER:?DEPLOY_SSH_USER is required}"
: "${DEPLOY_SSH_HOST:?DEPLOY_SSH_HOST is required}"
: "${SERVER_APP_PATH:?SERVER_APP_PATH is required}"
: "${DEPLOY_IMAGE:?DEPLOY_IMAGE is required}"

compose_file="${DEPLOY_COMPOSE_FILE:-docker-compose.yml}"
canary_service="${DEPLOY_CANARY_SERVICE:-axum-server-canary}"
health_url="${DEPLOY_CANARY_HEALTHCHECK_URL:-}"
health_retries="${DEPLOY_HEALTHCHECK_RETRIES:-20}"
health_interval="${DEPLOY_HEALTHCHECK_INTERVAL_SECONDS:-5}"

ssh "${DEPLOY_SSH_USER}@${DEPLOY_SSH_HOST}" \
  bash -s -- \
  "${SERVER_APP_PATH}" \
  "${compose_file}" \
  "${canary_service}" \
  "${DEPLOY_IMAGE}" \
  "${health_url}" \
  "${health_retries}" \
  "${health_interval}" <<'REMOTE'
set -euo pipefail

server_app_path="$1"
compose_file="$2"
service="$3"
target_image="$4"
health_url="$5"
health_retries="$6"
health_interval="$7"

cd "${server_app_path}"
mkdir -p .deploy

if ! docker compose -f "${compose_file}" config --services | grep -Fxq "${service}"; then
  echo "service '${service}' is not defined in ${compose_file}"
  exit 1
fi

container_id="$(docker compose -f "${compose_file}" ps -q "${service}" || true)"
if [ -n "${container_id}" ]; then
  docker inspect --format '{{.Config.Image}}' "${container_id}" > ".deploy/${service}.previous_image"
fi

override_file=".deploy/${service}.release.override.yml"
cat > "${override_file}" <<EOF
services:
  ${service}:
    image: ${target_image}
EOF

docker compose -f "${compose_file}" -f "${override_file}" pull "${service}"
docker compose -f "${compose_file}" -f "${override_file}" up -d --no-deps "${service}"
docker compose -f "${compose_file}" -f "${override_file}" ps "${service}"

if [ -n "${health_url}" ]; then
  for attempt in $(seq 1 "${health_retries}"); do
    if curl -fsS --max-time 5 "${health_url}" > /dev/null; then
      echo "canary health check passed: ${health_url}"
      exit 0
    fi
    if [ "${attempt}" -lt "${health_retries}" ]; then
      sleep "${health_interval}"
    fi
  done
  echo "canary health check failed after ${health_retries} attempts: ${health_url}"
  exit 1
fi
REMOTE

