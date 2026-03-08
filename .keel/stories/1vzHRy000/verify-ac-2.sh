#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
control_pid=""
demo_runtime="$tmpdir/runtime"
config_copy="$tmpdir/port-hosted-pvm.toml"

cleanup() {
  if [[ -f "$demo_runtime/demo/firecracker.pid" ]]; then
    nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml machine stop --machine demo --runtime-root "$demo_runtime" >/dev/null 2>&1 || true
  fi
  if [[ -n "$control_pid" ]]; then
    kill "$control_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$tmpdir"
}
trap cleanup EXIT

cp /home/alex/workspace/spoke-sh/port/examples/port.toml "$config_copy"
perl -0pi -e 's/(?m)(^\[machines\.cloud-generic\]\n(?:.*\n)*?^protection_mode = ")standard"/${1}pvm"/' "$config_copy"

control_port="$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)"
control_addr="127.0.0.1:${control_port}"
perl -0pi -e 's/(?m)(^\[control_planes\.demo\]\n(?:.*\n)*?^endpoint = \")([^\"]+)(\")/${1}http:\/\/'"$control_addr"'${3}/' "$config_copy"

nix develop -c cargo run -q -p port-cli -- --config "$config_copy" artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config "$config_copy" artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config "$config_copy" artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config "$config_copy" artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-kernel --architecture native
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-guest --architecture native
nix develop -c cargo run -q -p port-cli -- --config "$config_copy" doctor >"$tmpdir/doctor.log"

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$config_copy" control-plane serve --control-plane demo --bind "$control_addr" >"$tmpdir/control-plane.stdout.log" 2>"$tmpdir/control-plane.stderr.log" &
control_pid=$!

for _ in $(seq 1 100); do
  if bash -lc "exec 3<>/dev/tcp/127.0.0.1/${control_port}" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$config_copy" machine status --machine cloud-generic >"$tmpdir/pvm-status.log"
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml machine launch --machine demo --runtime-root "$demo_runtime" >"$tmpdir/standard-launch.log"
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml machine status --machine demo --runtime-root "$demo_runtime" >"$tmpdir/standard-status.log"
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml machine stop --machine demo --runtime-root "$demo_runtime" >"$tmpdir/standard-stop.log"

cat "$tmpdir/doctor.log"
cat "$tmpdir/pvm-status.log"
cat "$tmpdir/standard-launch.log"
cat "$tmpdir/standard-status.log"
cat "$tmpdir/standard-stop.log"

rg "pvm:local:x86_64:host-platform" "$tmpdir/doctor.log"
rg "pvm:local:x86_64:boot-line" "$tmpdir/doctor.log"
rg "pvm:local:x86_64:firecracker-binary" "$tmpdir/doctor.log"
rg "machine: cloud-generic" "$tmpdir/pvm-status.log"
rg "state: malformed" "$tmpdir/pvm-status.log"
rg "generic-linux-node" "$tmpdir/pvm-status.log"
rg "planned" "$tmpdir/pvm-status.log"
rg "PVM" "$tmpdir/pvm-status.log"
rg "launched machine: demo" "$tmpdir/standard-launch.log"
rg "runtime dir:" "$tmpdir/standard-launch.log"
rg "machine: demo" "$tmpdir/standard-status.log"
rg "inventory owner: local-runtime-root" "$tmpdir/standard-status.log"
rg "machine: demo" "$tmpdir/standard-stop.log"
rg "current state: stopped" "$tmpdir/standard-stop.log"
