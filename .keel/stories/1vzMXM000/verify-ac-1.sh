#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

nix develop -c cargo test -q -p port-cli help_includes_primary_surfaces
rg -n "guest pty --machine demo" crates/port-cli/src/lib.rs README.md
rg -n "logs --follow" crates/port-cli/src/lib.rs README.md docs/hosted.md
rg -n "logs_stream\\(\\)|forward_stream\\(\\)" docs/sdk.md
rg -n "node-owned listener" crates/port-cli/src/lib.rs README.md docs/hosted.md
rg -n "Hosted detached lifecycle management" README.md docs/hosted.md docs/sdk.md
