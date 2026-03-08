#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo run -q -p port-sdk --example hosted-sdk =="
cargo run -q -p port-sdk --example hosted-sdk

echo
echo "== hosted contract docs excerpt =="
rg -n 'port-hosted-protocol|route-context|node-agent request paths|auth-header|x-port-audience|x-port-node-agent-token' \
  README.md docs/hosted.md docs/sdk.md crates/port-hosted-protocol/src/lib.rs crates/port-sdk/src/lib.rs
