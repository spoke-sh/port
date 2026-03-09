#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-sdk service_requests_cover_secret_and_sandbox_surfaces
nix develop -c cargo test -q -p port-cli --test service_commands cli_service_commands_cover_hosted_secret_service_and_sandbox_contracts
