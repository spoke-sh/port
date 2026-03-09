#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

rg -n -F 'Hosted `port service secret` and `port service apply|list|status|stop` now execute through the live control-plane and node-agent path' crates/port-cli/src/lib.rs
rg -n -F 'Apply a service or sandbox definition through the resolved runtime owner' crates/port-cli/src/lib.rs
rg -n -F 'List service and sandbox definitions plus runtime state for a machine' crates/port-cli/src/lib.rs
rg -n -F 'Inspect one service or sandbox definition and runtime state' crates/port-cli/src/lib.rs
rg -n -F 'Stop one service or sandbox through the resolved runtime owner' crates/port-cli/src/lib.rs
rg -n -F 'hosted demo lane executes them through' README.md
rg -n -F 'executes the resulting managed process through the live control-plane and' README.md
rg -n -F 'node-owned runtime record path' README.md
rg -n -F 'managed process through that same hosted route' docs/hosted.md
rg -n -F 'those service calls execute through the same live hosted control-plane' docs/sdk.md
