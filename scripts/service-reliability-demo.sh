#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${PORT_SERVICE_RELIABILITY_WORKDIR:-$(mktemp -d "/tmp/portsvc.XXXXXX")}"
KEEP_WORKDIR="${PORT_SERVICE_RELIABILITY_KEEP:-0}"
NODE_ADDR="${PORT_SERVICE_RELIABILITY_NODE_ADDR:-127.0.0.1:9234}"
CONTROL_ADDR="${PORT_SERVICE_RELIABILITY_CONTROL_ADDR:-127.0.0.1:7040}"
NODE_TOKEN="${PORT_SERVICE_RELIABILITY_NODE_TOKEN:-node-secret}"
export PORT_DEMO_TOKEN="${PORT_DEMO_TOKEN:-demo-token}"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
PORT_BIN="$TARGET_DIR/debug/port"
GUEST_AGENT_BIN="$TARGET_DIR/debug/port-guest-agent"

SERVER_CONFIG="$WORKDIR/server-port.toml"
CLIENT_CONFIG="$WORKDIR/client-port.toml"
GUEST_ROOT="$WORKDIR/guest-root"
HOSTED_RUNTIME_ROOT="$WORKDIR/hosted/aws-linux-node"
BOGUS_RUNTIME_ROOT="$WORKDIR/bogus/aws-linux-node"
MACHINE_DIR="$HOSTED_RUNTIME_ROOT/cloud-aws"
SOCKET_PATH="$MACHINE_DIR/guest-agent.sock"
HEALTH_MARKER="$GUEST_ROOT/workspace/healthy"

cleanup() {
  local status=$?
  for pid_var in CONTROL_PLANE_PID NODE_AGENT_PID GUEST_AGENT_PID; do
    local pid="${!pid_var:-}"
    if [[ -n "${pid}" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
  if [[ "$KEEP_WORKDIR" != "1" ]]; then
    rm -rf "$WORKDIR"
  fi
  return "$status"
}
trap cleanup EXIT

wait_for_path() {
  local path="$1"
  for _ in $(seq 1 100); do
    if [[ -e "$path" ]]; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for path '$path'" >&2
  return 1
}

wait_for_tcp() {
  local addr="$1"
  local host="${addr%:*}"
  local port="${addr##*:}"
  for _ in $(seq 1 100); do
    if (echo >/dev/tcp/"$host"/"$port") >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for tcp listener '$addr'" >&2
  return 1
}

wait_for_machine_list() {
  local config="$1"
  local machine="$2"
  for _ in $(seq 1 100); do
    if "$PORT_BIN" --config "$config" machine list | grep -Fq "$machine"; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for hosted machine '$machine' to appear in machine list" >&2
  return 1
}

require_contains() {
  local haystack="$1"
  local needle="$2"
  if ! grep -Fq "$needle" <<<"$haystack"; then
    echo "expected output to contain: $needle" >&2
    echo "$haystack" >&2
    return 1
  fi
}

require_not_contains() {
  local haystack="$1"
  local needle="$2"
  if grep -Fq "$needle" <<<"$haystack"; then
    echo "expected output to omit: $needle" >&2
    echo "$haystack" >&2
    return 1
  fi
}

wait_for_service_status() {
  local config="$1"
  local machine="$2"
  local service="$3"
  local needle="$4"
  local output=""
  for _ in $(seq 1 100); do
    output="$("$PORT_BIN" --config "$config" service status --machine "$machine" --name "$service")"
    if grep -Fq "$needle" <<<"$output"; then
      printf '%s\n' "$output"
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for service status to contain '$needle'" >&2
  echo "$output" >&2
  return 1
}

mkdir -p "$GUEST_ROOT/workspace" "$MACHINE_DIR" "$BOGUS_RUNTIME_ROOT"

(cd "$REPO_ROOT" && cargo build -q -p port --bin port)
(cd "$REPO_ROOT" && cargo build -q -p port-guest-agent --bin port-guest-agent)

cp "$REPO_ROOT/examples/port.toml" "$SERVER_CONFIG"
sed -i \
  "s#endpoint = \"https://port.example.internal\"#endpoint = \"http://$CONTROL_ADDR\"#" \
  "$SERVER_CONFIG"
sed -i \
  "s#runtime_root = \"runtime/hosted/aws-linux-node\"#runtime_root = \"$HOSTED_RUNTIME_ROOT\"#" \
  "$SERVER_CONFIG"
cp "$SERVER_CONFIG" "$CLIENT_CONFIG"
sed -i \
  "s#runtime_root = \"$HOSTED_RUNTIME_ROOT\"#runtime_root = \"$BOGUS_RUNTIME_ROOT\"#" \
  "$CLIENT_CONFIG"

cat >"$MACHINE_DIR/manifest.json" <<EOF
{
  "machine_name": "cloud-aws",
  "pid": 424243,
  "launched_at_unix_s": 1,
  "runtime_dir": "$MACHINE_DIR",
  "firecracker_binary": "/usr/bin/firecracker",
  "config_path": "$MACHINE_DIR/firecracker-config.json",
  "log_path": "$MACHINE_DIR/firecracker.log",
  "stdout_path": "$MACHINE_DIR/console.stdout.log",
  "stderr_path": "$MACHINE_DIR/console.stderr.log",
  "manifest_path": "$MACHINE_DIR/manifest.json"
}
EOF

("$GUEST_AGENT_BIN" --socket "$SOCKET_PATH" --root "$GUEST_ROOT") \
  >"$WORKDIR/guest-agent.stdout.log" 2>"$WORKDIR/guest-agent.stderr.log" &
GUEST_AGENT_PID=$!
wait_for_path "$SOCKET_PATH"

("$PORT_BIN" --config "$SERVER_CONFIG" control-plane serve --control-plane demo --bind "$CONTROL_ADDR") \
  >"$WORKDIR/control-plane.stdout.log" 2>"$WORKDIR/control-plane.stderr.log" &
CONTROL_PLANE_PID=$!
wait_for_tcp "$CONTROL_ADDR"

("$PORT_BIN" --config "$SERVER_CONFIG" node-agent serve --node aws-linux-node --bind "$NODE_ADDR" --token "$NODE_TOKEN") \
  >"$WORKDIR/node-agent.stdout.log" 2>"$WORKDIR/node-agent.stderr.log" &
NODE_AGENT_PID=$!
wait_for_tcp "$NODE_ADDR"
wait_for_machine_list "$CLIENT_CONFIG" "cloud-aws"

echo "workdir: $WORKDIR"
echo "server config: $SERVER_CONFIG"
echo "client config: $CLIENT_CONFIG"

secret_put="$("$PORT_BIN" --config "$CLIENT_CONFIG" service secret put --machine cloud-aws --name demo-token --value s3cr3t)"
echo
echo "service secret put:"
printf '%s\n' "$secret_put"
require_contains "$secret_put" "backend: runtime-file"
require_contains "$secret_put" "materialization: env"
require_contains "$secret_put" "backend path: "

secret_list="$("$PORT_BIN" --config "$CLIENT_CONFIG" service secret list --machine cloud-aws)"
echo
echo "service secret list:"
printf '%s\n' "$secret_list"
require_contains "$secret_list" "secret: demo-token"
require_contains "$secret_list" "backend: runtime-file"

apply_output="$("$PORT_BIN" --config "$CLIENT_CONFIG" service apply --machine cloud-aws --host-group aws-builders --name api --kind service --restart on-failure --health command --health-command /bin/test --health-command=-f --health-command workspace/healthy --secret API_TOKEN=demo-token -- /bin/sh -lc 'count_file=workspace/restarts; count=$(cat "$count_file" 2>/dev/null || echo 0); count=$((count + 1)); printf "%s" "$count" > "$count_file"; if [ "$count" -eq 1 ]; then sleep 0.2; exit 23; fi; trap '\''exit 0'\'' TERM; while :; do sleep 1; done')"
echo
echo "service apply:"
printf '%s\n' "$apply_output"
require_contains "$apply_output" "restart policy: on-failure"
require_contains "$apply_output" "health policy: command"
require_contains "$apply_output" "secret sources: API_TOKEN<=demo-token via runtime-file/env @ "
require_not_contains "$apply_output" "s3cr3t"

first_status="$(wait_for_service_status "$CLIENT_CONFIG" cloud-aws api "restart count: 1")"
echo
echo "service status (unhealthy after restart):"
printf '%s\n' "$first_status"
require_contains "$first_status" "last exit code: 23"
require_contains "$first_status" "health state: unhealthy"
require_contains "$first_status" "health detail: health command exited with code 1"
require_contains "$first_status" "secret sources: API_TOKEN<=demo-token via runtime-file/env @ "
require_not_contains "$first_status" "s3cr3t"

printf 'ok' >"$HEALTH_MARKER"
second_status="$(wait_for_service_status "$CLIENT_CONFIG" cloud-aws api "health state: healthy")"
echo
echo "service status (healthy):"
printf '%s\n' "$second_status"
require_contains "$second_status" "restart count: 1"
require_contains "$second_status" "health detail: (none)"
require_not_contains "$second_status" "s3cr3t"

stop_output="$("$PORT_BIN" --config "$CLIENT_CONFIG" service stop --machine cloud-aws --name api)"
echo
echo "service stop:"
printf '%s\n' "$stop_output"
require_contains "$stop_output" "desired state: stopped"
require_contains "$stop_output" "runtime state: stopped"

echo
echo "current service reliability limits:"
echo "- runtime-file is the shipped repo-local secret backend; external secret managers remain follow-on work"
echo "- the demo keeps one hosted control plane and one node-agent in-process; no multi-tenant secret isolation ships yet"
echo "- autoscaling, preemption, and broader orchestration remain out of scope for this workflow"
