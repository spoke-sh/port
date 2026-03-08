#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime avf_guest_exec_pty_and_logs_use_runtime_socket_after_launch
nix develop -c cargo test -q -p port-runtime avf_copy_and_forward_use_runtime_socket_after_launch
