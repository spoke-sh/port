#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-runtime =="
cargo test -q -p port-runtime

echo
echo "== cargo test -q -p port-cli --test guest_commands cli_guest_commands_cover_hosted_control_plane_runtime =="
cargo test -q -p port-cli --test guest_commands cli_guest_commands_cover_hosted_control_plane_runtime
