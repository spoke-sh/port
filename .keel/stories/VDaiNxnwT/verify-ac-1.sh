#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

output="$(just mission VDaiFelPf)"
printf '%s\n' "$output"

grep -F "Mission report" <<<"$output"
grep -F "Improve Operator Signal And Documentation Experience" <<<"$output"
grep -F "Epic progress" <<<"$output"
grep -F "Throughput (weekly)" <<<"$output"
