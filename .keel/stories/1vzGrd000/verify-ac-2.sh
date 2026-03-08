#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
nix develop -c cargo test -q -p port-runtime doctor_report_fails_fast_for_missing_pvm_boot_arg_and_binary -- --nocapture
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml doctor
