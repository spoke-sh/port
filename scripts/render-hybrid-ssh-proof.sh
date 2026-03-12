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
cast_path = output_dir / "hybrid-ssh-workflow.cast"
gif_path = output_dir / "ac-2.gif"
tmpdir = pathlib.Path(tempfile.mkdtemp(prefix="port-hybrid-ssh-proof-"))


def write_executable(path: pathlib.Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def run_checked(
    argv: list[str],
    *,
    env: dict[str, str],
    cwd: pathlib.Path,
    shell: bool = False,
) -> str:
    if shell:
        result = subprocess.run(
            ["bash", "-lc", argv[0]],
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
            check=True,
        )
    else:
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


runtime_root = tmpdir / "runtime"
bin_dir = tmpdir / "bin"
runtime_root.mkdir(parents=True, exist_ok=True)
bin_dir.mkdir(parents=True, exist_ok=True)

config_path = tmpdir / "port-ssh-proof.toml"
config_path.write_text(
    textwrap.dedent(
        """
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
        path = "artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux"

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
        path = "artifacts/guest/demo/x86_64/firecracker/standard/rootfs.ext4"

        [artifacts.guest_images.demo-guest.variants.selector]
        architecture = "x86_64"
        substrate = "firecracker"
        protection_mode = "standard"

        [hosts.generic-linux]
        platform = "linux"
        provider = "generic-linux"

        [hosts.generic-linux.connection]
        mode = "ssh"
        destination = "builder.example.internal"
        user = "ubuntu"
        port = 2222

        [hosts.generic-linux.firecracker]
        local_launch = false
        notes = ["Remote Linux host must already expose Port, Firecracker, and the selected artifact paths."]

        [machines.cloud-generic]
        host = "generic-linux"
        kernel = "demo-kernel"
        guest_image = "demo-guest"
        substrate = "firecracker"
        protection_mode = "standard"
        architecture = "native"
        vcpu_count = 2
        memory_mib = 512
        kernel_args = "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"
        rootfs_read_only = false

        [machines.cloud-generic.guest]
        vsock_cid = 60
        control_port = 7000
        console_log = "runtime/cloud-generic/console.log"
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
write_executable(
    bin_dir / "port",
    f"#!/usr/bin/env bash\nexec '{port_bin}' \"$@\"\n",
)
write_executable(
    bin_dir / "ssh",
    textwrap.dedent(
        """\
        #!/usr/bin/env bash
        set -euo pipefail

        while [[ $# -gt 0 ]]; do
          case "$1" in
            -p|-o)
              shift 2
              ;;
            --)
              shift
              break
              ;;
            -*)
              echo "unexpected ssh option: $1" >&2
              exit 64
              ;;
            *)
              shift
              break
              ;;
          esac
        done

        if [[ $# -eq 0 ]]; then
          echo "missing remote command" >&2
          exit 64
        fi

        exec "$@"
        """
    ),
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

    doctor_command = (
        f"{port_bin} --config {config_path} doctor | "
        "rg 'host:generic-linux:ssh-(auth|bootstrap)|note:'"
    )
    steps.append(
        (
            doctor_command,
            run_checked([doctor_command], env=env, cwd=repo_root, shell=True),
        )
    )

    launch_command = [
        str(port_bin),
        "--config",
        str(config_path),
        "machine",
        "launch",
        "--machine",
        "cloud-generic",
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
        "cloud-generic",
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
        "cloud-generic",
        "--runtime-root",
        str(runtime_root),
    ]
    steps.append(
        (
            " ".join(stop_command),
            run_checked(stop_command, env=env, cwd=repo_root),
        )
    )

    write_cast(cast_path, steps, width=120, height=32)
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
            "32",
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
    pid_path = runtime_root / "cloud-generic" / "firecracker.pid"
    if pid_path.exists():
        try:
            pid = int(pid_path.read_text(encoding="utf-8").strip())
            os.kill(pid, signal.SIGTERM)
        except (OSError, ValueError):
            pass
    shutil.rmtree(tmpdir, ignore_errors=True)
PY
