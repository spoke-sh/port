#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-cli help_includes_primary_surfaces
nix develop -c cargo test -q -p port-cli cli_guest_forward_supports_hosted_detached_lifecycle
