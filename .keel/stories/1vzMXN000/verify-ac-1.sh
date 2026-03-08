#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-agent-protocol =="
cargo test -q -p port-agent-protocol

echo
echo "== cargo test -q -p port-hosted-protocol =="
cargo test -q -p port-hosted-protocol

echo
echo "== shared stream contract excerpt =="
rg -n 'StreamSessionContract|StreamInputMode|StreamOutputChannel|StreamTerminationMode|HostedGuestStreamRoute|HostedGuestStreamProtocol|guest:.*:stream' \
  crates/port-agent-protocol/src/lib.rs \
  crates/port-hosted-protocol/src/lib.rs
