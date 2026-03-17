#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${APP_DIR}/../.." && pwd)"
APP_LOG="${APP_DIR}/target/incident-platform-smoke.log"

mkdir -p "${APP_DIR}/target"

cd "${APP_DIR}"

pkill -f '/Users/Mahin/Desktop/shaperail/examples/incident-platform/target/debug/incident-platform' >/dev/null 2>&1 || true
sleep 1

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

set -a
source .env
set +a

BASE_URL="http://127.0.0.1:${SHAPERAIL_PORT:-3300}"
API_BASE="${BASE_URL}/v1"
ORG_ID="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
USER_ID="00000000-0000-0000-0000-000000000001"
GRPC_PORT="${INCIDENT_GRPC_PORT:-53051}"
RUN_ID="$(date +%s)"
SERVICE_SLUG="smoke-test-service-${RUN_ID}"
INCIDENT_SLUG="smoke-test-incident-${RUN_ID}"
ALERT_EXTERNAL_ID="smoke-alert-${RUN_ID}"
ALERT_FINGERPRINT="smoke-alert-fingerprint-${RUN_ID}"

docker compose up -d
docker compose exec -T redis redis-cli FLUSHALL >/dev/null

cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p shaperail-cli -- generate >/dev/null

cargo run --features graphql,grpc >"${APP_LOG}" 2>&1 &
APP_PID=$!

cleanup() {
  if kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  if curl -fsS "${BASE_URL}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

curl -fsS "${BASE_URL}/health" >/dev/null

ADMIN_TOKEN="$(curl -fsS "${BASE_URL}/dev/token?user_id=${USER_ID}&role=admin&tenant_id=${ORG_ID}")"
MEMBER_TOKEN="$(curl -fsS "${BASE_URL}/dev/token?user_id=${USER_ID}&role=member&tenant_id=${ORG_ID}")"

SERVICE_RESPONSE="$(
  curl -fsS -X POST "${API_BASE}/services" \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{
      \"name\": \"Smoke Test Service\",
      \"slug\": \"${SERVICE_SLUG}\",
      \"tier\": \"high\",
      \"status\": \"healthy\",
      \"owner_team\": \"Platform\",
      \"runbook_url\": \"https://runbooks.example.com/smoke-test-service\",
      \"created_by\": \"${USER_ID}\"
    }"
)"
SERVICE_ID="$(printf '%s' "${SERVICE_RESPONSE}" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')"

if [[ -z "${SERVICE_ID}" ]]; then
  echo "Failed to extract service id from response" >&2
  exit 1
fi

INCIDENT_RESPONSE="$(
  curl -fsS -X POST "${API_BASE}/incidents" \
    -H "Authorization: Bearer ${MEMBER_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{
      \"service_id\": \"${SERVICE_ID}\",
      \"title\": \"Smoke Test Incident ${RUN_ID}\",
      \"slug\": \"${INCIDENT_SLUG}\",
      \"severity\": \"sev2\",
      \"summary\": \"Synthetic smoke incident for example verification.\",
      \"commander_id\": \"${USER_ID}\",
      \"room_key\": \"temporary-room\",
      \"created_by\": \"${USER_ID}\"
    }"
)"
INCIDENT_ID="$(printf '%s' "${INCIDENT_RESPONSE}" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')"

if [[ -z "${INCIDENT_ID}" ]]; then
  echo "Failed to extract incident id from response" >&2
  exit 1
fi

curl -fsS -X POST "${API_BASE}/alerts" \
  -H "X-API-Key: ${INCIDENT_INGEST_KEY}" \
  -H "Content-Type: application/json" \
  -d "{
    \"org_id\": \"${ORG_ID}\",
    \"service_id\": \"${SERVICE_ID}\",
    \"external_id\": \"${ALERT_EXTERNAL_ID}\",
    \"source\": \"pagerduty\",
    \"severity\": \"sev2\",
    \"fingerprint\": \"${ALERT_FINGERPRINT}\",
    \"summary\": \"Synthetic alert for example verification.\",
    \"payload\": {
      \"source\": \"pagerduty\",
      \"smoke\": true
    }
  }" >/dev/null

GRAPHQL_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/graphql" \
    -H "Authorization: Bearer ${MEMBER_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"query":"query Smoke { list_incidents(limit: 10, offset: 0) { id title status room_key } }"}'
)"

if ! printf '%s' "${GRAPHQL_RESPONSE}" | grep -q "${INCIDENT_ID}"; then
  echo "GraphQL query did not return the created incident" >&2
  exit 1
fi

for _ in $(seq 1 60); do
  SINK_RESPONSE="$(curl -fsS "${BASE_URL}/dev/webhook-sink")"
  if printf '%s' "${SINK_RESPONSE}" | grep -q 'incident.opened'; then
    break
  fi
  sleep 1
done

SINK_RESPONSE="$(curl -fsS "${BASE_URL}/dev/webhook-sink")"
if ! printf '%s' "${SINK_RESPONSE}" | grep -q 'incident.opened'; then
  echo "Outbound webhook sink did not receive incident.opened" >&2
  exit 1
fi

if ! bash -lc "</dev/tcp/127.0.0.1/${GRPC_PORT}" >/dev/null 2>&1; then
  echo "gRPC port ${GRPC_PORT} did not open" >&2
  exit 1
fi

echo "incident-platform smoke test passed"
