#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port service --help excerpt =="
cargo run -q -p port-cli -- service --help | sed -n '1,200p'

echo
echo "== hosted service docs excerpt =="
rg -n 'port service|service secret|kind sandbox|runtime-owned JSON|service_route|SDK/API clients|follow-on work' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs
