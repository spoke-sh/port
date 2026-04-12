#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage: scripts/render-mission-proof.sh <mission-id> [--register]

Render a mission-level verification.cast + verification.gif artifact by
recording the mission surface, charter excerpt, and linked board entities.

When --register is passed, update the mission README frontmatter to point at
verification.gif and refresh updated_at.
EOF
  exit 64
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
fi

mission_id=$1
register=${2:-}
if [[ -n "$register" && "$register" != "--register" ]]; then
  usage
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

python3 - "$repo_root" "$mission_id" "$register" <<'PY'
import datetime as dt
import json
import os
import pathlib
import re
import subprocess
import sys
import time


repo_root = pathlib.Path(sys.argv[1]).resolve()
mission_id = sys.argv[2]
register = sys.argv[3] == "--register"
mission_dir = repo_root / ".keel" / "missions" / mission_id
readme_path = mission_dir / "README.md"
charter_path = mission_dir / "CHARTER.md"
cast_path = mission_dir / "verification.cast"
gif_path = mission_dir / "verification.gif"

if not mission_dir.is_dir():
    raise SystemExit(f"mission directory not found: {mission_dir}")
if not readme_path.is_file():
    raise SystemExit(f"mission README not found: {readme_path}")
if not charter_path.is_file():
    raise SystemExit(f"mission CHARTER not found: {charter_path}")


def run_checked(argv: list[str], *, cwd: pathlib.Path) -> str:
    result = subprocess.run(
        argv,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=True,
    )
    output = result.stdout
    if result.stderr:
        output += result.stderr
    return output if output.endswith("\n") else f"{output}\n"


def entity_command(entity_id: str) -> list[str] | None:
    if (repo_root / ".keel" / "epics" / entity_id / "README.md").is_file():
        return ["keel", "epic", "show", entity_id]
    if (repo_root / ".keel" / "bearings" / entity_id / "README.md").is_file():
        return ["keel", "bearing", "show", entity_id]
    if (repo_root / ".keel" / "stories" / entity_id / "README.md").is_file():
        return ["keel", "story", "show", entity_id]
    return None


charter_text = charter_path.read_text(encoding="utf-8")
board_entities: list[str] = []
seen_entities: set[str] = set()
for match in re.finditer(r"board:\s*([A-Za-z0-9]+)", charter_text):
    entity_id = match.group(1)
    if entity_id not in seen_entities:
        seen_entities.add(entity_id)
        board_entities.append(entity_id)

steps: list[tuple[str, str]] = [
    (
        f"keel mission show {mission_id}",
        run_checked(["keel", "mission", "show", mission_id], cwd=repo_root),
    ),
    (
        f"sed -n '1,120p' .keel/missions/{mission_id}/CHARTER.md",
        run_checked(
            ["sed", "-n", "1,120p", str(charter_path.relative_to(repo_root))],
            cwd=repo_root,
        ),
    ),
]

for entity_id in board_entities:
    command = entity_command(entity_id)
    if command is None:
        continue
    steps.append(
        (
            " ".join(command),
            run_checked(command, cwd=repo_root),
        )
    )


def write_cast(path: pathlib.Path, steps: list[tuple[str, str]], width: int, height: int) -> None:
    header = {
        "version": 2,
        "width": width,
        "height": height,
        "timestamp": int(time.time()),
        "env": {
            "SHELL": "/bin/bash",
            "TERM": "xterm-256color",
        },
    }
    with path.open("w", encoding="utf-8") as handle:
        handle.write(json.dumps(header) + "\n")
        offset = 0.0
        for command, output in steps:
            prompt = f"$ {command}\n".replace("\n", "\r\n")
            handle.write(json.dumps([round(offset, 2), "o", prompt]) + "\n")
            offset += 0.45
            handle.write(
                json.dumps([round(offset, 2), "o", output.replace("\n", "\r\n")]) + "\n"
            )
            offset += 1.35


write_cast(cast_path, steps, width=144, height=42)
subprocess.run(
    [
        "agg",
        "--theme",
        "github-dark",
        "--font-size",
        "14",
        "--cols",
        "144",
        "--rows",
        "42",
        "--idle-time-limit",
        "1.2",
        "--last-frame-duration",
        "2",
        str(cast_path),
        str(gif_path),
    ],
    cwd=repo_root,
    check=True,
)

if register:
    readme_text = readme_path.read_text(encoding="utf-8")
    parts = readme_text.split("---", 2)
    if len(parts) < 3:
        raise SystemExit(f"expected YAML frontmatter in {readme_path}")
    frontmatter = parts[1].strip("\n")
    body = parts[2]
    lines = frontmatter.splitlines()

    updated_at = dt.datetime.now().replace(microsecond=0).isoformat()
    new_lines: list[str] = []
    saw_updated_at = False
    saw_verification_artifact = False
    inserted_artifact = False
    for line in lines:
        if line.startswith("updated_at: "):
            new_lines.append(f"updated_at: {updated_at}")
            saw_updated_at = True
            continue
        if line.startswith("verification_artifact: "):
            new_lines.append("verification_artifact: verification.gif")
            saw_verification_artifact = True
            continue
        new_lines.append(line)
        if line.startswith("verified_at: ") and not saw_verification_artifact:
            new_lines.append("verification_artifact: verification.gif")
            saw_verification_artifact = True
            inserted_artifact = True

    if not saw_updated_at:
        new_lines.append(f"updated_at: {updated_at}")
    if not saw_verification_artifact and not inserted_artifact:
        new_lines.append("verification_artifact: verification.gif")

    readme_path.write_text(
        f"---\n{'\n'.join(new_lines)}\n---{body}",
        encoding="utf-8",
    )

print(f"generated cast: {cast_path}")
print(f"generated gif: {gif_path}")
if register:
    print(f"registered artifact in: {readme_path}")
PY
