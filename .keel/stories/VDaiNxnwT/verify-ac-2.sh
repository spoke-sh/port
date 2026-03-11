#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

help_output="$(just)"
list_output="$(just --list)"
demo_output="$(just --list demo)"

printf '%s\n' "$help_output"
printf '%s\n' "$list_output"
printf '%s\n' "$demo_output"

grep -F "Common recipes:" <<<"$help_output"
grep -F "just --list demo" <<<"$help_output"
! grep -Fq "push-oci" <<<"$list_output"
! grep -Fq "build-kernel" <<<"$list_output"
grep -F "push-oci" <<<"$demo_output"
grep -F "build-kernel" <<<"$demo_output"
