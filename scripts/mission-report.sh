#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

frontmatter_value() {
  local key="$1"
  local file="$2"
  sed -n "s/^${key}: //p" "$file" | head -n 1
}

run_keel() {
  if command -v nix >/dev/null 2>&1 && [[ -f flake.nix ]]; then
    nix develop -c keel "$@"
  else
    keel "$@"
  fi
}

select_mission() {
  local best_id=""
  local best_rank=999
  local best_ts=""
  local readme id status ts_key ts rank

  shopt -s nullglob
  for readme in .keel/missions/*/README.md; do
    id="$(frontmatter_value id "$readme")"
    status="$(frontmatter_value status "$readme")"
    case "$status" in
      active) rank=0; ts_key="activated_at" ;;
      achieved) rank=1; ts_key="achieved_at" ;;
      paused) rank=2; ts_key="updated_at" ;;
      defining) rank=3; ts_key="created_at" ;;
      verified) rank=4; ts_key="verified_at" ;;
      abandoned) rank=5; ts_key="updated_at" ;;
      *) rank=9; ts_key="updated_at" ;;
    esac
    ts="$(frontmatter_value "$ts_key" "$readme")"
    if [[ -z "$ts" ]]; then
      ts="$(frontmatter_value updated_at "$readme")"
    fi
    if [[ -z "$ts" ]]; then
      ts="$(frontmatter_value created_at "$readme")"
    fi
    if [[ -z "$best_id" || "$rank" -lt "$best_rank" || ( "$rank" -eq "$best_rank" && "$ts" > "$best_ts" ) ]]; then
      best_id="$id"
      best_rank="$rank"
      best_ts="$ts"
    fi
  done

  if [[ -z "$best_id" ]]; then
    echo "no mission found under .keel/missions" >&2
    return 1
  fi

  printf '%s\n' "$best_id"
}

emit_epic_progress() {
  local mission_id="$1"
  local found=0
  local readme linked_mission epic_id title progress

  shopt -s nullglob
  for readme in .keel/epics/*/README.md; do
    linked_mission="$(frontmatter_value mission "$readme")"
    if [[ "$linked_mission" != "$mission_id" ]]; then
      continue
    fi
    found=1
    epic_id="$(frontmatter_value id "$readme")"
    title="$(frontmatter_value title "$readme")"
    progress="$(sed -n 's/^\*\*Progress:\*\* //p' "$readme" | head -n 1)"
    if [[ -z "$progress" ]]; then
      progress="progress unavailable"
    fi
    printf '  - %s %s :: %s\n' "$epic_id" "$title" "$progress"
  done

  if [[ "$found" -eq 0 ]]; then
    echo "  (no epics linked to this mission)"
  fi
}

emit_next() {
  local mission_id="$1"
  local status="$2"
  local next_output

  if next_output="$(NO_COLOR=1 run_keel mission next "$mission_id" 2>/dev/null)"; then
    printf '%s\n' "$next_output" | sed 's/^/  /'
    return 0
  fi

  case "$status" in
    verified|abandoned)
      echo "  No actionable next step; mission is $status."
      ;;
    *)
      echo "  No mission next step available."
      ;;
  esac
}

emit_trend() {
  local throughput_output

  throughput_output="$(NO_COLOR=1 run_keel throughput 2>/dev/null || true)"
  if [[ -z "$throughput_output" ]]; then
    echo "  Throughput signal unavailable."
    return 0
  fi

  printf '%s\n' "$throughput_output" \
    | awk '
        /Throughput \(weekly\)/ {capture=1}
        capture {print}
        capture && /Avg now/ {exit}
      ' \
    | sed 's/^/  /'
}

mission_id="${1:-}"
selection_note="explicit"
if [[ -z "$mission_id" ]]; then
  mission_id="$(select_mission)"
  selection_note="auto"
fi

mission_readme=".keel/missions/${mission_id}/README.md"
if [[ ! -f "$mission_readme" ]]; then
  echo "mission '$mission_id' not found" >&2
  exit 1
fi

title="$(frontmatter_value title "$mission_readme")"
status="$(frontmatter_value status "$mission_readme")"

echo "Mission report"
printf '  Mission: %s (%s)\n' "$title" "$mission_id"
printf '  Status: %s\n' "$status"
printf '  Selection: %s\n' "$selection_note"
echo
NO_COLOR=1 run_keel mission show "$mission_id"
echo
echo "Epic progress"
emit_epic_progress "$mission_id"
echo
echo "Next"
emit_next "$mission_id" "$status"
echo
echo "Trend"
emit_trend
