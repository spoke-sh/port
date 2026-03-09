#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model sample_config_exposes_expected_sections
cargo test -q -p port-model sample_config_derives_hosted_node_inventory_contract
cargo test -q -p port-model sample_config_derives_hosted_machine_lifecycle_contracts
cargo test -q -p port-hosted-protocol route_context_preserves_inventory_and_guest_broker_context
cargo test -q -p port-runtime service_status_exposes_runtime_contract_even_before_execution
cargo test -q -p port-runtime hosted_service_lifecycle_routes_through_live_runtime_and_persists_redacted_state
