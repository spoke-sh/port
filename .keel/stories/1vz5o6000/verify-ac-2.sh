#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli

echo
echo "== port guest forward --help excerpt =="
cargo run -q -p port-cli -- guest forward --help | sed -n '1,160p'

echo
echo "== detached and Unix-socket docs excerpt =="
rg -n 'unix:/|detached|--list|--stop|hosted guest runtime path|monitoring and `top`|services/sandboxes|SDK' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs
