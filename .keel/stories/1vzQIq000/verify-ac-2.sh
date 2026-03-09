#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-hosted-protocol route_context_preserves_inventory_and_guest_broker_context
nix develop -c cargo test -q -p port-hosted-protocol route_context_serializes_detached_forward_identity
nix develop -c cargo test -q -p port-sdk hosted_client_surfaces_route_context_from_live_errors
