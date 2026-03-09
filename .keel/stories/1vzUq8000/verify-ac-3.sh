#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime hosted_fleet_state_reports_merge_failures_with_control_plane_and_node_detail =="
nix develop -c cargo test -q -p port-runtime hosted_fleet_state_reports_merge_failures_with_control_plane_and_node_detail

echo
echo "== hosted inspection output render paths =="
rg -n 'format_machine_status|format_hosted_fleet_nodes|print_hosted_fleet_nodes' \
  crates/port-cli/src/lib.rs
