#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-model =="
cargo test -q -p port-model

echo
echo "== cargo test -q -p port-runtime =="
cargo test -q -p port-runtime

echo
echo "== hosted machine list =="
cargo run -q -p port-cli -- --config examples/port.toml machine list | sed -n '1,160p'

echo
echo "== hosted machine status cloud-aws =="
cargo run -q -p port-cli -- --config examples/port.toml machine status --machine cloud-aws | sed -n '1,120p'

echo
echo "== hosted machine stop cloud-aws =="
cargo run -q -p port-cli -- --config examples/port.toml machine stop --machine cloud-aws --wait-secs 1 | sed -n '1,120p'
