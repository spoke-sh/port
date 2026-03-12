#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

help_output="$(nix develop -c just port --help)"
printf '%s\n' "$help_output"

grep -F "Quick start:" <<<"$help_output"
grep -F "Examples:" <<<"$help_output"
grep -F "port doctor" <<<"$help_output"
grep -F "machine list" <<<"$help_output"
grep -F "artifacts build --artifact demo-kernel" <<<"$help_output"
grep -F "machine launch --machine demo" <<<"$help_output"
grep -F "guest exec --machine demo -- /bin/sh -lc 'cat /proc/version'" <<<"$help_output"
grep -F "CONFIGURATION.md" <<<"$help_output"
grep -F "docs/operators.md" <<<"$help_output"

help_example_count="$(
  awk '
    /^Examples:$/ {in_examples=1; next}
    /^Detailed examples:$/ {in_examples=0}
    in_examples && /^  port / {count++}
    END {print count + 0}
  ' <<<"$help_output"
)"
test "$help_example_count" -eq 5

readme_example_count="$(grep -c '^port ' README.md)"
test "$readme_example_count" -eq 5
rg -n 'CONFIGURATION.md|docs/operators.md|docs/hosted.md|docs/cloud.md|docs/artifacts.md' README.md

nix develop -c cargo test -q -p port-cli help_includes_primary_surfaces
