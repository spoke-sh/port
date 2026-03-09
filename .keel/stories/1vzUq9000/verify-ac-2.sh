#!/usr/bin/env bash
set -euo pipefail

repo=/home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
control_pid=""
node_pid=""

cleanup() {
  if [[ -n "$node_pid" ]]; then
    kill "$node_pid" >/dev/null 2>&1 || true
    wait "$node_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$control_pid" ]]; then
    kill "$control_pid" >/dev/null 2>&1 || true
    wait "$control_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$tmpdir"
}
trap cleanup EXIT

reserve_addr() {
  python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(f"{s.getsockname()[0]}:{s.getsockname()[1]}")
s.close()
PY
}

wait_for_tcp() {
  local addr="$1"
  for _ in $(seq 1 100); do
    if python3 - "$addr" <<'PY'
import socket
import sys

host, port = sys.argv[1].rsplit(":", 1)
sock = socket.socket()
sock.settimeout(0.1)
try:
    sock.connect((host, int(port)))
except OSError:
    raise SystemExit(1)
finally:
    sock.close()
PY
    then
      return 0
    fi
    sleep 0.1
  done
  echo "timed out waiting for $addr" >&2
  exit 1
}

wait_for_status() {
  local config_path="$1"
  local output_path="$2"
  local error_path="$3"
  for _ in $(seq 1 120); do
    if PORT_DEMO_TOKEN=demo-token cargo run -q --manifest-path "$repo/Cargo.toml" -p port-cli -- --config "$config_path" machine status --machine cloud-aws >"$output_path" 2>"$error_path"; then
      if rg -q "node: aws-linux-node" "$output_path" \
        && rg -q "imported: true" "$output_path" \
        && rg -q "registered: true" "$output_path" \
        && rg -q "freshness: live" "$output_path" \
        && rg -q "routing eligibility: eligible" "$output_path"; then
        return 0
      fi
    fi
    sleep 0.25
  done

  echo "timed out waiting for durable hosted fleet status" >&2
  [[ -f "$error_path" ]] && cat "$error_path" >&2 || true
  [[ -f "$output_path" ]] && cat "$output_path" >&2 || true
  exit 1
}

config_path="$tmpdir/port.toml"
cp "$repo/examples/port.toml" "$config_path"

control_addr="$(reserve_addr)"
node_addr="$(reserve_addr)"
runtime_root="$tmpdir/runtime/hosted/aws-linux-node"

python3 - "$config_path" "$control_addr" "$runtime_root" <<'PY'
from pathlib import Path
import sys

config_path, control_addr, runtime_root = sys.argv[1:4]
text = Path(config_path).read_text()
text = text.replace(
    'endpoint = "https://port.example.internal"',
    f'endpoint = "http://{control_addr}"',
    1,
)
text = text.replace(
    'runtime_root = "runtime/hosted/aws-linux-node"',
    f'runtime_root = "{runtime_root}"',
    1,
)
Path(config_path).write_text(text)
PY

mkdir -p "$tmpdir/.port/hosted/demo"
python3 - "$config_path" "$tmpdir/.port/hosted/demo/imported-inventory.json" <<'PY'
import json
from pathlib import Path
import sys
import tomllib

config = tomllib.loads(Path(sys.argv[1]).read_text())
capabilities = config["nodes"]["aws-linux-node"]["capabilities"]
state = {
    "control_plane": "demo",
    "nodes": {
        "aws-linux-node": {
            "provider": "aws",
            "provenance": "inventory-sync",
            "imported_at": 123,
            "capability_summary": capabilities,
        }
    },
}
Path(sys.argv[2]).write_text(json.dumps(state, indent=2))
PY

cd "$tmpdir"

PORT_DEMO_TOKEN=demo-token cargo run -q --manifest-path "$repo/Cargo.toml" -p port-cli -- --config "$config_path" control-plane serve --control-plane demo --bind "$control_addr" >"$tmpdir/control.out" 2>"$tmpdir/control.err" &
control_pid="$!"
wait_for_tcp "$control_addr"

PORT_DEMO_TOKEN=demo-token cargo run -q --manifest-path "$repo/Cargo.toml" -p port-cli -- --config "$config_path" node-agent serve --node aws-linux-node --bind "$node_addr" --token node-secret >"$tmpdir/node.out" 2>"$tmpdir/node.err" &
node_pid="$!"
wait_for_tcp "$node_addr"

status_before="$tmpdir/status-before.log"
status_before_err="$tmpdir/status-before.err"
wait_for_status "$config_path" "$status_before" "$status_before_err"

kill "$control_pid" >/dev/null 2>&1 || true
wait "$control_pid" >/dev/null 2>&1 || true
control_pid=""

PORT_DEMO_TOKEN=demo-token cargo run -q --manifest-path "$repo/Cargo.toml" -p port-cli -- --config "$config_path" control-plane serve --control-plane demo --bind "$control_addr" >"$tmpdir/control-restart.out" 2>"$tmpdir/control-restart.err" &
control_pid="$!"
wait_for_tcp "$control_addr"

status_after="$tmpdir/status-after.log"
status_after_err="$tmpdir/status-after.err"
wait_for_status "$config_path" "$status_after" "$status_after_err"

echo "== status before control-plane restart =="
cat "$status_before"
echo
echo "== status after control-plane restart =="
cat "$status_after"

rg -n "machine: cloud-aws|inventory scope: hosted-fleet|node: aws-linux-node|imported: true|registered: true|freshness: live|routing eligibility: eligible" "$status_before"
rg -n "machine: cloud-aws|inventory scope: hosted-fleet|node: aws-linux-node|imported: true|registered: true|freshness: live|routing eligibility: eligible" "$status_after"
