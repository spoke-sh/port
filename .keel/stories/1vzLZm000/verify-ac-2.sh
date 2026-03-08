#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime avf_launch_status_and_stop_write_canonical_runtime_state
nix develop -c cargo test -q -p port-runtime
nix develop -c cargo test -q -p port-cli
