#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
nix develop -c cargo test -q -p port-runtime
