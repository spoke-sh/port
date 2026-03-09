#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

cargo test -q -p port-model hosted_registered_node_contract_rejects_invalid_registration_inputs
cargo test -q -p port-runtime hosted_imported_inventory_surfaces_import_path_and_node_on_conflict
