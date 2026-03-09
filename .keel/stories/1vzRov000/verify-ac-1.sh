#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-agent-protocol managed_service_operations_round_trip_through_json
nix develop -c cargo test -q -p port-hosted-protocol route_context_serializes_service_identity
nix develop -c cargo test -q -p port-runtime service_status_exposes_runtime_contract_even_before_execution
nix develop -c cargo test -q -p port-sdk hosted_client_surfaces_service_identity_from_live_errors
nix develop -c cargo test -q -p port-cli --test service_commands cli_service_commands_cover_hosted_secret_service_and_sandbox_contracts
