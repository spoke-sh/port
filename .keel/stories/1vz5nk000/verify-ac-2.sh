#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port --help hosted guest excerpt =="
cargo run -q -p port-cli -- --help | sed -n '59,69p'

echo
echo "== hosted guest runtime docs excerpt =="
rg -n 'Hosted guest|hosted guest|Detached forwarding|detached and Unix-socket forwarding|SDK clients|runtime_root' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs
