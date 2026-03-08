#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime avf_launch_fails_fast_on_non_macos_hosts
nix develop -c cargo test -q -p port-runtime
nix develop -c cargo test -q -p port-cli
