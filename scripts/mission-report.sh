#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

frontmatter_value() {
  local key="$1"
  local file="$2"
  sed -n "s/^${key}: //p" "$file" | head -n 1
}

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

normalize_human_text() {
  local value="$1"

  value="${value//just keel doctor/just doctor}"
  value="${value//just keel flow/just flow}"
  value="${value//canonical mission signal/mission report}"
  value="${value//mission-signal path/mission report path}"
  value="${value//a visual throughput or progress plot/high-level artifacts}"
  value="${value//visual throughput or progress plot/high-level artifacts}"
  value="${value//throughput sparkline/high-level artifacts}"
  value="${value//high-level proof signals/artifact gallery}"
  value="${value//high-level verification signals/artifact gallery}"
  value="${value//runs the canonical verification path and ends with/presents}"

  printf '%s' "$value"
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

story_readmes_for_epic() {
  local epic_id="$1"

  rg -l "^scope: ${epic_id}(/.*)?$" .keel/stories -g 'README.md' 2>/dev/null || true
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

  if [[ ! -s "$file" ]]; then
    return 1
  fi

  while IFS= read -r line; do
    printf '%s%s\n' "$prefix" "$line"
    count=$((count + 1))
    if (( count >= limit )); then
      break
    fi
  done < "$file"

  return 0
}

resolve_verify_script_path() {
  local verify_cmd="$1"
  local path=""

  path="$(
    printf '%s\n' "$verify_cmd" \
      | grep -oE '/home/alex/workspace/spoke-sh/port/[^[:space:]\")]+\.sh|scripts/[^[:space:]\")]+\.sh|\.keel/stories/[^[:space:]\")]+\.sh' \
      | head -n 1 \
      || true
  )"

  if [[ "$path" == "${repo_root}/"* ]]; then
    path="${path#${repo_root}/}"
  fi

  if [[ -n "$path" && -f "$path" ]]; then
    printf '%s\n' "$path"
  fi
}

normalize_demo_command() {
  local value="$1"

  value="${value#\'}"
  value="${value%\'}"
  value="${value#\"}"
  value="${value%\"}"
  value="${value#\(}"
  value="${value%\)}"
  value="${value#\`}"
  value="${value%\`}"
  value="${value#bash -lc }"
  value="${value//\/tmp\/port-target\/debug\/port/port}"
  value="${value//cargo run -q -p port-cli -- /port }"
  value="${value//cargo run -p port-cli -- /port }"
  value="${value//${repo_root}\//}"
  value="${value//cd ${repo_root} && /}"
  value="$(trim "$value")"

  printf '%s' "$value"
}

collect_doc_artifacts_from_text() {
  local text="$1"
  local docs_file="$2"
  local path

  while IFS= read -r path; do
    path="$(trim "$path")"
    [[ -n "$path" ]] || continue
    add_unique_line "$path" "$docs_file"
  done < <(
    printf '%s\n' "$text" \
      | grep -oE 'README\.md|CONSTITUTION\.md|ARCHITECTURE\.md|CONFIGURATION\.md|RELEASE\.md|EVALUATIONS\.md|AGENTS\.md|docs/[A-Za-z0-9._/-]+\.md|examples/[A-Za-z0-9._/-]+\.toml|justfile' \
      | sort -u
  )
}

collect_demo_artifacts_from_text() {
  local text="$1"
  local demos_file="$2"
  local value

  while IFS= read -r value; do
    value="$(trim "$value")"
    case "$value" in
      just\ *|port\ *|nix\ develop\ -c\ just\ *|bash\ scripts/*.sh|scripts/*.sh|PORT_[A-Z0-9_]*=*\ just\ *|PORT_[A-Z0-9_]*=*\ port\ *)
        ;;
      *)
        continue
        ;;
    esac

    value="$(normalize_demo_command "$value")"
    [[ -n "$value" ]] || continue
    case "$value" in
      cargo\ test*|rg\ *|grep\ *|test\ *|for\ *|if\ *|printf\ *|awk\ *|sed\ *|cat\ *|port\ [A-Z]*|port\ */*)
        continue
        ;;
    esac
    add_unique_line "$value" "$demos_file"
  done < <(printf '%s\n' "$text")
}

collect_artifacts_from_doc_path() {
  local path="$1"
  local docs_file="$2"
  local demos_file="$3"
  local text

  add_unique_line "$path" "$docs_file"
  if [[ ! -f "$path" ]]; then
    return 0
  fi

  text="$(sed -n '1,520p' "$path")"
  collect_demo_artifacts_from_text "$text" "$demos_file"
}

collect_visual_artifacts_for_story() {
  local story_readme="$1"
  local visuals_file="$2"
  local story_dir media

  story_dir="$(dirname "$story_readme")"
  if [[ -d "$story_dir/EVIDENCE" ]]; then
    while IFS= read -r media; do
      media="$(trim "$media")"
      [[ -n "$media" ]] || continue
      add_unique_line "$media" "$visuals_file"
    done < <(
      find "$story_dir/EVIDENCE" -type f \
        \( -name '*.png' -o -name '*.jpg' -o -name '*.jpeg' -o -name '*.gif' -o -name '*.webm' -o -name '*.mp4' \) \
        2>/dev/null \
        | sort
    )
  fi
}

collect_human_artifacts_for_story() {
  local story_readme="$1"
  local docs_file="$2"
  local demos_file="$3"
  local line description meta verify_cmd script_path source_text referenced_docs doc_path

  referenced_docs="$(mktemp)"

  while IFS= read -r line; do
    if [[ "$line" == "- [x] "* && "$line" == *"<!--"* && "$line" == *" verify: "* ]]; then
      description="${line%% <!--*}"
      description="$(printf '%s\n' "$description" | sed 's/^- \[x\] //; s/^\[[^]]*\] //')"
      meta="${line#* verify: }"
      verify_cmd="${meta%%, proof:*}"

      source_text="$description"$'\n'"$verify_cmd"
      script_path="$(resolve_verify_script_path "$verify_cmd" || true)"
      if [[ -n "$script_path" ]]; then
        if [[ "$script_path" == scripts/* ]]; then
          add_unique_line "$script_path" "$demos_file"
        fi
        source_text+=$'\n'"$(sed -n '1,240p' "$script_path")"
      fi

      collect_doc_artifacts_from_text "$source_text" "$referenced_docs"
      collect_demo_artifacts_from_text "$source_text" "$demos_file"
    fi
  done < "$story_readme"

  while IFS= read -r doc_path; do
    doc_path="$(trim "$doc_path")"
    [[ -n "$doc_path" ]] || continue
    collect_artifacts_from_doc_path "$doc_path" "$docs_file" "$demos_file"
  done < "$referenced_docs"

  rm -f "$referenced_docs"
}

epic_progress_fields() {
  local epic_id="$1"
  local epic_readme=".keel/epics/${epic_id}/README.md"
  local epic_title voyage_readme voyage_status progress
  local total_voyages=0 done_voyages=0 total_stories=0 done_stories=0 status="planned"

  epic_title="$(frontmatter_value title "$epic_readme")"

  shopt -s nullglob
  for voyage_readme in .keel/epics/"$epic_id"/voyages/*/README.md; do
    total_voyages=$((total_voyages + 1))
    voyage_status="$(frontmatter_value status "$voyage_readme")"
    if [[ "$voyage_status" == "done" ]]; then
      done_voyages=$((done_voyages + 1))
    fi
    progress="$(sed -n 's/^\*\*Progress:\*\* //p' "$voyage_readme" | head -n 1)"
    if [[ "$progress" =~ ^([0-9]+)/([0-9]+)\ stories ]]; then
      done_stories=$((done_stories + ${BASH_REMATCH[1]}))
      total_stories=$((total_stories + ${BASH_REMATCH[2]}))
    fi
  done

  if (( total_voyages == 0 )); then
    status="planned"
  elif (( done_voyages == total_voyages )); then
    status="done"
  else
    status="in progress"
  fi

  printf '%s\t%s\t%d/%d voyages\t%d/%d stories\n' \
    "$epic_title" \
    "$status" \
    "$done_voyages" \
    "$total_voyages" \
    "$done_stories" \
    "$total_stories"
}

emit_mission_goals() {
  local charter_file="$1"
  local goal_id description verification target epic_title epic_status voyage_progress story_progress

  while IFS=$'\t' read -r goal_id description verification; do
    [[ -n "$goal_id" ]] || continue
    case "$verification" in
      board:*)
        target="$(trim "${verification#board:}")"
        IFS=$'\t' read -r epic_title epic_status voyage_progress story_progress < <(epic_progress_fields "$target")
        description="$(normalize_human_text "$description")"
        printf '  - %s [%s] %s\n' "$goal_id" "$epic_status" "$description"
        printf '    board: %s (%s, %s, %s)\n' "$epic_title" "$target" "$voyage_progress" "$story_progress"
        ;;
      manual:*)
        description="$(normalize_human_text "$description")"
        printf '  - %s [manual] %s\n' "$goal_id" "$description"
        printf '    manual: %s\n' "$(normalize_human_text "$(trim "${verification#manual:}")")"
        ;;
      metric:*)
        description="$(normalize_human_text "$description")"
        printf '  - %s [metric] %s\n' "$goal_id" "$description"
        printf '    metric: %s\n' "$(normalize_human_text "$(trim "${verification#metric:}")")"
        ;;
      *)
        description="$(normalize_human_text "$description")"
        printf '  - %s [info] %s\n' "$goal_id" "$description"
        printf '    signal: %s\n' "$(normalize_human_text "$verification")"
        ;;
    esac
  done < <(goal_rows "$charter_file")
}

emit_key_achievements() {
  local mission_id="$1"
  local log_file=".keel/missions/${mission_id}/LOG.md"
  local found=0

  if [[ -f "$log_file" ]]; then
    while IFS= read -r line; do
      printf '  - %s\n' "$(trim "${line#- }")"
      found=1
    done < <(grep "Completed" "$log_file" | tail -n 5 || true)

    if (( found == 0 )); then
      awk '
        /^## / { section++ ; next }
        section > 0 && NF { print; count++; if (count == 3) exit }
      ' "$log_file" | sed 's/^/  /'
      return 0
    fi
    return 0
  fi

  echo "  No mission log found."
}

play_visual_artifacts() {
  local visuals_file="$1"
  local -a proofs=()
  local proof

  [[ -t 1 ]] || return 0
  [[ "${MISSION_NO_PLAYBACK:-0}" == "1" ]] && return 0
  [[ -s "$visuals_file" ]] || return 0

  while IFS= read -r proof; do
    proofs+=("$proof")
  done < "$visuals_file"

  open_visual() {
    local file="$1"
    (xdg-open "$file" >/dev/null 2>&1 || open "$file" >/dev/null 2>&1 || printf '    manual: %s/%s\n' "$repo_root" "$file") &
  }

  tmux_client_value() {
    local fmt="$1"
    if [[ -n "${TMUX:-}" ]] && command -v tmux >/dev/null 2>&1; then
      tmux display-message -p "$fmt" 2>/dev/null || true
    fi
  }

  tmux_env_value() {
    local key="$1"
    if [[ -n "${TMUX:-}" ]] && command -v tmux >/dev/null 2>&1; then
      tmux show-environment -gv "$key" 2>/dev/null || true
    fi
  }

  terminal_signature() {
    printf '%s\n' \
      "${TERM:-}" \
      "${TERM_PROGRAM:-}" \
      "${LC_TERMINAL:-}" \
      "${KITTY_WINDOW_ID:-}" \
      "${WEZTERM_EXECUTABLE:-}" \
      "$(tmux_env_value TERM_PROGRAM)" \
      "$(tmux_env_value LC_TERMINAL)" \
      "$(tmux_client_value '#{client_termname}')" \
      "$(tmux_client_value '#{client_termtype}')" \
      "$(tmux_client_value '#{client_termfeatures}')"
  }

  choose_chafa_format() {
    local signature
    signature="$(terminal_signature | tr '[:upper:]' '[:lower:]')"

    if printf '%s' "$signature" | grep -Eq 'kitty|ghostty|wezterm'; then
      echo "kitty"
      return
    fi

    if printf '%s' "$signature" | grep -Eq 'iterm'; then
      echo "iterm"
      return
    fi

    if printf '%s' "$signature" | grep -Eq 'sixel'; then
      echo "sixels"
      return
    fi

    echo "auto"
  }

  choose_chafa_passthrough() {
    if [[ -n "${TMUX:-}" ]]; then
      echo "tmux"
    else
      echo "none"
    fi
  }

  render_with_chafa() {
    local file="$1"
    local duration="$2"
    local format="$3"
    local passthrough="$4"
    local -a args=(--bg "#120d0a" --scale max --passthrough "$passthrough")

    if [[ "$duration" != "0" ]]; then
      args+=(--duration "$duration")
    fi

    case "$format" in
      auto)
        args+=(--probe 0.5 --probe-mode ctty --optimize 9)
        ;;
      kitty|iterm|sixels)
        args+=(--format "$format" --probe 0.5 --probe-mode ctty --optimize 9)
        ;;
      *)
        args+=(--probe 0.5 --probe-mode ctty --optimize 9)
        ;;
    esac

    if command -v chafa >/dev/null 2>&1; then
      chafa "${args[@]}" "$file"
    elif command -v nix >/dev/null 2>&1; then
      nix develop "$repo_root" -c chafa "${args[@]}" "$file"
    else
      return 1
    fi
  }

  local has_chafa=0
  local chafa_format="auto"
  local chafa_passthrough="none"
  if command -v chafa >/dev/null 2>&1 || nix develop "$repo_root" -c chafa --version >/dev/null 2>&1; then
    has_chafa=1
    chafa_format="$(choose_chafa_format)"
    chafa_passthrough="$(choose_chafa_passthrough)"
  fi

  echo
  echo "Playback"
  if (( has_chafa == 1 )); then
    echo "  Renderer: chafa ${chafa_format} (passthrough: ${chafa_passthrough})"
  else
    echo "  Renderer: external viewer fallback"
  fi

  for proof in "${proofs[@]}"; do
    echo "  -> $proof"
    case "$proof" in
      *.png|*.jpg|*.jpeg)
        if (( has_chafa == 1 )); then
          render_with_chafa "$proof" 0 "$chafa_format" "$chafa_passthrough" || open_visual "$proof"
        else
          open_visual "$proof"
        fi
        ;;
      *.gif|*.mp4|*.webm)
        if (( has_chafa == 1 )); then
          render_with_chafa "$proof" 5 "$chafa_format" "$chafa_passthrough" || open_visual "$proof"
        else
          open_visual "$proof"
        fi
        ;;
      *)
        open_visual "$proof"
        ;;
    esac
  done
}

emit_artifact_gallery() {
  local mission_id="$1"
  local visuals_file docs_file demos_file demo_scripts_file run_commands_file
  local epic_id story_readme
  local demo

  visuals_file="$(mktemp)"
  docs_file="$(mktemp)"
  demos_file="$(mktemp)"
  demo_scripts_file="$(mktemp)"
  run_commands_file="$(mktemp)"

  while IFS= read -r epic_id; do
    [[ -n "$epic_id" ]] || continue
    while IFS= read -r story_readme; do
      [[ -n "$story_readme" ]] || continue
      collect_visual_artifacts_for_story "$story_readme" "$visuals_file"
      collect_human_artifacts_for_story "$story_readme" "$docs_file" "$demos_file"
    done < <(story_readmes_for_epic "$epic_id")
  done < <(related_epic_ids "$mission_id")

  while IFS= read -r demo; do
    [[ -n "$demo" ]] || continue
    case "$demo" in
      bash\ scripts/*|scripts/*)
        add_unique_line "$demo" "$demo_scripts_file"
        ;;
      *)
        add_unique_line "$demo" "$run_commands_file"
        ;;
    esac
  done < "$demos_file"

  echo "  Visual evidence:"
  if ! emit_section_from_file "$visuals_file" "    - " 6; then
    echo "    - No visual artifacts recorded."
  fi

  echo "  Demo scripts:"
  if ! emit_section_from_file "$demo_scripts_file" "    - " 6; then
    echo "    - No demo scripts recorded."
  fi

  echo "  Run commands:"
  if ! emit_section_from_file "$run_commands_file" "    - " 8; then
    echo "    - No runnable demos recorded."
  fi

  echo "  Human docs:"
  if ! emit_section_from_file "$docs_file" "    - " 12; then
    echo "    - No human-facing docs recorded."
  fi

  play_visual_artifacts "$visuals_file"

  rm -f "$visuals_file" "$docs_file" "$demos_file" "$demo_scripts_file" "$run_commands_file"
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
emit_next "$mission_id" "$status"
