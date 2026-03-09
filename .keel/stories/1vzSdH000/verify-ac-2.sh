#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

HELP_OUTPUT="$(nix develop -c cargo run -q -p port-cli -- --help)"

grep -F 'Current hosted service limits:' <<<"$HELP_OUTPUT"
grep -F 'No autoscaling or rescheduling yet.' <<<"$HELP_OUTPUT"
grep -F 'Deterministic-first-fit is the only shipped scheduler policy.' <<<"$HELP_OUTPUT"
grep -F 'No fleet manager, durable node registration, or broader service orchestration yet.' <<<"$HELP_OUTPUT"

rg -n '^Current hosted service limits:$' README.md docs/hosted.md
rg -n 'No autoscaling or rescheduling yet\.' README.md docs/hosted.md
rg -n 'Deterministic-first-fit is the only shipped scheduler policy\.' README.md docs/hosted.md
rg -n 'No fleet manager, durable node registration, or broader service orchestration' README.md docs/hosted.md
