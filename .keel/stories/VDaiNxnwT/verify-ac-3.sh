#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

rg -n "goal_rows|emit_artifact_gallery|mission next" scripts/mission-report.sh
! rg -n "keel throughput|emit_trend|run_keel mission show" scripts/mission-report.sh
