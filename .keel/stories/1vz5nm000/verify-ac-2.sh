#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo run -q -p port-sdk --example hosted-sdk =="
cargo run -q -p port-sdk --example hosted-sdk

echo
echo "== hosted SDK docs excerpt =="
rg -n 'port-sdk|request-builder|HostedClient::from_machine|machines\\(\\)|guest\\(\\)|services\\(\\)|planned|transport' \
  README.md docs/hosted.md docs/sdk.md crates/port-sdk/src/lib.rs
