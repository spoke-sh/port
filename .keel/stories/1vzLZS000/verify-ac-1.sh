#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-model avf
nix develop -c cargo test -q -p port-runtime driver_selection_rejects_avf_lane_without_driver
