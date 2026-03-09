#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

nix develop -c cargo test -q -p port-cli cli_guest_logs_follow_streams_appended_output
nix develop -c cargo test -q -p port-cli cli_guest_commands_cover_hosted_control_plane_runtime
nix develop -c cargo test -q -p port-cli cli_guest_forward_supports_hosted_unix_socket_mode
