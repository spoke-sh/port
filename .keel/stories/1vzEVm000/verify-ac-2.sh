#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime control_plane_rejects_invalid_client_token =="
cargo test -q -p port-runtime control_plane_rejects_invalid_client_token

echo
echo "== cargo test -q -p port-runtime control_plane_reports_missing_node_binding_with_route_context =="
cargo test -q -p port-runtime control_plane_reports_missing_node_binding_with_route_context

echo
echo "== control-plane docs and help excerpt =="
cargo run -q -p port-cli -- control-plane serve --help

echo
rg -n 'control-plane serve|node-binding|first live hosted HTTP server|authorization' \
  README.md docs/hosted.md crates/port-cli/src/lib.rs crates/port-runtime/src/hosted_control_plane.rs
