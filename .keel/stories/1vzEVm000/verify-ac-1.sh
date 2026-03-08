#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime control_plane =="
cargo test -q -p port-runtime control_plane

echo
echo "== cargo test -q -p port-cli =="
cargo test -q -p port-cli
