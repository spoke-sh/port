#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

if rg -n "cargo run" \
  README.md \
  docs \
  CONSTITUTION.md \
  ARCHITECTURE.md \
  CONFIGURATION.md \
  RELEASE.md \
  EVALUATIONS.md \
  crates/port-cli/src/lib.rs
then
  exit 1
fi

rg -n 'port doctor|port --config examples/port.toml' \
  README.md \
  docs/operators.md \
  CONFIGURATION.md \
  crates/port-cli/src/lib.rs
