#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

rm -rf .port crates/port-runtime/.port crates/port-cli/.port

nix develop -c sh -lc '
  cargo test -q -p port-runtime hosted_machine_status_prefers_stored_placement_over_live_candidate_selection &&
  cargo test -q -p port-cli --test machine_commands cli_machine_status_prefers_stored_hosted_placement_over_live_candidate
'
