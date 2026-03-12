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
import shutil
import signal
import subprocess
import sys
import tempfile
import textwrap
import time

repo_root = pathlib.Path(sys.argv[1]).resolve()
output_dir = pathlib.Path(sys.argv[2]).resolve()
cast_path = output_dir / "attached-volume-workflow.cast"
gif_path = output_dir / "ac-2.gif"
tmpdir = pathlib.Path(tempfile.mkdtemp(prefix="port-attached-volume-proof-"))


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


def filter_lines(output: str, needles: list[str]) -> str:
    filtered = [
        line
        for line in output.splitlines()
        if any(needle in line for needle in needles)
    ]
    if not filtered:
        return output
    return "\n".join(filtered) + "\n"


runtime_root = tmpdir / "runtime"
bin_dir = tmpdir / "bin"
runtime_root.mkdir(parents=True, exist_ok=True)
bin_dir.mkdir(parents=True, exist_ok=True)

kernel_path = tmpdir / "demo-vmlinux"
guest_path = tmpdir / "demo-rootfs.ext4"
volume_path = tmpdir / "demo-data.ext4"
kernel_path.write_bytes(b"fake-kernel")
guest_path.write_bytes(b"fake-rootfs")
volume_path.write_bytes(b"fake-attached-volume")

config_path = tmpdir / "port-attached-volume.toml"
config_path.write_text(
    textwrap.dedent(
        f"""
        [artifacts.kernels.demo-kernel]
        build = "port artifacts build --artifact demo-kernel"
        validate = "port artifacts validate --artifact demo-kernel"

        [artifacts.kernels.demo-kernel.reference]
        registry = "demo-fs"
        repository = "port/demo-kernel"
        version = "v1"

        [artifacts.kernels.demo-kernel.distribution]
        cache_root = ".port/cache"

        [artifacts.kernels.demo-kernel.distribution.push]
        backend = "file-system"
        root = "artifact-store/demo-fs"

        [artifacts.kernels.demo-kernel.distribution.pull]
        backend = "file-system"
        root = "artifact-store/demo-fs"

        [[artifacts.kernels.demo-kernel.variants]]
        path = "{kernel_path}"

        [artifacts.kernels.demo-kernel.variants.selector]
        architecture = "x86_64"
        substrate = "firecracker"
        protection_mode = "standard"

        [artifacts.guest_images.demo-guest]
        build = "port artifacts build --artifact demo-guest"
        validate = "port artifacts validate --artifact demo-guest"

        [artifacts.guest_images.demo-guest.reference]
        registry = "demo-fs"
        repository = "port/demo-guest"
        version = "v1"

        [artifacts.guest_images.demo-guest.distribution]
        cache_root = ".port/cache"

        [artifacts.guest_images.demo-guest.distribution.push]
        backend = "file-system"
        root = "artifact-store/demo-fs"

        [artifacts.guest_images.demo-guest.distribution.pull]
        backend = "file-system"
        root = "artifact-store/demo-fs"

        [[artifacts.guest_images.demo-guest.variants]]
        path = "{guest_path}"

        [artifacts.guest_images.demo-guest.variants.selector]
        architecture = "x86_64"
        substrate = "firecracker"
        protection_mode = "standard"

        [hosts.local]
        platform = "linux"
        provider = "local"

        [hosts.local.connection]
        mode = "local"

        [hosts.local.firecracker]
        local_launch = true
        notes = ["Attached-volume proof uses the direct local Firecracker lane."]

        [machines.demo]
        host = "local"
        kernel = "demo-kernel"
        guest_image = "demo-guest"
        substrate = "firecracker"
        protection_mode = "standard"
        architecture = "native"
        vcpu_count = 2
        memory_mib = 512
        kernel_args = "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"
        rootfs_read_only = false

        [machines.demo.guest]
        vsock_cid = 52
        control_port = 7000
        console_log = "runtime/demo/console.log"

        [[machines.demo.volumes]]
        name = "data"
        backend = "host-file"
        persistence = "persistent"
        path = "{volume_path}"
        """
    ).strip()
    + "\n",
    encoding="utf-8",
)

port_bin = repo_root / "target" / "debug" / "port"

write_executable(
    bin_dir / "firecracker",
    "#!/usr/bin/env bash\nsleep 30\n",
)

env = os.environ.copy()
env["PATH"] = f"{bin_dir}{os.pathsep}{env.get('PATH', '')}"

steps: list[tuple[str, str]] = []

try:
    subprocess.run(
        ["cargo", "build", "-q", "--bin", "port"],
        cwd=repo_root,
        env=env,
        check=True,
    )

    doctor_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "doctor",
    ]
    doctor_output = run_checked(doctor_command, env=env, cwd=repo_root)
    steps.append(
        (
            " ".join(doctor_command)
            + " | rg 'machine:demo:volume:data:attached-volume|host-platform|note:'",
            filter_lines(
                doctor_output,
                [
                    "machine:demo:volume:data:attached-volume",
                    "host-platform",
                    "note:",
                ],
            ),
        )
    )

    launch_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "machine",
        "launch",
        "--machine",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(launch_command),
            run_checked(launch_command, env=env, cwd=repo_root),
        )
    )

    status_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "machine",
        "status",
        "--machine",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(status_command),
            run_checked(status_command, env=env, cwd=repo_root),
        )
    )

    stop_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "machine",
        "stop",
        "--machine",
        "demo",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(stop_command),
            run_checked(stop_command, env=env, cwd=repo_root),
        )
    )

    write_cast(cast_path, steps, width=120, height=34)
    subprocess.run(
        [
            "agg",
            "--theme",
            "github-dark",
            "--font-size",
            "14",
            "--cols",
            "120",
            "--rows",
            "34",
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
    pid_path = runtime_root / "demo" / "firecracker.pid"
    if pid_path.exists():
        try:
            pid = int(pid_path.read_text(encoding="utf-8").strip())
            os.kill(pid, signal.SIGTERM)
        except (OSError, ValueError):
            pass
    shutil.rmtree(tmpdir, ignore_errors=True)
PY
