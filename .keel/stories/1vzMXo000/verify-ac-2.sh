#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-cli cli_guest_logs_follow_streams_appended_output =="
nix develop -c cargo test -q -p port-cli cli_guest_logs_follow_streams_appended_output

echo
echo "== cargo test -q =="
nix develop -c cargo test -q

echo
echo "== streamed PTY/log follow surface excerpt =="
rg -n 'StreamRequestFrame|StreamResponseFrame|stream_guest_pty|stream_guest_logs|GuestOperation::Pty|GuestOperation::Logs|follow' \
  crates/port-agent-protocol/src/lib.rs \
  crates/port-guest-agent/src/lib.rs \
  crates/port-runtime/src/lib.rs \
  crates/port-cli/src/lib.rs \
  crates/port-cli/tests/guest_commands.rs
