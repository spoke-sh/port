#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

rg -n "keel mission show|keel mission next|keel throughput" scripts/mission-report.sh
