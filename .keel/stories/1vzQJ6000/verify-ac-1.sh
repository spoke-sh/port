#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c sh -lc 'cargo build -q -p port-cli --bin port && cargo test -q -p port-runtime hosted_detached_forward_start_returns_node_owned_manifest'
