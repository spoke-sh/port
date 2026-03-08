#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-model sample_config_models_all_remote_provider_lanes
nix develop -c cargo test -q -p port-model checked_in_example_models_all_provider_variants
nix develop -c cargo test -q -p port-cli cli_help_mentions_native_avf_workflow_and_boundaries
rg -n "demo-avf|PORT_AVF_LAUNCHER|AVF/PVM|Firecracker launch stays Linux-only" README.md docs/avf.md docs/operators.md
