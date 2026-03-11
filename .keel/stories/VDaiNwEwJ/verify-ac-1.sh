#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

for file in CONSTITUTION.md ARCHITECTURE.md CONFIGURATION.md RELEASE.md EVALUATIONS.md; do
  test -f "$file"
done

rg -n \
  'CONSTITUTION.md|ARCHITECTURE.md|CONFIGURATION.md|RELEASE.md|EVALUATIONS.md' \
  README.md
rg -n '^# Constitution$' CONSTITUTION.md
rg -n '^# Architecture$' ARCHITECTURE.md
rg -n '^# Configuration Guide$' CONFIGURATION.md
rg -n '^# Release Process$' RELEASE.md
rg -n '^# Evaluation Guide$' EVALUATIONS.md
