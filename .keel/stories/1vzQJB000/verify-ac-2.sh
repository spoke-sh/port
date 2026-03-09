#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-cli cli_guest_forward_lists_and_stops_hosted_detached_forwards
