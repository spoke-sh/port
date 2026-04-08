#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${PORT_HOSTED_PVM_WORKDIR:-$(mktemp -d "/tmp/port-pvm.XXXXXX")}"
KEEP_WORKDIR="${PORT_HOSTED_PVM_KEEP:-0}"
NODE_ADDR="${PORT_HOSTED_PVM_NODE_ADDR:-127.0.0.1:9236}"
CONTROL_ADDR="${PORT_HOSTED_PVM_CONTROL_ADDR:-127.0.0.1:7042}"
NODE_TOKEN="${PORT_HOSTED_PVM_NODE_TOKEN:-node-secret}"
export PORT_DEMO_TOKEN="${PORT_DEMO_TOKEN:-demo-token}"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
PORT_BIN="$TARGET_DIR/debug/port"
PORT_GUEST_AGENT_BIN="$TARGET_DIR/debug/port-guest-agent"

SERVER_CONFIG="$WORKDIR/server-port.toml"
CLIENT_CONFIG="$WORKDIR/client-port.toml"
HOSTED_RUNTIME_ROOT="$WORKDIR/hosted/aws-linux-node"
BOGUS_RUNTIME_ROOT="$WORKDIR/bogus/aws-linux-node"
PVM_KERNEL_PATH="$WORKDIR/pvm-vmlinux"
PVM_GUEST_PATH="$WORKDIR/pvm-rootfs.ext4"
PVM_FIRECRACKER="$WORKDIR/firecracker-pvm"
IMPORTED_INVENTORY_PATH="$REPO_ROOT/.port/hosted/demo/imported-inventory.json"

cleanup() {
  local status=$?
  for pid_var in CONTROL_PLANE_PID NODE_AGENT_PID; do
    local pid="${!pid_var:-}"
    if [[ -n "${pid}" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
  rm -rf "$REPO_ROOT/.port/hosted/demo" >/dev/null 2>&1 || true
  if [[ "$KEEP_WORKDIR" != "1" ]]; then
    rm -rf "$WORKDIR"
  fi
  return "$status"
}
trap cleanup EXIT

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
    if "$PORT_BIN" --config "$config" machine list 2>/dev/null | grep -Fq "$machine"; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for hosted machine '$machine' to appear in machine list" >&2
  return 1
}

print_command() {
  printf '$'
  for arg in "$@"; do
    printf ' %q' "$arg"
  done
  printf '\n'
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

cd "$REPO_ROOT"
rm -rf .port/hosted/demo
mkdir -p "$HOSTED_RUNTIME_ROOT" "$BOGUS_RUNTIME_ROOT"
printf 'fake-pvm-kernel' >"$PVM_KERNEL_PATH"
printf 'fake-pvm-rootfs' >"$PVM_GUEST_PATH"
cat >"$PVM_FIRECRACKER" <<'EOF'
#!/usr/bin/env bash
sleep 30
EOF
chmod 0755 "$PVM_FIRECRACKER"
export PORT_PVM_FIRECRACKER_BINARY="$PVM_FIRECRACKER"

cargo build -q -p port --bin port
cargo build -q -p port-guest-agent --bin port-guest-agent

python3 - "$REPO_ROOT/examples/port.toml" "$SERVER_CONFIG" "$CONTROL_ADDR" "$HOSTED_RUNTIME_ROOT" "$PVM_KERNEL_PATH" "$PVM_GUEST_PATH" <<'PY'
from pathlib import Path
import re
import sys

source_path = Path(sys.argv[1])
server_config = Path(sys.argv[2])
control_addr = sys.argv[3]
hosted_runtime_root = sys.argv[4]
kernel_path = sys.argv[5]
guest_path = sys.argv[6]

text = source_path.read_text(encoding="utf-8")
text = text.replace(
    'endpoint = "https://port.example.internal"',
    f'endpoint = "http://{control_addr}"',
    1,
)
text = text.replace(
    'runtime_root = "runtime/hosted/aws-linux-node"',
    f'runtime_root = "{hosted_runtime_root}"',
    1,
)
text = text.replace(
    'path = "artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux"',
    f'path = "{kernel_path}"',
    1,
)
text = text.replace(
    'path = "artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4"',
    f'path = "{guest_path}"',
    1,
)
text = text.replace(
    'console_log = "runtime/cloud-aws/console.log"\n',
    'console_log = "runtime/cloud-aws/console.log"\n\n[machines.cloud-aws.network]\nenabled = false\n',
    1,
)
text = text.replace("requires_custom_host_kernel = true", "requires_custom_host_kernel = false", 2)
text = text.replace('host_boot_args = ["pti=off"]', "host_boot_args = []", 2)
text, count = re.subn(
    r'(?m)^\[clusters\.demo\]$[\s\S]*\Z',
    "",
    text,
    count=1,
)
if count != 1:
    raise SystemExit("failed to strip the local cluster block from the hosted PVM proof config")
text, count = re.subn(
    r'(?m)(^\[machines\.cloud-aws\]$[\s\S]*?^protection_mode = )"standard"$',
    r'\1"pvm"',
    text,
    count=1,
)
if count != 1:
    raise SystemExit("failed to switch machines.cloud-aws to protection_mode = \"pvm\"")
server_config.write_text(text, encoding="utf-8")
PY

python3 - "$SERVER_CONFIG" "$CLIENT_CONFIG" "$HOSTED_RUNTIME_ROOT" "$BOGUS_RUNTIME_ROOT" <<'PY'
from pathlib import Path
import sys

server_config = Path(sys.argv[1])
client_config = Path(sys.argv[2])
hosted_runtime_root = sys.argv[3]
bogus_runtime_root = sys.argv[4]

text = server_config.read_text(encoding="utf-8")
text = text.replace(
    f'runtime_root = "{hosted_runtime_root}"',
    f'runtime_root = "{bogus_runtime_root}"',
    1,
)
client_config.write_text(text, encoding="utf-8")
PY

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
echo "proof lane: hosted aws pvm"
echo "machine: cloud-aws"
echo "node: aws-linux-node"
echo "architecture: x86_64"
echo "provider boundary: aws only"
echo "firecracker binary: $PORT_PVM_FIRECRACKER_BINARY"

prepare_cmd=(
  "$PORT_BIN" --config "$CLIENT_CONFIG"
  control-plane prepare-pvm-node
  --control-plane demo
  --node aws-linux-node
  --architecture x86-64
  --provenance repo-proof
  --package-name firecracker-pvm-host-kit
  --package-version 2026.04
  --host-kernel-release 6.12.0-port-pvm
  --firecracker-build v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c
)
echo
echo "prepare-pvm-node:"
print_command "${prepare_cmd[@]}"
prepare_output="$("${prepare_cmd[@]}")"
printf '%s\n' "$prepare_output"
require_contains "$prepare_output" "prepared hosted pvm node: aws-linux-node"
require_contains "$prepare_output" "firecracker-pvm-host-kit@2026.04"

echo
echo "imported inventory:"
python3 - "$IMPORTED_INVENTORY_PATH" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
record = data["nodes"]["aws-linux-node"]
lane = record["capability_summary"]["pvm_lanes"][0]
package = record["pvm_host_kit_packages"][0]["package"]
print(f"provider: {record['provider']}")
print(f"provenance: {record['provenance']}")
print(f"state: {lane['state']}")
print(
    "package: "
    f"{package['name']}@{package['version']} "
    f"kernel={package['host_kernel_release']} "
    f"build={package['firecracker_build']}"
)
PY

launch_cmd=("$PORT_BIN" --config "$CLIENT_CONFIG" machine launch --machine cloud-aws)
echo
echo "machine launch:"
print_command "${launch_cmd[@]}"
launch_output="$("${launch_cmd[@]}")"
printf '%s\n' "$launch_output"
require_contains "$launch_output" "launched machine: cloud-aws"
require_contains "$launch_output" "$PORT_PVM_FIRECRACKER_BINARY"

status_cmd=("$PORT_BIN" --config "$CLIENT_CONFIG" machine status --machine cloud-aws)
echo
echo "machine status:"
print_command "${status_cmd[@]}"
status_output="$("${status_cmd[@]}")"
printf '%s\n' "$status_output"
require_contains "$status_output" "machine: cloud-aws"
require_contains "$status_output" "state: running"
require_contains "$status_output" "control plane 'demo'"
require_contains "$status_output" "node 'aws-linux-node'"
require_contains "$status_output" "provider 'aws'"

stop_cmd=("$PORT_BIN" --config "$CLIENT_CONFIG" machine stop --machine cloud-aws)
echo
echo "machine stop:"
print_command "${stop_cmd[@]}"
stop_output="$("${stop_cmd[@]}")"
printf '%s\n' "$stop_output"
require_contains "$stop_output" "machine: cloud-aws"
require_contains "$stop_output" "current state: stopped"
require_contains "$stop_output" "control plane 'demo'"
require_contains "$stop_output" "node 'aws-linux-node'"
require_contains "$stop_output" "provider 'aws'"

echo
echo "current hosted aws pvm limits:"
echo "- this proof is x86_64 AWS hosted PVM only"
echo "- the prepared host contract still requires prepare-pvm-node plus the explicit AWS host-kit package"
echo "- generic, GCP, and Azure hosted nodes do not inherit this AWS PVM lane"
echo "- aarch64 firecracker/pvm remains research-only"
