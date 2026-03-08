#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port --help hosted runtime excerpt =="
cargo run -q -p port-cli -- --help | sed -n '59,69p'

echo
echo "== hosted runtime docs excerpt =="
rg -n 'runtime_root|hosted machine list\\|status\\|stop|hosted guest runtime|malformed' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs
