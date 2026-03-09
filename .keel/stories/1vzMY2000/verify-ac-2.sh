#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

nix develop -c cargo test -q -p port-runtime hosted_guest_forward_errors_include_route_context
nix develop -c cargo test -q -p port-cli cli_guest_forward_rejects_hosted_detached_lifecycle
