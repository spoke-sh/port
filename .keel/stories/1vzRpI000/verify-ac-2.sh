#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

rg -n -F 'restart policy, health checks, scheduler policy, and hardened secret backends remain explicit follow-on work' crates/port-cli/src/lib.rs
rg -n -F 'Secret values are still stored as runtime-owned JSON for the demo lane' README.md
rg -n -F 'restart policy, health checks, scheduler policy, and hardened secret' README.md
rg -n -F 'no restart-policy, scheduler-policy, or hardened secret-backend product' docs/hosted.md
rg -n -F 'single-node demo rather than a hardened multi-node hosted product' docs/hosted.md
rg -n -F 'restart policy, scheduler policy, health checks, and hardened secret' docs/sdk.md
if rg -n -e 'real hosted execution remains follow-on work|real hosted execution and teardown remain follow-on work|no hosted secrets/services/sandboxes execution product exists yet|later live execution will fill in|rather than materializing real hosted execution yet' README.md docs/hosted.md crates/port-cli/src/lib.rs; then
  exit 1
fi
