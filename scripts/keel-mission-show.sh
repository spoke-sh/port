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

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

run_keel() {
  if command -v nix >/dev/null 2>&1 && [[ -f "$repo_root/flake.nix" ]]; then
    nix develop "$repo_root" -c keel "$@"
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

goal_rows() {
  local charter_file="$1"

  awk '
    BEGIN { in_goals=0 }
    /^## Goals$/ { in_goals=1; next }
    in_goals && /^\| ID / { next }
    in_goals && /^\|----/ { next }
    in_goals && /^\| MG-/ {
      line=$0
      sub(/^\|[[:space:]]*/, "", line)
      sub(/[[:space:]]*\|[[:space:]]*$/, "", line)
      count=split(line, parts, /[[:space:]]*\|[[:space:]]*/)
      if (count >= 3) {
        printf "%s\t%s\t%s\n", parts[1], parts[2], parts[3]
      }
    }
    in_goals && /^## / { exit }
  ' "$charter_file"
}

related_epic_ids() {
  local mission_id="$1"
  local readme linked_mission epic_id

  shopt -s nullglob
  for readme in .keel/epics/*/README.md; do
    linked_mission="$(frontmatter_value mission "$readme")"
    if [[ "$linked_mission" == "$mission_id" ]]; then
      epic_id="$(frontmatter_value id "$readme")"
      [[ -n "$epic_id" ]] && printf '%s\n' "$epic_id"
    fi
  done
}

epic_progress_fields() {
  local epic_id="$1"
  local epic_readme=".keel/epics/${epic_id}/README.md"
  local epic_title progress status="planned"
  local done_voyages=0 total_voyages=0 done_stories=0 total_stories=0

  epic_title="$(frontmatter_value title "$epic_readme")"
  progress="$(sed -n 's/^\*\*Progress:\*\* //p' "$epic_readme" | head -n 1)"
  if [[ "$progress" =~ ^([0-9]+)/([0-9]+)\ voyages\ complete,\ ([0-9]+)/([0-9]+)\ stories\ done$ ]]; then
    done_voyages="${BASH_REMATCH[1]}"
    total_voyages="${BASH_REMATCH[2]}"
    done_stories="${BASH_REMATCH[3]}"
    total_stories="${BASH_REMATCH[4]}"
  fi

  if (( total_voyages == 0 )); then
    status="planned"
  elif (( done_voyages == total_voyages )); then
    status="done"
  else
    status="in progress"
  fi

  printf '%s\t%s\t%s/%s voyages\t%s/%s stories\n' \
    "${epic_title:-$epic_id}" \
    "$status" \
    "$done_voyages" \
    "$total_voyages" \
    "$done_stories" \
    "$total_stories"
}

emit_mission_goals() {
  local charter_file="$1"
  local goal_id description verification target board_title board_status detail_one detail_two

  while IFS=$'\t' read -r goal_id description verification; do
    [[ -n "$goal_id" ]] || continue
    case "$verification" in
      board:*)
        target="$(trim "${verification#board:}")"
        IFS=$'\t' read -r board_title board_status detail_one detail_two < <(epic_progress_fields "$target")
        printf '  - %s [%s] %s\n' "$goal_id" "$board_status" "$description"
        printf '    board: %s (%s, %s, %s)\n' "$board_title" "$target" "$detail_one" "$detail_two"
        ;;
      manual:*)
        printf '  - %s [manual] %s\n' "$goal_id" "$description"
        printf '    manual: %s\n' "$(trim "${verification#manual:}")"
        ;;
      *)
        printf '  - %s [info] %s\n' "$goal_id" "$description"
        printf '    signal: %s\n' "$verification"
        ;;
    esac
  done < <(goal_rows "$charter_file")
}

story_readmes_for_epic() {
  local epic_id="$1"
  rg -l "^scope: ${epic_id}(/.*)?$" .keel/stories -g 'README.md' 2>/dev/null | sort || true
}

add_unique_line() {
  local value="$1"
  local file="$2"
  [[ -n "$value" ]] || return 0
  if [[ ! -f "$file" ]] || ! grep -Fxq "$value" "$file" 2>/dev/null; then
    printf '%s\n' "$value" >>"$file"
  fi
}

emit_section_from_file() {
  local file="$1"
  local prefix="$2"
  local limit="$3"
  local count=0
  local line

  [[ -s "$file" ]] || return 1
  while IFS= read -r line; do
    printf '%s%s\n' "$prefix" "$line"
    count=$((count + 1))
    if (( count >= limit )); then
      break
    fi
  done < "$file"
}

collect_demo_scripts_from_text() {
  local text="$1"
  local demos_file="$2"
  local value

  while IFS= read -r value; do
    value="$(trim "$value")"
    value="${value#./}"
    [[ -n "$value" ]] || continue
    add_unique_line "$value" "$demos_file"
  done < <(
    printf '%s\n' "$text" \
      | grep -oE '\./scripts/[A-Za-z0-9._/-]+\.sh|scripts/[A-Za-z0-9._/-]+\.sh' \
      | sort -u
  )
}

collect_artifacts_from_doc_path() {
  local path="$1"
  local demos_file="$2"
  local text

  [[ -f "$path" ]] || return 0
  text="$(sed -n '1,520p' "$path")"
  collect_demo_scripts_from_text "$text" "$demos_file"
}

collect_evidence_paths_for_story() {
  local story_readme="$1"
  local story_dir manifest evidence_path

  story_dir="$(dirname "$story_readme")"
  manifest="$story_dir/manifest.yaml"

  if [[ -f "$manifest" ]]; then
    while IFS= read -r evidence_path; do
      evidence_path="$(trim "$evidence_path")"
      [[ -n "$evidence_path" ]] || continue
      printf '%s\n' "$story_dir/$evidence_path"
    done < <(awk '/^  EVIDENCE\// { path=$1; sub(/:$/, "", path); print path }' "$manifest")
    return 0
  fi

  if [[ -d "$story_dir/EVIDENCE" ]]; then
    find "$story_dir/EVIDENCE" -type f 2>/dev/null | sort
  fi
}

collect_artifacts_for_story() {
  local story_readme="$1"
  local visuals_file="$2"
  local proofs_file="$3"
  local demos_file="$4"
  local story_text artifact source_text

  story_text="$(sed -n '1,260p' "$story_readme")"
  collect_demo_scripts_from_text "$story_text" "$demos_file"

  while IFS= read -r artifact; do
    artifact="$(trim "$artifact")"
    [[ -n "$artifact" ]] || continue
    case "$artifact" in
      *.png|*.jpg|*.jpeg|*.gif|*.webm|*.mp4|*.mov)
        add_unique_line "$artifact" "$visuals_file"
        ;;
      *)
        add_unique_line "$artifact" "$proofs_file"
        ;;
    esac

    case "$artifact" in
      *.log|*.txt|*.md)
        if [[ -f "$artifact" ]]; then
          source_text="$(sed -n '1,240p' "$artifact")"
          collect_demo_scripts_from_text "$source_text" "$demos_file"
        fi
        ;;
    esac
  done < <(collect_evidence_paths_for_story "$story_readme")
}

preferred_demo_script() {
  local file="$1"
  local line

  [[ -s "$file" ]] || return 0
  while IFS= read -r line; do
    if [[ "$line" == *hosted-external-project-demo.sh ]]; then
      printf '%s\n' "$line"
      return 0
    fi
  done < "$file"

  while IFS= read -r line; do
    if [[ "$line" == *external-project* ]]; then
      printf '%s\n' "$line"
      return 0
    fi
  done < "$file"

  while IFS= read -r line; do
    if [[ "$line" == *demo.sh ]]; then
      printf '%s\n' "$line"
      return 0
    fi
  done < "$file"

  head -n 1 "$file"
}

emit_artifact_gallery() {
  local mission_id="$1"
  local visuals_file proofs_file demos_file
  local epic_id story_readme primary_visual primary_proof primary_demo script_path

  visuals_file="$(mktemp)"
  proofs_file="$(mktemp)"
  demos_file="$(mktemp)"

  collect_artifacts_from_doc_path "README.md" "$demos_file"
  collect_artifacts_from_doc_path "docs/operators.md" "$demos_file"
  shopt -s nullglob
  for script_path in scripts/*external-project*demo.sh; do
    add_unique_line "$script_path" "$demos_file"
  done

  while IFS= read -r epic_id; do
    [[ -n "$epic_id" ]] || continue
    while IFS= read -r story_readme; do
      [[ -n "$story_readme" ]] || continue
      collect_artifacts_for_story "$story_readme" "$visuals_file" "$proofs_file" "$demos_file"
    done < <(story_readmes_for_epic "$epic_id")
  done < <(related_epic_ids "$mission_id" | sort)

  primary_visual="$(head -n 1 "$visuals_file" 2>/dev/null || true)"
  primary_proof="$(head -n 1 "$proofs_file" 2>/dev/null || true)"
  primary_demo="$(preferred_demo_script "$demos_file")"

  echo "  Primary proof:"
  printf '    - entrypoint: %s\n' "just mission ${mission_id}"
  if [[ -n "$primary_demo" ]]; then
    printf '    - demo script: %s\n' "$primary_demo"
  fi
  if [[ -n "$primary_visual" ]]; then
    printf '    - review artifact: %s\n' "$primary_visual"
  elif [[ -n "$primary_proof" ]]; then
    printf '    - proof artifact: %s\n' "$primary_proof"
  else
    echo "    - No featured proof recorded yet."
  fi

  echo "  Recorded proof media:"
  if ! emit_section_from_file "$visuals_file" "    - " 6; then
    echo "    - No visual artifacts recorded."
  fi

  echo "  Proof artifacts:"
  if ! emit_section_from_file "$proofs_file" "    - " 8; then
    echo "    - No proof artifacts recorded."
  fi

  echo "  Demo scripts:"
  if ! emit_section_from_file "$demos_file" "    - " 6; then
    echo "    - No demo scripts recorded."
  fi

  rm -f "$visuals_file" "$proofs_file" "$demos_file"
}

emit_key_achievements() {
  local mission_id="$1"
  local log_file=".keel/missions/${mission_id}/LOG.md"
  local found=0
  local line

  if [[ -f "$log_file" ]]; then
    while IFS= read -r line; do
      printf '  - %s\n' "$(trim "$line")"
      found=1
    done < <(grep -E '^(Completed|Submitted|Accepted) ' "$log_file" | tail -n 5 || true)
  fi

  if (( found == 0 )); then
    echo "  - No mission achievements recorded yet."
  fi
}

emit_next() {
  local mission_id="$1"
  local next_output

  if next_output="$(run_keel mission next "$mission_id" 2>&1)"; then
    printf '%s\n' "$next_output" | sed 's/^/  /'
  else
    echo "  No mission next step available."
  fi
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
charter_file=".keel/missions/${mission_id}/CHARTER.md"

echo "Mission report"
printf '  Mission: %s (%s)\n' "$title" "$mission_id"
printf '  Status: %s\n' "$status"
printf '  Selection: %s\n' "$selection_note"
echo
echo "Mission goals"
emit_mission_goals "$charter_file"
echo
echo "Key achievements"
emit_key_achievements "$mission_id"
echo
echo "Artifact gallery"
emit_artifact_gallery "$mission_id"
echo
echo "Next"
emit_next "$mission_id"
