#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

frontmatter_value() {
  local key="$1"
  local file="$2"
  [[ -f "$file" ]] || return 0
  sed -n "s/^${key}: //p" "$file" | head -n 1
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

  printf '%s
' "$best_id"
}

mission_id="${1:-}"
if [[ -z "$mission_id" ]]; then
  mission_id="$(select_mission)"
fi

if command -v nix >/dev/null 2>&1 && [[ -f flake.nix ]]; then
  nix develop -c keel mission show "$mission_id"
else
  keel mission show "$mission_id"
fi
