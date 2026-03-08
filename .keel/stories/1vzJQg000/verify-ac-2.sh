#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

nix develop -c cargo test -q -p port-runtime launch_rejects_malformed_pvm_host_kit_contract_with_explicit_detail

config_copy="$tmpdir/port-malformed-pvm.toml"
cp /home/alex/workspace/spoke-sh/port/examples/port.toml "$config_copy"

perl -0pi -e 's/(firecracker_binary_name = )"firecracker-pvm"/${1}""/' "$config_copy"
perl -0pi -e 's/(\[machines\.demo\][\s\S]*?protection_mode = )"standard"/${1}"pvm"/' "$config_copy"

if nix develop -c cargo run -q -p port-cli -- --config "$config_copy" machine launch --machine demo >"$tmpdir/launch.stdout.log" 2>"$tmpdir/launch.stderr.log"; then
  echo "expected malformed host-kit launch to fail" >&2
  exit 1
fi

cat "$tmpdir/launch.stderr.log"
rg "invalid Port config" "$tmpdir/launch.stderr.log"
rg "host-kit contract" "$tmpdir/launch.stderr.log"
rg "firecracker binary" "$tmpdir/launch.stderr.log"
