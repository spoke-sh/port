#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port --help hosted lifecycle excerpt =="
cargo run -q -p port-cli -- --help | rg -n 'Hosted `machine list`, `status`, and `stop`|In the MVP, those verbs still run only against the local runtime'

echo
echo "== hosted lifecycle docs excerpt =="
rg -n 'Hosted Machine Lifecycle Surface|Hosted `machine list`, `status`, and `stop`|modeled today, not runnable yet|current hosted lifecycle contract' README.md docs/hosted.md crates/port-cli/src/lib.rs
