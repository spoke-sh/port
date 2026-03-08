#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port --help hosted guest attach excerpt =="
cargo run -q -p port-cli -- --help | rg -n 'Hosted guest attach|In the MVP, those guest verbs still run only through the local runtime path'

echo
echo "== hosted guest attach docs excerpt =="
rg -n 'Hosted guest `exec`, `copy`, `pty`, `logs`, and `forward`|Hosted guest attach contract|What still remains after this contract|modeled today, not runnable yet' README.md docs/hosted.md crates/port-cli/src/lib.rs
