#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime list_machines_includes_hosted_control_plane_statuses =="
nix develop -c cargo test -q -p port-runtime list_machines_includes_hosted_control_plane_statuses

echo
echo "== cargo test -q -p port-cli hosted_fleet_render_includes_node_state_breakdown =="
nix develop -c cargo test -q -p port-cli hosted_fleet_render_includes_node_state_breakdown
