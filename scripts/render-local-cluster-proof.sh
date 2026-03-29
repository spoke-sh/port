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
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time

repo_root = pathlib.Path(sys.argv[1]).resolve()
output_dir = pathlib.Path(sys.argv[2]).resolve()
cast_path = output_dir / "local-cluster-workflow.cast"
gif_path = output_dir / "ac-2.gif"
tmpdir = pathlib.Path(tempfile.mkdtemp(prefix="plc-proof-", dir="/tmp"))


def write_executable(path: pathlib.Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


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


def replace_once(text: str, needle: str, replacement: str) -> str:
    if needle not in text:
        raise RuntimeError(f"expected '{needle}' in examples/port.toml")
    return text.replace(needle, replacement, 1)


bin_dir = tmpdir / "bin"
runtime_root = tmpdir / "runtime"
guest_root = tmpdir / "guest-root"
config_path = tmpdir / "port-local-cluster.toml"
kernel_path = tmpdir / "standard-vmlinux"
guest_path = tmpdir / "standard-rootfs.ext4"
bin_dir.mkdir(parents=True, exist_ok=True)
runtime_root.mkdir(parents=True, exist_ok=True)
guest_root.mkdir(parents=True, exist_ok=True)
kernel_path.write_bytes(b"fake-standard-kernel")
guest_path.write_bytes(b"fake-standard-rootfs")

config_text = (repo_root / "examples" / "port.toml").read_text(encoding="utf-8")
config_text = replace_once(
    config_text,
    'artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux',
    str(kernel_path),
)
config_text = replace_once(
    config_text,
    'artifacts/guest/demo/x86_64/firecracker/standard/rootfs.ext4',
    str(guest_path),
)
config_path.write_text(config_text, encoding="utf-8")

guest_socket = runtime_root / "demo" / "guest-agent.sock"
forward_manifest = runtime_root / "demo" / "forwards" / "cluster-demo-api.json"
firecracker_pid_path = runtime_root / "demo" / "firecracker.pid"
steps: list[tuple[str, str]] = []
agent_process: subprocess.Popen[str] | None = None

write_executable(
    bin_dir / "firecracker",
    "#!/usr/bin/env bash\nsleep 30\n",
)

env = os.environ.copy()
env["PATH"] = f"{bin_dir}{os.pathsep}{env.get('PATH', '')}"
cargo_target_dir = pathlib.Path(env.get("CARGO_TARGET_DIR", repo_root / "target"))

try:
    subprocess.run(
        ["cargo", "build", "-q", "--bin", "port", "--bin", "port-guest-agent"],
        cwd=repo_root,
        env=env,
        check=True,
    )

    port_bin = cargo_target_dir / "debug" / "port"
    guest_agent_bin = cargo_target_dir / "debug" / "port-guest-agent"
    wait_for_runtime = shlex.quote(str(runtime_root / "demo"))
    quoted_agent = shlex.quote(str(guest_agent_bin))
    quoted_socket = shlex.quote(str(guest_socket))
    quoted_root = shlex.quote(str(guest_root))
    agent_process = subprocess.Popen(
        [
            "bash",
            "-lc",
            f"while [[ ! -d {wait_for_runtime} ]]; do sleep 0.05; done; exec {quoted_agent} --socket {quoted_socket} --root {quoted_root}",
        ],
        cwd=repo_root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    )

    show_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "cluster",
        "show",
        "--cluster",
        "demo",
    ]
    steps.append(
        (
            " ".join(shlex.quote(part) for part in show_command),
            run_checked(show_command, env=env, cwd=repo_root),
        )
    )

    up_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "cluster",
        "up",
        "--cluster",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(shlex.quote(part) for part in up_command),
            run_checked(up_command, env=env, cwd=repo_root),
        )
    )

    status_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "cluster",
        "status",
        "--cluster",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(shlex.quote(part) for part in status_command),
            run_checked(status_command, env=env, cwd=repo_root),
        )
    )

    kubeconfig_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "cluster",
        "kubeconfig",
        "--cluster",
        "demo",
        "--runtime-root",
        str(runtime_root),
        "--format",
        "json",
    ]
    steps.append(
        (
            " ".join(shlex.quote(part) for part in kubeconfig_command),
            run_checked(kubeconfig_command, env=env, cwd=repo_root),
        )
    )

    down_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "cluster",
        "down",
        "--cluster",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(shlex.quote(part) for part in down_command),
            run_checked(down_command, env=env, cwd=repo_root),
        )
    )

    write_cast(cast_path, steps, width=132, height=36)
    subprocess.run(
        [
            "agg",
            "--theme",
            "github-dark",
            "--font-size",
            "14",
            "--cols",
            "132",
            "--rows",
            "36",
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
finally:
    if forward_manifest.exists():
        try:
            manifest = json.loads(forward_manifest.read_text(encoding="utf-8"))
            os.kill(int(manifest["pid"]), signal.SIGTERM)
        except (OSError, ValueError, KeyError, json.JSONDecodeError):
            pass
    if firecracker_pid_path.exists():
        try:
            os.kill(int(firecracker_pid_path.read_text(encoding="utf-8").strip()), signal.SIGTERM)
        except (OSError, ValueError):
            pass
    if agent_process is not None:
        agent_process.terminate()
        try:
            agent_process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            agent_process.kill()
    shutil.rmtree(tmpdir, ignore_errors=True)
PY
