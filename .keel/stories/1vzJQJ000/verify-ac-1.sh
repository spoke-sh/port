#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

nix develop -c cargo run -q -p port-cli -- --help >"$tmpdir/help.txt"

rg "Hosted prepared-node PVM workflow:" "$tmpdir/help.txt"
rg "machine launch --machine cloud-aws" "$tmpdir/help.txt"
rg -i "other hosted launch paths still return provider-aware guidance" "$tmpdir/help.txt"

rg "Hosted prepared-node PVM workflow" README.md
rg "machine launch --machine cloud-aws" README.md
rg -i "other hosted launch paths still return provider-aware guidance" README.md

rg -i "Hosted prepared-node" docs/pvm.md
rg "machine launch --machine cloud-aws" docs/pvm.md
rg "prepared node" docs/pvm.md

if rg -n "Hosted launch is still partial|still a follow-on runtime slice|does not yet imply a shipped remote launch path" README.md docs/pvm.md "$tmpdir/help.txt"; then
  echo "stale partial-launch wording is still present" >&2
  exit 1
fi
