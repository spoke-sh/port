#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${PORT_HOSTED_HTTP_APP_WORKDIR:-$(mktemp -d "/tmp/port-http-app.XXXXXX")}"
KEEP_WORKDIR="${PORT_HOSTED_HTTP_APP_KEEP:-0}"
NODE_ADDR="${PORT_HOSTED_HTTP_APP_NODE_ADDR:-127.0.0.1:9235}"
CONTROL_ADDR="${PORT_HOSTED_HTTP_APP_CONTROL_ADDR:-127.0.0.1:7041}"
NODE_TOKEN="${PORT_HOSTED_HTTP_APP_NODE_TOKEN:-node-secret}"
export PORT_DEMO_TOKEN="${PORT_DEMO_TOKEN:-demo-token}"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
PORT_BIN="$TARGET_DIR/debug/port"
GUEST_AGENT_BIN="$TARGET_DIR/debug/port-guest-agent"
BUSYBOX_BIN="$(command -v busybox || true)"

SERVER_CONFIG="$WORKDIR/server-port.toml"
CLIENT_CONFIG="$WORKDIR/client-port.toml"
GUEST_ROOT="$WORKDIR/guest-root"
HOSTED_RUNTIME_ROOT="$WORKDIR/hosted/aws-linux-node"
BOGUS_RUNTIME_ROOT="$WORKDIR/bogus/aws-linux-node"
MACHINE_DIR="$HOSTED_RUNTIME_ROOT/cloud-aws"
SOCKET_PATH="$MACHINE_DIR/guest-agent.sock"
SITE_SOURCE="$REPO_ROOT/examples/external-static-site/index.html"
SITE_EXPECTED="external-project-ok"

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

wait_for_curl() {
  local url="$1"
  local expected="$2"
  local output=""
  for _ in $(seq 1 100); do
    if output="$(curl --http1.0 -i -fsS "$url" 2>/dev/null)"; then
      if grep -Fq "$expected" <<<"$output"; then
        printf '%s\n' "$output"
        return 0
      fi
    fi
    sleep 0.05
  done
  echo "timed out waiting for curl response from '$url'" >&2
  echo "$output" >&2
  return 1
}

print_command() {
  printf '$'
  for arg in "$@"; do
    printf ' %q' "$arg"
  done
  printf '\n'
}

extract_forward_listen() {
  local output="$1"
  local line=""
  while IFS= read -r line; do
    if [[ "$line" == forward\ listening:* ]]; then
      printf '%s\n' "${line#forward listening: }"
      return 0
    fi
  done <<<"$output"
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

if [[ -z "$BUSYBOX_BIN" ]]; then
  echo "busybox is required for the hosted HTTP app proof workflow" >&2
  exit 1
fi

if [[ ! -f "$SITE_SOURCE" ]]; then
  echo "external static-site snapshot is missing: $SITE_SOURCE" >&2
  exit 1
fi

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
  "pid": 424244,
  "launched_at_unix_s": 1,
  "runtime_dir": "$MACHINE_DIR",
  "firecracker_binary": "/usr/bin/firecracker",
  "config_path": "$MACHINE_DIR/firecracker-config.json",
  "log_path": "$MACHINE_DIR/firecracker.log",
  "stdout_path": "$MACHINE_DIR/console.stdout.log",
  "stderr_path": "$MACHINE_DIR/console.stderr.log",
  "manifest_path": "$MACHINE_DIR/manifest.json",
  "attached_volumes": []
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
echo "machine: cloud-aws"
echo "host group: aws-builders"
echo "snapshot source: $SITE_SOURCE"
echo "target: 127.0.0.1:18080"

prepare_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  guest exec
  --machine cloud-aws
  --
  "$BUSYBOX_BIN"
  mkdir
  -p
  workspace/site
)
echo
echo "guest exec prepare:"
print_command "${prepare_cmd[@]}"
prepare_output="$("${prepare_cmd[@]}")"
printf '%s\n' "$prepare_output"

copy_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  guest copy
  --machine cloud-aws
  --direction host-to-guest
  --source "$SITE_SOURCE"
  --destination /workspace/site/index.html
)
echo
echo "guest copy:"
print_command "${copy_cmd[@]}"
copy_output="$("${copy_cmd[@]}")"
printf '%s\n' "$copy_output"
require_contains "$copy_output" "copied"
require_contains "$copy_output" "/workspace/site/index.html"

inspect_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  guest exec
  --machine cloud-aws
  --
  "$BUSYBOX_BIN"
  cat
  workspace/site/index.html
)
echo
echo "guest exec inspect:"
print_command "${inspect_cmd[@]}"
inspect_output="$("${inspect_cmd[@]}")"
printf '%s\n' "$inspect_output"
require_contains "$inspect_output" "$SITE_EXPECTED"

apply_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  service apply
  --machine cloud-aws
  --host-group aws-builders
  --name web
  --kind service
  --
  /bin/sh
  -lc
  "exec $BUSYBOX_BIN httpd -f -p 127.0.0.1:18080 -h workspace/site"
)
echo
echo "service apply:"
print_command "${apply_cmd[@]}"
apply_output="$("${apply_cmd[@]}")"
printf '%s\n' "$apply_output"
require_contains "$apply_output" "name: web"
require_contains "$apply_output" "runtime state: running"
require_contains "$apply_output" "target host group: aws-builders"

status_output="$(wait_for_service_status "$CLIENT_CONFIG" cloud-aws web "runtime state: running")"
echo
echo "service status:"
print_command "$PORT_BIN" --config "$CLIENT_CONFIG" service status --machine cloud-aws --name web
printf '%s\n' "$status_output"
require_contains "$status_output" "service route: hosted-control-plane"
require_contains "$status_output" "control plane: demo"
require_contains "$status_output" "node: aws-linux-node"

forward_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  guest forward
  --machine cloud-aws
  --listen 127.0.0.1:0
  --target 127.0.0.1:18080
)
echo
echo "guest forward:"
print_command "${forward_cmd[@]}"
forward_output="$("${forward_cmd[@]}")"
printf '%s\n' "$forward_output"
require_contains "$forward_output" "forward lifecycle: hosted-control-plane"
forward_listen="$(extract_forward_listen "$forward_output")"
echo "forward listen address: $forward_listen"

curl_url="http://$forward_listen/"
curl_output="$(wait_for_curl "$curl_url" "$SITE_EXPECTED")"
echo
echo "curl:"
print_command curl --http1.0 -i -fsS "$curl_url"
printf '%s\n' "$curl_output"
require_contains "$curl_output" "$SITE_EXPECTED"

stop_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  service stop
  --machine cloud-aws
  --name web
)
echo
echo "service stop:"
print_command "${stop_cmd[@]}"
stop_output="$("${stop_cmd[@]}")"
printf '%s\n' "$stop_output"
require_contains "$stop_output" "desired state: stopped"
require_contains "$stop_output" "runtime state: stopped"
