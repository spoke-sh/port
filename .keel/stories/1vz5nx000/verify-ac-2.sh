#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port machine --help excerpt =="
cargo run -q -p port-cli -- machine --help | sed -n '1,160p'

echo
echo "== hosted monitoring docs excerpt =="
rg -n 'machine monitor|machine top|detached forward|secrets/services/sandboxes|SDK/API|hosted-control-plane|node-agent runtime' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs
