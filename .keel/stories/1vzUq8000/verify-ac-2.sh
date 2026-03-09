#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime hosted_fleet_state_surfaces_live_stale_and_imported_only_nodes =="
nix develop -c cargo test -q -p port-runtime hosted_fleet_state_surfaces_live_stale_and_imported_only_nodes

echo
echo "== cargo test -q -p port-cli hosted_fleet_render_distinguishes_live_stale_and_missing_nodes =="
nix develop -c cargo test -q -p port-cli hosted_fleet_render_distinguishes_live_stale_and_missing_nodes
