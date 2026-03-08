#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

HELP_OUTPUT="$(nix develop -c cargo run -q -p port-cli -- --help)"

grep -F 'Hosted demo flow:' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml guest exec --machine cloud-aws' <<<"$HELP_OUTPUT"
grep -F 'Hosted `guest copy` in the single-node demo still assumes' <<<"$HELP_OUTPUT"
grep -F 'Hosted `guest forward` still uses the repo-local guest transport lane' <<<"$HELP_OUTPUT"

rg -n 'bash scripts/hosted-demo.sh' README.md docs/hosted.md
rg -n 'port-guest-agent' README.md docs/hosted.md
rg -n 'single-node demo.*copy|hosted `copy` still assumes node-visible' README.md docs/hosted.md
rg -n 'guest forward.*repo-local guest transport|repo-local guest transport path' README.md docs/hosted.md
