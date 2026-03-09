#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

cargo test -q -p port-runtime hosted_imported_inventory_persists_and_loads_imported_node_records
cargo test -q -p port-runtime hosted_imported_inventory_rejects_unknown_runtime_only_nodes
