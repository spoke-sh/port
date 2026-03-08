#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model
cargo test -q -p port-runtime
cargo test -q -p port-cli

cargo run -q -p port-cli -- --config examples/port.toml doctor >/tmp/1vz4gb000-doctor.verify
sed -n '1,25p' /tmp/1vz4gb000-doctor.verify

set +e
cargo run -q -p port-cli -- --config examples/port.toml machine launch --machine cloud-aws \
  >/tmp/1vz4gb000-cloud-launch.verify 2>&1
status=$?
set -e

test "$status" -ne 0
sed -n '1,20p' /tmp/1vz4gb000-cloud-launch.verify
grep -F "Hosted routing is modeled through control plane 'demo'" /tmp/1vz4gb000-cloud-launch.verify
grep -nE 'HostedControlPlaneSpec|HostedAuthTokenContract|HostedApiIdentityContract|hosted_api_identity_contract' \
  crates/port-model/src/lib.rs \
  crates/port-runtime/src/lib.rs
