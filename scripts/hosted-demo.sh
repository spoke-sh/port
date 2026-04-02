#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${PORT_HOSTED_DEMO_WORKDIR:-$(mktemp -d "/tmp/porthd.XXXXXX")}"
KEEP_WORKDIR="${PORT_HOSTED_DEMO_KEEP:-0}"
NODE_ADDR="${PORT_HOSTED_NODE_ADDR:-127.0.0.1:9234}"
CONTROL_ADDR="${PORT_HOSTED_CONTROL_ADDR:-127.0.0.1:7040}"
NODE_TOKEN="${PORT_HOSTED_NODE_TOKEN:-node-secret}"
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
HOST_SOURCE="$WORKDIR/host.txt"
ROUNDTRIP_PATH="$WORKDIR/roundtrip.txt"
DETACHED_LISTEN_PATH="$WORKDIR/hosted-detached.sock"

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
    if "$PORT_BIN" --config "$config" machine list | grep -q "$machine"; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for hosted machine '$machine' to appear in machine list" >&2
  return 1
}

mkdir -p "$GUEST_ROOT/var/log" "$MACHINE_DIR" "$BOGUS_RUNTIME_ROOT"
printf 'first\nsecond\n' >"$GUEST_ROOT/var/log/app.log"
printf 'copy-ok' >"$HOST_SOURCE"

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
  "pid": 424242,
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
echo
echo "machine list:"
("$PORT_BIN" --config "$CLIENT_CONFIG" machine list)
echo
echo "machine status:"
("$PORT_BIN" --config "$CLIENT_CONFIG" machine status --machine cloud-aws)
echo
echo "guest exec:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest exec --machine cloud-aws -- /bin/echo hosted-http-ok)
echo
echo "guest copy host-to-guest:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest copy --machine cloud-aws --direction host-to-guest --source "$HOST_SOURCE" --destination /workspace/copied.txt)
echo
echo "guest copy guest-to-host:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest copy --machine cloud-aws --direction guest-to-host --source /workspace/copied.txt --destination "$ROUNDTRIP_PATH")
echo
echo "guest logs:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest logs --machine cloud-aws --path /var/log/app.log --tail-lines 1)
echo
echo "guest forward detached start:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest forward --machine cloud-aws --listen "unix:$DETACHED_LISTEN_PATH" --target "unix:$SOCKET_PATH" --lifecycle detached --name demo-sock)
echo
echo "guest forward detached list:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest forward --machine cloud-aws --list)
echo
echo "guest forward detached stop:"
("$PORT_BIN" --config "$CLIENT_CONFIG" guest forward --machine cloud-aws --stop --name demo-sock)
echo
echo "roundtrip file:"
cat "$ROUNDTRIP_PATH"
echo
echo "current hosted demo limits:"
echo "- hosted guest forward detached lifecycle now ships through start/list/stop/name on the live control-plane path"
echo "- no autoscaling, no broader fleet policy, and no external inventory yet"
echo "- retries, richer client policies, and advanced auth/tenancy remain follow-on work"
