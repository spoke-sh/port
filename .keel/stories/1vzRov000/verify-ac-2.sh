#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

help_output="$(mktemp)"
service_help="$(mktemp)"
trap 'rm -f "$help_output" "$service_help"' EXIT

nix develop -c cargo run -q -p port-cli -- --help >"$help_output"
nix develop -c cargo run -q -p port-cli -- service --help >"$service_help"

rg -n --fixed-strings 'Service Control:' "$help_output"
rg -n --fixed-strings '`port service` is the canonical secrets/services/sandboxes family' "$help_output"
rg -n --fixed-strings 'Managed guest-process `start|list|status|stop` is an internal contract beneath that same surface,' "$help_output"
rg -n --fixed-strings 'not a second hosted-only CLI family.' "$help_output"

rg -n --fixed-strings 'secret  Manage machine-bound secret references' "$service_help"
rg -n --fixed-strings 'apply   Store a service or sandbox definition under the resolved runtime owner' "$service_help"
rg -n --fixed-strings 'list    List stored service and sandbox definitions for a machine' "$service_help"
rg -n --fixed-strings 'status  Inspect one stored service or sandbox definition' "$service_help"
rg -n --fixed-strings 'stop    Set a stored service or sandbox definition to the stopped desired state' "$service_help"

rg -n --fixed-strings 'Managed guest-process `start|list|status|stop` is an internal runtime' README.md
rg -n --fixed-strings 'not a second hosted-only CLI surface.' README.md
rg -n --fixed-strings 'managed guest-process `start|list|status|stop` remains an internal guest and' docs/hosted.md
rg -n --fixed-strings 'is not a hosted-only CLI family.' docs/hosted.md
rg -n --fixed-strings 'managed guest-process `start|list|status|stop` stays internal to the shared' docs/sdk.md
rg -n --fixed-strings 'guest/runtime contract, so the SDK does not add a second hosted-only service' docs/sdk.md
rg -n --fixed-strings 'client family' docs/sdk.md
