#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-model =="
cargo test -q -p port-model

echo
echo "== cargo test -q -p port-runtime =="
cargo test -q -p port-runtime

echo
echo "== cargo test -q -p port-cli --test service_commands =="
cargo test -q -p port-cli --test service_commands
