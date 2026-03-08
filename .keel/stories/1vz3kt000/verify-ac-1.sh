#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model
cargo test -q -p port-runtime
cargo test -q -p port-cli

cargo run -q -p port-cli -- machine list --runtime-root runtime >/tmp/1vz3kt000-machine-list.verify
sed -n '1,10p' /tmp/1vz3kt000-machine-list.verify

grep -nE 'machine_control_contract|MachineControlContract|control-contract' \
  crates/port-model/src/lib.rs \
  crates/port-runtime/src/lib.rs \
  crates/port-cli/src/lib.rs \
  README.md \
  docs/hosted.md \
  docs/cloud.md \
  docs/operators.md
