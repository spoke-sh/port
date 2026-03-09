#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

cargo test -q -p port-cli tests::help_includes_primary_surfaces -- --exact

grep -n "Hosted standard cloud workflow" crates/port-cli/src/lib.rs
grep -n "Repository-local hosted standard proof" crates/port-cli/src/lib.rs
grep -n "cloud-generic" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs
grep -n "cloud-aws" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs
grep -n "cloud-gcp" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs
