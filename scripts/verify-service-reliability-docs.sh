#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

rg -n -F "restart" README.md docs examples/port.toml crates/port-cli/src/lib.rs
rg -n -F "health" README.md docs examples/port.toml crates/port-cli/src/lib.rs
rg -n -F "service secret" README.md docs examples/port.toml crates/port-cli/src/lib.rs
rg -n -F "service apply" README.md docs examples/port.toml crates/port-cli/src/lib.rs
rg -n -F "service status" README.md docs examples/port.toml crates/port-cli/src/lib.rs
