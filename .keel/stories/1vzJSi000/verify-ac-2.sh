#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine stop --machine demo --runtime-root "$tmpdir/runtime" >/dev/null 2>&1 || true; rm -rf "$tmpdir"' EXIT

nix develop -c cargo test -q -p port-runtime
nix develop -c cargo test -q -p port-cli

nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine launch --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/launch.out" 2>"$tmpdir/launch.err"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine status --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/status.out" 2>"$tmpdir/status.err"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine stop --machine demo --runtime-root "$tmpdir/runtime" >"$tmpdir/stop.out" 2>"$tmpdir/stop.err"

cat "$tmpdir/launch.out"
cat "$tmpdir/status.out"
cat "$tmpdir/stop.out"

rg "launched machine: demo" "$tmpdir/launch.out"
rg "state: running" "$tmpdir/status.out"
rg "current state: stopped" "$tmpdir/stop.out"
