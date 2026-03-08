#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo run -q -p port-cli -- --help >/tmp/1vz4gb000-help.verify
sed -n '70,120p' /tmp/1vz4gb000-help.verify
grep -F '[control_planes.demo]' /tmp/1vz4gb000-help.verify
grep -F 'PORT_DEMO_TOKEN' /tmp/1vz4gb000-help.verify
grep -nE 'control_planes.demo|PORT_DEMO_TOKEN|hosted-control-plane|port.example.internal' \
  README.md \
  docs/hosted.md \
  docs/operators.md \
  docs/cloud.md \
  examples/port.toml
