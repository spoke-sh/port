#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

cargo test -q -p port-cli help_includes_primary_surfaces

rg -q "Registered hosted node workflow for the current demo lane" README.md
rg -q "bootstrap or debug override" README.md docs/hosted.md crates/port-cli/src/lib.rs
rg -q "no autoscaling, no broader fleet policy, and no external inventory yet" README.md docs/hosted.md scripts/hosted-demo.sh
rg -q "machine list" README.md docs/hosted.md crates/port-cli/src/lib.rs
