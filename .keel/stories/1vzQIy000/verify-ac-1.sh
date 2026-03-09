#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-cli help_includes_primary_surfaces

rg -n --fixed-strings 'Hosted `guest forward` now supports foreground and detached lifecycle modes' README.md
rg -n --fixed-strings 'Hosted `guest forward --list`, `--stop`, `--lifecycle detached`, and `--name`' README.md
rg -n --fixed-strings 'Hosted detached lifecycle now ships through the same surface' docs/hosted.md
rg -n --fixed-strings 'guest().forward_detached_start()' docs/sdk.md
rg -n --fixed-strings 'forward_detached_list()' docs/sdk.md
rg -n --fixed-strings 'forward_detached_stop()' docs/sdk.md
rg -n --fixed-strings 'guest forward detached start:' scripts/hosted-demo.sh
rg -n --fixed-strings 'guest forward detached list:' scripts/hosted-demo.sh
rg -n --fixed-strings 'guest forward detached stop:' scripts/hosted-demo.sh
