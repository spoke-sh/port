#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-guest-agent connection_streams_pty_and_followed_logs =="
nix develop -c cargo test -q -p port-guest-agent connection_streams_pty_and_followed_logs

echo
echo "== cargo test -q -p port-runtime guest_streaming_operations_aggregate_pty_and_followed_logs_from_runtime_socket =="
nix develop -c cargo test -q -p port-runtime guest_streaming_operations_aggregate_pty_and_followed_logs_from_runtime_socket

echo
echo "== cargo test -q -p port-runtime avf_guest_exec_pty_and_logs_use_runtime_socket_after_launch =="
nix develop -c cargo test -q -p port-runtime avf_guest_exec_pty_and_logs_use_runtime_socket_after_launch

echo
echo "== cargo test -q -p port-cli cli_guest_commands_cover_all_capabilities =="
nix develop -c cargo test -q -p port-cli cli_guest_commands_cover_all_capabilities
