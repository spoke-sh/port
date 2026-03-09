#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime hosted_service_lifecycle_routes_through_live_runtime_and_persists_redacted_state
