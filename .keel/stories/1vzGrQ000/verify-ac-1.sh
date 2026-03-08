#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
nix develop -c cargo run -q -p port-cli -- --help
rg -n "x86_64|aarch64|pti=off|firecracker-pvm|x86_64/firecracker/pvm" README.md docs/pvm.md crates/port-cli/src/lib.rs
