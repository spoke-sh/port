#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo run -q -p port-cli -- --help >/tmp/1vz3l2000-help.verify

grep -nE 'docs/avf.md|virtio sockets|serial ports|Rosetta|entitlement' \
  /tmp/1vz3l2000-help.verify \
  crates/port-cli/src/lib.rs \
  README.md \
  docs/operators.md \
  docs/cloud.md \
  docs/avf.md
