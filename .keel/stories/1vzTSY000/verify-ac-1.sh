#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

cleanup() {
  rm -rf .port crates/port-runtime/.port crates/port-cli/.port
}
trap cleanup EXIT

cargo test -q -p port-cli help_includes_primary_surfaces

demo_output="$(mktemp /tmp/port-registered-demo.XXXXXX)"
PORT_DEMO_TOKEN=demo-token bash scripts/hosted-demo.sh | tee "$demo_output"

grep -q "machine list:" "$demo_output"
grep -q "cloud-aws" "$demo_output"
grep -q "machine status:" "$demo_output"
grep -q "current hosted demo limits:" "$demo_output"

