#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

output="$(just mission VDaiFelPf)"
printf '%s\n' "$output"

grep -F "Mission report" <<<"$output"
grep -F "Improve Operator Signal And Documentation Experience" <<<"$output"
grep -F "Mission goals" <<<"$output"
grep -F "Key achievements" <<<"$output"
grep -F "Artifact gallery" <<<"$output"
! grep -Fq "Throughput (weekly)" <<<"$output"
