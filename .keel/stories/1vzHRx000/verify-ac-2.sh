#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'kill "${control_pid:-0}" >/dev/null 2>&1 || true; rm -rf "$tmpdir"' EXIT

config_copy="$tmpdir/port-hosted-pvm.toml"
cp /home/alex/workspace/spoke-sh/port/examples/port.toml "$config_copy"

control_port="$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)"
control_addr="127.0.0.1:${control_port}"

perl -0pi -e 's/(\[control_planes\.demo\][\s\S]*?endpoint = \")([^\"]+)(\")/${1}http:\/\/'"$control_addr"'${3}/' "$config_copy"
perl -0pi -e 's/(\[machines\.cloud-generic\][\s\S]*?protection_mode = \")standard"/${1}pvm"/' "$config_copy"

nix develop -c cargo test -q -p port-cli
nix develop -c cargo test -q -p port-runtime

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$config_copy" control-plane serve --control-plane demo --bind "$control_addr" >"$tmpdir/control-plane.stdout.log" 2>"$tmpdir/control-plane.stderr.log" &
control_pid=$!

for _ in $(seq 1 100); do
  if bash -lc "exec 3<>/dev/tcp/127.0.0.1/${control_port}" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

PORT_DEMO_TOKEN=demo-token nix develop -c cargo run -q -p port-cli -- --config "$config_copy" machine status --machine cloud-generic >"$tmpdir/status.log"
if nix develop -c cargo run -q -p port-cli -- --config "$config_copy" machine launch --machine cloud-generic >"$tmpdir/launch.stdout.log" 2>"$tmpdir/launch.stderr.log"; then
  echo "expected hosted PVM launch to fail before remote guidance" >&2
  exit 1
fi

cat "$tmpdir/status.log"
cat "$tmpdir/launch.stderr.log"
rg "machine: cloud-generic|state: malformed|generic-linux-node|planned|PVM" "$tmpdir/status.log"
rg "cloud-generic|generic-linux-node|planned|PVM" "$tmpdir/launch.stderr.log"
