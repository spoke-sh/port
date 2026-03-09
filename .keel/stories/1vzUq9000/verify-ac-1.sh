#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

help_output="$(cargo run -q -p port-cli -- --help)"
control_help="$(cargo run -q -p port-cli -- control-plane serve --help)"
node_help="$(cargo run -q -p port-cli -- node-agent serve --help)"

printf '%s\n' "$help_output"
printf '\n== control-plane serve --help ==\n%s\n' "$control_help"
printf '\n== node-agent serve --help ==\n%s\n' "$node_help"

grep -F ".port/hosted/<control-plane>/registered-nodes.json" <<<"$help_output"
grep -F ".port/hosted/<control-plane>/imported-inventory.json" <<<"$help_output"
grep -F "routing-eligibility detail" <<<"$help_output"
grep -F "first-class \`port inventory import\` command" <<<"$help_output"
grep -F "reload durable" <<<"$control_help"
grep -F "refreshing durable registration" <<<"$node_help"

rg -n 'registered-nodes.json|imported-inventory.json|routing-eligibility|machine status --machine cloud-aws' README.md docs/hosted.md
rg -n 'first-class `port inventory import` command' README.md docs/hosted.md
