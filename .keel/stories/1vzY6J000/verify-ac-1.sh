#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

cargo test -q -p port-cli tests::help_includes_machine_commands_examples -- --exact
grep -nE "host kit|research-only|aarch64|prepare-pvm-node" \
  README.md docs/pvm.md docs/hosted.md crates/port-cli/src/lib.rs
