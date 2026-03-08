#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

nix develop -c cargo run -q -p port-cli -- --help >"$tmpdir/help.log"

cat "$tmpdir/help.log"
sed -n '/## PVM Contract/,/## AVF Contract/p' README.md
sed -n '/## Hosted Admission Workflow/,/## Follow-On Work/p' docs/pvm.md
sed -n '/# Hosted PVM admission example:/,/\[machines.cloud-gcp\]/p' examples/port.toml

rg "Hosted PVM admission workflow|Standard lane preservation|aarch64 remains research only" "$tmpdir/help.log"
rg "Hosted PVM admission workflow|cloud-generic|cloud-aws|aarch64" README.md
rg "Hosted Admission Workflow|Preserved Standard Lane|arm64 Boundary" docs/pvm.md
rg "Hosted PVM admission example|Hosted PVM admission-ready example|Standard shipped lane|research-only" examples/port.toml
