#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
server_pid=""
control_pid=""
trap 'if [[ -n "$server_pid" ]]; then kill "$server_pid" >/dev/null 2>&1 || true; wait "$server_pid" >/dev/null 2>&1 || true; fi; if [[ -n "$control_pid" ]]; then kill "$control_pid" >/dev/null 2>&1 || true; wait "$control_pid" >/dev/null 2>&1 || true; fi; nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/client.toml" machine stop --machine cloud-aws >/dev/null 2>&1 || true; nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine stop --machine demo --runtime-root "$tmpdir/runtime" >/dev/null 2>&1 || true; rm -rf "$tmpdir"' EXIT

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
import socket, sys
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

node_addr="$(reserve_addr)"
control_addr="$(reserve_addr)"
hosted_runtime_root="$tmpdir/hosted/aws-linux-node"
fake_binary="$tmpdir/firecracker-pvm"
kernel_path="$tmpdir/pvm-vmlinux"
guest_path="$tmpdir/pvm-rootfs.ext4"

printf '#!/usr/bin/env bash\nsleep 30\n' >"$fake_binary"
chmod 755 "$fake_binary"
printf 'fake-kernel' >"$kernel_path"
printf 'fake-rootfs' >"$guest_path"

cp examples/port.toml "$tmpdir/server.toml"
python3 - "$tmpdir/server.toml" "$kernel_path" "$guest_path" "$hosted_runtime_root" "$control_addr" <<'PY'
from pathlib import Path
import sys

config_path, kernel_path, guest_path, hosted_runtime_root, control_addr = sys.argv[1:]
text = Path(config_path).read_text()
replacements = [
    ('endpoint = "https://port.example.internal"', f'endpoint = "http://{control_addr}"'),
    ('path = "artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux"', f'path = "{kernel_path}"'),
    ('path = "artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4"', f'path = "{guest_path}"'),
    ('runtime_root = "runtime/hosted/aws-linux-node"', f'runtime_root = "{hosted_runtime_root}"'),
    ('requires_custom_host_kernel = true', 'requires_custom_host_kernel = false'),
    ('host_boot_args = ["pti=off"]', 'host_boot_args = []'),
]
for old, new in replacements:
    text = text.replace(old, new, 1 if old.startswith('runtime_root = "runtime/hosted/aws-linux-node"') else text.count(old))
text = text.replace('protection_mode = "standard"\narchitecture = "native"\nvcpu_count = 2\nmemory_mib = 512\nkernel_args = "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"\nrootfs_read_only = false\n\n[machines.cloud-aws.guest]', 'protection_mode = "pvm"\narchitecture = "native"\nvcpu_count = 2\nmemory_mib = 512\nkernel_args = "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"\nrootfs_read_only = false\n\n[machines.cloud-aws.guest]', 1)
Path(config_path).write_text(text)
PY

cp "$tmpdir/server.toml" "$tmpdir/client.toml"

PORT_PVM_FIRECRACKER_BINARY="$fake_binary" nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/server.toml" node-agent serve --node aws-linux-node --bind "$node_addr" --token node-secret >"$tmpdir/node.out" 2>"$tmpdir/node.err" &
server_pid="$!"
wait_for_tcp "$node_addr"

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/server.toml" control-plane serve --control-plane demo --bind "$control_addr" --node-binding "aws-linux-node=http://$node_addr,node-secret" >"$tmpdir/control.out" 2>"$tmpdir/control.err" &
control_pid="$!"
wait_for_tcp "$control_addr"

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/client.toml" machine launch --machine cloud-aws >"$tmpdir/pvm-launch.out" 2>"$tmpdir/pvm-launch.err"
PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/client.toml" machine status --machine cloud-aws >"$tmpdir/pvm-status.out" 2>"$tmpdir/pvm-status.err"
PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$tmpdir/client.toml" machine stop --machine cloud-aws >"$tmpdir/pvm-stop.out" 2>"$tmpdir/pvm-stop.err"

nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine launch --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/std-launch.out" 2>"$tmpdir/std-launch.err"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine status --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/std-status.out" 2>"$tmpdir/std-status.err"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine stop --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/std-stop.out" 2>"$tmpdir/std-stop.err"

cat "$tmpdir/pvm-launch.out"
cat "$tmpdir/pvm-status.out"
cat "$tmpdir/pvm-stop.out"
cat "$tmpdir/std-launch.out"
cat "$tmpdir/std-status.out"
cat "$tmpdir/std-stop.out"

rg "launched machine: cloud-aws" "$tmpdir/pvm-launch.out"
rg "firecracker binary: .*firecracker-pvm" "$tmpdir/pvm-launch.out"
rg "state: running" "$tmpdir/pvm-status.out"
rg "current state: stopped" "$tmpdir/pvm-stop.out"

rg "launched machine: demo" "$tmpdir/std-launch.out"
rg "state: running" "$tmpdir/std-status.out"
rg "current state: stopped" "$tmpdir/std-stop.out"
