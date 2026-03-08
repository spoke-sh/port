#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
config_copy="$tmpdir/port-pvm.toml"
cp /home/alex/workspace/spoke-sh/port/examples/port.toml "$config_copy"
perl -0pi -e 's/(\[machines\.demo\][\s\S]*?protection_mode = ")standard"/${1}pvm"/' "$config_copy"

nix develop -c cargo test -q -p port-cli
nix develop -c cargo run -q -p port-cli -- --config "$config_copy" doctor >"$tmpdir/doctor.log"
if nix develop -c cargo run -q -p port-cli -- --config "$config_copy" machine launch --machine demo >"$tmpdir/launch.stdout.log" 2>"$tmpdir/launch.stderr.log"; then
  echo "expected launch to fail for an unprepared local PVM host kit" >&2
  exit 1
fi
cat "$tmpdir/doctor.log"
cat "$tmpdir/launch.stderr.log"
rg "pvm:local:x86_64:boot-line|pvm:local:x86_64:firecracker-binary" "$tmpdir/doctor.log"
rg "pvm host-kit preflight failed|pti=off|firecracker-pvm" "$tmpdir/launch.stderr.log"
