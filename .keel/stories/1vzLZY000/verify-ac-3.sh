#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo run -q -p port-cli -- --help | rg -n "Firecracker launch stays Linux-only"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml doctor | rg -n "Firecracker and Firecracker/PVM remain separate Linux lanes"
bash -lc 'set +e; output=$(nix develop -c cargo run -q -p port-cli -- --config examples/port.toml machine launch --machine demo-avf 2>&1); status=$?; set -e; test "$status" -ne 0; printf "%s\n" "$output" | rg -n "AVF local launch requires running Port on macOS"'
