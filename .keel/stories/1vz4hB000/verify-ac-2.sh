#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo run -q -p port-cli -- --help >/tmp/1vz4hB000-help.verify
sed -n '88,118p' /tmp/1vz4hB000-help.verify
grep -F '[nodes.<name>]' /tmp/1vz4hB000-help.verify
grep -F '[host_groups.<name>]' /tmp/1vz4hB000-help.verify
grep -nE 'Node And Host-Group Inventory Contract|scheduler|monitoring|services|explicit membership' \
  docs/hosted.md \
  README.md
