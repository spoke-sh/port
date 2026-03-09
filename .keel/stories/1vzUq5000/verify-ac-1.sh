#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

cargo test -q -p port-model sample_config_derives_hosted_node_inventory_contract
cargo test -q -p port-hosted-protocol hosted_registration_request_and_contract_serialize_stably
