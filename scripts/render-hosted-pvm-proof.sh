#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <output-dir>" >&2
  exit 64
fi

output_dir=$1
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
mkdir -p "$output_dir"

python3 - "$repo_root" "$output_dir" <<'PY'
import json
import os
import pathlib
import subprocess
import sys
import time

repo_root = pathlib.Path(sys.argv[1]).resolve()
output_dir = pathlib.Path(sys.argv[2]).resolve()
cast_path = output_dir / "hosted-pvm-workflow.cast"
gif_path = output_dir / "ac-1.gif"


def run_checked(argv: list[str], *, env: dict[str, str], cwd: pathlib.Path) -> str:
    result = subprocess.run(
        argv,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    output = result.stdout
    if result.stderr:
        output += result.stderr
    return output if output.endswith("\n") else f"{output}\n"


def write_cast(
    path: pathlib.Path, steps: list[tuple[str, str]], width: int, height: int
) -> None:
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


env = os.environ.copy()

subprocess.run(
    ["cargo", "build", "-q", "-p", "port", "--bin", "port"],
    cwd=repo_root,
    env=env,
    check=True,
)
subprocess.run(
    ["cargo", "build", "-q", "-p", "port-guest-agent", "--bin", "port-guest-agent"],
    cwd=repo_root,
    env=env,
    check=True,
)

steps = [
    (
        "bash scripts/hosted-pvm-demo.sh",
        run_checked(
            [str(repo_root / "scripts" / "hosted-pvm-demo.sh")],
            env=env,
            cwd=repo_root,
        ),
    ),
]

write_cast(cast_path, steps, width=138, height=38)
subprocess.run(
    [
        "agg",
        "--theme",
        "github-dark",
        "--font-size",
        "14",
        "--cols",
        "138",
        "--rows",
        "38",
        "--idle-time-limit",
        "1.2",
        "--last-frame-duration",
        "2",
        str(cast_path),
        str(gif_path),
    ],
    cwd=repo_root,
    env=env,
    check=True,
)

print(f"generated cast: {cast_path}")
print(f"generated gif: {gif_path}")
PY
