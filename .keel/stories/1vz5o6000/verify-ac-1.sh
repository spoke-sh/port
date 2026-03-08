#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-agent-protocol =="
cargo test -q -p port-agent-protocol

echo
echo "== cargo test -q -p port-guest-agent =="
cargo test -q -p port-guest-agent

echo
echo "== cargo test -q -p port-runtime =="
cargo test -q -p port-runtime

echo
echo "== cargo test -q -p port-cli --test guest_commands cli_guest_forward_supports_hosted_unix_socket_mode =="
cargo test -q -p port-cli --test guest_commands cli_guest_forward_supports_hosted_unix_socket_mode

echo
echo "== cargo test -q -p port-cli --test guest_commands cli_guest_forward_supports_hosted_detached_lifecycle =="
cargo test -q -p port-cli --test guest_commands cli_guest_forward_supports_hosted_detached_lifecycle
