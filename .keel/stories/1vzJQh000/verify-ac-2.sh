#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime hosted_pvm_launch_rejects_unplaceable_nodes_before_remote_guidance
nix develop -c cargo test -q -p port-cli cli_machine_status_surfaces_hosted_pvm_placement_denial
nix develop -c cargo test -q -p port-cli cli_machine_monitor_reports_hosted_runtime_context
