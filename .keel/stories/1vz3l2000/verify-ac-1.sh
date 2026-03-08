#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model

grep -nE 'AvfExecutionContract|AvfGuestTransport|AvfConsoleTransport|AvfDirectoryShareContract|VirtioSocket|SerialPort' \
  crates/port-model/src/lib.rs

grep -nE 'Runtime Contract|virtio socket|serial ports|Launch Ownership|Guest Transport Mapping' \
  docs/avf.md
