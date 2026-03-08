#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime node_agent =="
cargo test -q -p port-runtime node_agent

echo
echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli
