#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model
cargo run -q -p port-cli -- --config examples/port.toml doctor >/tmp/1vz4hB000-doctor.verify
sed -n '1,25p' /tmp/1vz4hB000-doctor.verify
grep -nE 'HostedNodeSpec|HostedHostGroupSpec|HostedInventoryContract|hosted_inventory_contract|HostedPlacementPolicy' \
  crates/port-model/src/lib.rs
grep -nE '\[nodes\.aws-linux-node\]|\[host_groups\.remote-linux\]' examples/port.toml
