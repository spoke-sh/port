#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-agent-protocol managed_service_operations_round_trip_through_json
nix develop -c cargo test -q -p port-guest-agent service_handles_exec_pty_and_logs
nix develop -c cargo test -q -p port-guest-agent connection_streams_copy_and_forward
nix develop -c cargo test -q -p port-guest-agent service_manages_process_lifecycle_and_redacts_secret_output
