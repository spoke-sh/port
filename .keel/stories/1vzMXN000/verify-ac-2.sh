#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-sdk =="
cargo test -q -p port-sdk

echo
echo "== cargo test -q -p port-runtime =="
cargo test -q -p port-runtime

echo
echo "== cargo test -q =="
cargo test -q

echo
echo "== cleanup and ownership excerpt =="
rg -n 'HostedApiStreamRequest|PortAgentStreamV1|wait_for_tcp_or_server_error|start_live_hosted_servers|GuestStream|termination' \
  crates/port-sdk/src/lib.rs \
  crates/port-runtime/src/lib.rs \
  crates/port-agent-protocol/src/lib.rs \
  crates/port-hosted-protocol/src/lib.rs
