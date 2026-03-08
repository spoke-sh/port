#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime control_plane_proxies_machine_and_guest_routes_to_node_agent
nix develop -c cargo test -q -p port-runtime hosted_pvm_launch_routes_through_live_control_plane_and_prepared_node
nix develop -c cargo test -q -p port-cli cli_machine_launch_routes_hosted_pvm_through_live_control_plane
