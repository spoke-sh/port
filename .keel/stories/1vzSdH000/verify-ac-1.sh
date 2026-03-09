#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

HELP_OUTPUT="$(nix develop -c cargo run -q -p port-cli -- --help)"

grep -F 'Multi-node hosted service workflow:' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node' <<<"$HELP_OUTPUT"
grep -F -- '--bind 127.0.0.1:9234 --token node-secret' <<<"$HELP_OUTPUT"
grep -F 'aws-linux-node-b --bind 127.0.0.1:9235 --token node-secret-b' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane' <<<"$HELP_OUTPUT"
grep -F 'demo --bind 127.0.0.1:7040 --node-binding aws-linux-node=http://127.0.0.1:9234,node-secret' <<<"$HELP_OUTPUT"
grep -F -- '--node-binding aws-linux-node-b=http://127.0.0.1:9235,node-secret-b' <<<"$HELP_OUTPUT"
grep -F -- '--host-group aws-secondary --name api --kind service --secret API_TOKEN=demo-token -- /bin/sh -lc' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service list --machine cloud-aws' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service status --machine cloud-aws' <<<"$HELP_OUTPUT"
grep -F 'PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service stop --machine cloud-aws' <<<"$HELP_OUTPUT"
grep -F -- '--name api' <<<"$HELP_OUTPUT"
grep -F '`port service list|status|stop` surface the selected node, target host group, scheduler, and' <<<"$HELP_OUTPUT"
grep -F 'runtime state for the stored placement.' <<<"$HELP_OUTPUT"

rg -n '^## Multi-Node Hosted Service Workflow$' README.md docs/hosted.md
rg -n 'service apply --machine cloud-aws --host-group aws-secondary --name api --kind service' README.md docs/hosted.md
rg -n 'service list --machine cloud-aws' README.md docs/hosted.md
rg -n 'service status --machine cloud-aws --name api' README.md docs/hosted.md
rg -n 'service stop --machine cloud-aws --name api' README.md docs/hosted.md
rg -n 'selected node, target host group, scheduler, and' README.md docs/hosted.md
rg -n 'runtime state through the same canonical service output|runtime state\.' README.md docs/hosted.md
