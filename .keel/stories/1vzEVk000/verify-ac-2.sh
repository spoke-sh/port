#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime node_agent_reports_missing_guest_socket_with_runtime_context =="
cargo test -q -p port-runtime node_agent_reports_missing_guest_socket_with_runtime_context

echo
echo "== cargo run -q -p port-cli -- node-agent serve --help =="
cargo run -q -p port-cli -- node-agent serve --help

echo
echo "== node-agent docs and help excerpt =="
rg -n 'node-agent serve|x-port-node-agent-token|runtime root|hosted node-runtime server' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs crates/port-runtime/src/hosted_control_plane.rs
