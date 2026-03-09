#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

nix develop -c cargo test -q -p port-runtime copy_guest_file_uses_firecracker_vsock_tunnel_in_both_directions
nix develop -c cargo test -q -p port-runtime guest_forward_session_proxies_through_firecracker_vsock_tunnel
nix develop -c cargo test -q -p port-runtime hosted_pvm_launch_routes_through_live_control_plane_and_prepared_node
nix develop -c cargo test -q -p port-runtime avf_guest_exec_pty_and_logs_use_runtime_socket_after_launch
nix develop -c cargo test -q -p port-runtime avf_copy_and_forward_use_runtime_socket_after_launch
