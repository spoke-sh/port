#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model hosted_host_group_scheduler_field_is_required
cargo test -q -p port-model hosted_host_group_scheduler_value_must_be_known
