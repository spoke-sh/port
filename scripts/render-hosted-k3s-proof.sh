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
import socket
import subprocess
import sys
import tempfile
import textwrap
import threading
import time

repo_root = pathlib.Path(sys.argv[1]).resolve()
output_dir = pathlib.Path(sys.argv[2]).resolve()
cast_path = output_dir / "hosted-k3s-workflow.cast"
gif_path = output_dir / "ac-2.gif"
tmpdir = pathlib.Path(tempfile.mkdtemp(prefix="phk3s-", dir="/tmp"))


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
            if output:
                handle.write(
                    json.dumps([round(offset, 2), "o", output.replace("\n", "\r\n")])
                    + "\n"
                )
            offset += 1.35


def reserve_addr() -> str:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    host, port = listener.getsockname()
    listener.close()
    return f"{host}:{port}"


def wait_for_tcp(addr: str) -> None:
    host, port = addr.rsplit(":", 1)
    for _ in range(400):
        try:
            with socket.create_connection((host, int(port)), timeout=0.1):
                return
        except OSError:
            time.sleep(0.05)
    stderr_logs = []
    for log_path in sorted(tmpdir.glob("*.stderr.log")):
        log_body = log_path.read_text(encoding="utf-8").strip()
        if log_body:
            stderr_logs.append(f"--- {log_path.name} ---\n{log_body}")
    details = "\n".join(stderr_logs)
    if details:
        raise RuntimeError(
            f"timed out waiting for tcp listener at {addr}\n{details}"
        )
    raise RuntimeError(f"timed out waiting for tcp listener at {addr}")


def machine_paths(runtime_root: pathlib.Path, machine_name: str) -> dict[str, pathlib.Path]:
    runtime_dir = runtime_root / machine_name
    return {
        "runtime_dir": runtime_dir,
        "manifest_path": runtime_dir / "manifest.json",
        "pid_path": runtime_dir / "firecracker.pid",
        "vsock_path": runtime_dir / "guest.vsock",
    }


def spawn_exec_sequence_server(
    paths: dict[str, pathlib.Path], expected: list[tuple[list[str], str]]
) -> threading.Thread:
    def worker() -> None:
        manifest_path = paths["manifest_path"]
        runtime_dir = paths["runtime_dir"]
        vsock_path = paths["vsock_path"]

        for _ in range(1000):
            if manifest_path.exists():
                break
            time.sleep(0.01)
        if not manifest_path.exists():
            raise AssertionError(
                f"machine manifest should exist before binding guest transport at {manifest_path}"
            )

        runtime_dir.mkdir(parents=True, exist_ok=True)
        try:
            vsock_path.unlink()
        except FileNotFoundError:
            pass

        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(str(vsock_path))
        listener.listen(1)

        try:
            for expected_command, stdout in expected:
                conn, _ = listener.accept()
                with conn:
                    reader = conn.makefile("r", encoding="utf-8")
                    handshake = reader.readline()
                    if not handshake.startswith("CONNECT "):
                        raise AssertionError(
                            f"unexpected guest transport handshake: {handshake!r}"
                        )
                    conn.sendall(b"OK\n")
                    request = json.loads(reader.readline())
                    if request["operation"]["type"] != "exec":
                        raise AssertionError(
                            f"unexpected hosted guest operation: {request['operation']!r}"
                        )
                    if request["operation"]["command"] != expected_command:
                        raise AssertionError(
                            f"unexpected command {request['operation']['command']!r}; expected {expected_command!r}"
                        )
                    response = {
                        "status": "completed",
                        "id": request["id"],
                        "exit_code": 0,
                        "result": {
                            "type": "exec",
                            "stdout": stdout,
                            "stderr": "",
                        },
                    }
                    conn.sendall((json.dumps(response) + "\n").encode("utf-8"))
        finally:
            listener.close()
            try:
                vsock_path.unlink()
            except FileNotFoundError:
                pass

    thread = threading.Thread(target=worker, daemon=True)
    thread.start()
    return thread


def filter_lines(output: str, needles: list[str]) -> str:
    filtered = [
        line for line in output.splitlines() if any(needle in line for needle in needles)
    ]
    if not filtered:
        return output
    return "\n".join(filtered) + "\n"


control_addr = reserve_addr()
generic_addr = reserve_addr()
aws_addr = reserve_addr()

bin_dir = tmpdir / "bin"
bin_dir.mkdir(parents=True, exist_ok=True)
runtime_root = tmpdir / "rt"
generic_runtime_root = runtime_root / "g"
aws_runtime_root = runtime_root / "a"
generic_runtime_root.mkdir(parents=True, exist_ok=True)
aws_runtime_root.mkdir(parents=True, exist_ok=True)

kernel_path = tmpdir / "demo-vmlinux"
guest_path = tmpdir / "demo-rootfs.ext4"
kernel_path.write_bytes(b"fake-kernel")
guest_path.write_bytes(b"fake-rootfs")

config_path = tmpdir / "port-k3s.toml"
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

        [control_planes.demo]
        endpoint = "http://{control_addr}"
        audience = "port-hosted-demo"

        [control_planes.demo.auth]
        scheme = "bearer"
        header = "authorization"

        [control_planes.demo.auth.source]
        kind = "env"
        variable = "PORT_DEMO_TOKEN"

        [hosts.generic-linux]
        platform = "linux"
        provider = "generic-linux"

        [hosts.generic-linux.connection]
        mode = "hosted-control-plane"
        control_plane = "demo"

        [hosts.generic-linux.firecracker]
        local_launch = false
        notes = ["Generic Linux hosted K3s server node for the first stateless cluster slice."]

        [hosts.aws-linux]
        platform = "linux"
        provider = "aws"

        [hosts.aws-linux.connection]
        mode = "hosted-control-plane"
        control_plane = "demo"

        [hosts.aws-linux.firecracker]
        local_launch = false
        notes = ["AWS hosted K3s worker node for the first stateless cluster slice."]

        [nodes.generic-linux-node]
        host = "generic-linux"
        runtime_root = "{generic_runtime_root}"
        notes = ["Generic Linux server node for the first hosted K3s workflow proof."]

        [nodes.generic-linux-node.capabilities]
        providers = ["generic-linux"]
        platforms = ["linux"]
        substrates = ["firecracker"]
        architectures = ["x86_64"]
        protection_modes = ["standard"]

        [nodes.aws-linux-node]
        host = "aws-linux"
        runtime_root = "{aws_runtime_root}"
        notes = ["AWS worker node for the first hosted K3s workflow proof."]

        [nodes.aws-linux-node.capabilities]
        providers = ["aws"]
        platforms = ["linux"]
        substrates = ["firecracker"]
        architectures = ["x86_64"]
        protection_modes = ["standard"]

        [host_groups.remote-linux]
        placement = "explicit-membership"
        scheduler = "deterministic-first-fit"
        nodes = ["generic-linux-node", "aws-linux-node"]
        notes = ["One host group keeps the first hosted K3s workflow explicit and bounded."]

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

        [machines.cloud-aws]
        host = "aws-linux"
        kernel = "demo-kernel"
        guest_image = "demo-guest"
        substrate = "firecracker"
        protection_mode = "standard"
        architecture = "native"
        vcpu_count = 2
        memory_mib = 512
        kernel_args = "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"
        rootfs_read_only = false

        [machines.cloud-aws.guest]
        vsock_cid = 61
        control_port = 7000
        console_log = "runtime/cloud-aws/console.log"

        [k3s_clusters.demo]
        control_plane = "demo"
        host_group = "remote-linux"
        server_machine = "cloud-generic"
        worker_machines = ["cloud-aws"]
        version = "v1.32.0+k3s1"
        server_args = ["--disable=traefik"]
        worker_args = ["--node-label=role=worker"]
        """
    ).strip()
    + "\n",
    encoding="utf-8",
)

port_bin = repo_root / "target" / "debug" / "port"
write_executable(bin_dir / "port", f"#!/usr/bin/env bash\nexec '{port_bin}' \"$@\"\n")
write_executable(bin_dir / "firecracker", "#!/usr/bin/env bash\nsleep 30\n")

env = os.environ.copy()
env["PATH"] = f"{bin_dir}{os.pathsep}{env.get('PATH', '')}"
env["PORT_DEMO_TOKEN"] = "demo-token"

processes: list[tuple[subprocess.Popen[str], object, object]] = []
threads: list[threading.Thread] = []
steps: list[tuple[str, str]] = [("export PORT_DEMO_TOKEN=demo-token", "")]

try:
    subprocess.run(
        ["cargo", "build", "-q", "--bin", "port"],
        cwd=repo_root,
        env=env,
        check=True,
    )

    for name, argv in [
        (
            "control-plane",
            [
                "port",
                "--config",
                str(config_path),
                "control-plane",
                "serve",
                "--control-plane",
                "demo",
                "--bind",
                control_addr,
            ],
        ),
        (
            "generic-node-agent",
            [
                "port",
                "--config",
                str(config_path),
                "node-agent",
                "serve",
                "--node",
                "generic-linux-node",
                "--bind",
                generic_addr,
                "--token",
                "node-secret",
            ],
        ),
        (
            "aws-node-agent",
            [
                "port",
                "--config",
                str(config_path),
                "node-agent",
                "serve",
                "--node",
                "aws-linux-node",
                "--bind",
                aws_addr,
                "--token",
                "node-secret",
            ],
        ),
    ]:
        stdout_handle = (tmpdir / f"{name}.stdout.log").open("w", encoding="utf-8")
        stderr_handle = (tmpdir / f"{name}.stderr.log").open("w", encoding="utf-8")
        process = subprocess.Popen(
            argv,
            cwd=repo_root,
            env=env,
            stdout=stdout_handle,
            stderr=stderr_handle,
            text=True,
        )
        processes.append((process, stdout_handle, stderr_handle))

    wait_for_tcp(control_addr)
    wait_for_tcp(generic_addr)
    wait_for_tcp(aws_addr)

    generic_paths = machine_paths(generic_runtime_root, "cloud-generic")
    aws_paths = machine_paths(aws_runtime_root, "cloud-aws")
    threads.append(
        spawn_exec_sequence_server(
            generic_paths,
            [
                (
                    [
                        "/bin/sh",
                        "-lc",
                        "set -eu; mkdir -p /run/port /var/log /etc/rancher/k3s /var/lib/rancher/k3s /tmp; pid_path='/run/port/k3s-server.pid'; if [ -s \"$pid_path\" ] && kill -0 \"$(cat \"$pid_path\")\" 2>/dev/null; then echo k3s-already-running; exit 0; fi; k3s_bin=\"$(command -v k3s 2>/dev/null || true)\"; if [ -z \"$k3s_bin\" ] && [ -x /usr/bin/k3s ]; then k3s_bin=/usr/bin/k3s; fi; if [ -n \"$k3s_bin\" ]; then ( trap '' HUP INT TERM; exec \"$k3s_bin\" server --disable=traefik ) >'/var/log/k3s-server.log' 2>&1 < /dev/null & child=$!; printf '%s\\n' \"$child\" > \"$pid_path\"; echo \"k3s-launched:$child\"; exit 0; fi; installer=/tmp/install-k3s.sh; rm -f \"$installer\"; if command -v curl >/dev/null 2>&1; then curl -fsSL https://get.k3s.io -o \"$installer\"; elif command -v wget >/dev/null 2>&1; then wget -qO \"$installer\" https://get.k3s.io; elif command -v busybox >/dev/null 2>&1; then busybox wget -qO \"$installer\" https://get.k3s.io; else echo 'no supported fetcher found for get.k3s.io' >&2; exit 127; fi; chmod +x \"$installer\"; INSTALL_K3S_VERSION='v1.32.0+k3s1' INSTALL_K3S_EXEC='server --disable=traefik' sh \"$installer\"",
                    ],
                    "server bootstrapped\n",
                ),
                (
                    ["/bin/sh", "-lc", "attempt=0; while [ \"$attempt\" -lt 120 ]; do if [ -s /var/lib/rancher/k3s/server/node-token ]; then cat /var/lib/rancher/k3s/server/node-token; exit 0; fi; attempt=$((attempt + 1)); sleep 1; done; echo 'timed out waiting for /var/lib/rancher/k3s/server/node-token' >&2; exit 1"],
                    "demo-join-token\n",
                ),
                (
                    ["/bin/sh", "-lc", "cat /etc/rancher/k3s/k3s.yaml"],
                    "apiVersion: v1\nclusters:\n- cluster:\n    server: https://cloud-generic:6443\n",
                ),
                (
                    ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "NAME           STATUS   ROLES                  AGE   VERSION\ncloud-generic   Ready    control-plane,master   1m    v1.32.0+k3s1\ncloud-aws       Ready    <none>                 1m    v1.32.0+k3s1\n",
                ),
            ],
        )
    )
    threads.append(
        spawn_exec_sequence_server(
            aws_paths,
            [
                (
                    [
                        "/bin/sh",
                        "-lc",
                        "set -eu; mkdir -p /run/port /var/log /etc/rancher/k3s /var/lib/rancher/k3s /tmp; pid_path='/run/port/k3s-agent.pid'; if [ -s \"$pid_path\" ] && kill -0 \"$(cat \"$pid_path\")\" 2>/dev/null; then echo k3s-already-running; exit 0; fi; k3s_bin=\"$(command -v k3s 2>/dev/null || true)\"; if [ -z \"$k3s_bin\" ] && [ -x /usr/bin/k3s ]; then k3s_bin=/usr/bin/k3s; fi; if [ -n \"$k3s_bin\" ]; then ( trap '' HUP INT TERM; exec \"$k3s_bin\" agent --server 'https://cloud-generic:6443' --token 'demo-join-token' --node-label=role=worker ) >'/var/log/k3s-agent.log' 2>&1 < /dev/null & child=$!; printf '%s\\n' \"$child\" > \"$pid_path\"; echo \"k3s-launched:$child\"; exit 0; fi; installer=/tmp/install-k3s.sh; rm -f \"$installer\"; if command -v curl >/dev/null 2>&1; then curl -fsSL https://get.k3s.io -o \"$installer\"; elif command -v wget >/dev/null 2>&1; then wget -qO \"$installer\" https://get.k3s.io; elif command -v busybox >/dev/null 2>&1; then busybox wget -qO \"$installer\" https://get.k3s.io; else echo 'no supported fetcher found for get.k3s.io' >&2; exit 127; fi; chmod +x \"$installer\"; INSTALL_K3S_VERSION='v1.32.0+k3s1' K3S_URL='https://cloud-generic:6443' K3S_TOKEN='demo-join-token' INSTALL_K3S_EXEC='agent --node-label=role=worker' sh \"$installer\"",
                    ],
                    "worker joined\n",
                )
            ],
        )
    )

    launch_server = [
        "port",
        "--config",
        str(config_path),
        "machine",
        "launch",
        "--machine",
        "cloud-generic",
    ]
    steps.append((" ".join(launch_server), run_checked(launch_server, env=env, cwd=repo_root)))

    launch_worker = [
        "port",
        "--config",
        str(config_path),
        "machine",
        "launch",
        "--machine",
        "cloud-aws",
    ]
    steps.append((" ".join(launch_worker), run_checked(launch_worker, env=env, cwd=repo_root)))

    status_server = [
        "port",
        "--config",
        str(config_path),
        "machine",
        "status",
        "--machine",
        "cloud-generic",
    ]
    steps.append(
        (
            " ".join(status_server),
            filter_lines(
                run_checked(status_server, env=env, cwd=repo_root),
                [
                    "machine:",
                    "state:",
                    "control plane:",
                    "node:",
                    "host groups:",
                    "launch route:",
                    "status route:",
                    "guest route:",
                    "detail:",
                ],
            ),
        )
    )

    server_install = [
        "port",
        "--config",
        str(config_path),
        "guest",
        "exec",
        "--machine",
        "cloud-generic",
        "--",
        "/bin/sh",
        "-lc",
        "set -eu; mkdir -p /run/port /var/log /etc/rancher/k3s /var/lib/rancher/k3s /tmp; pid_path='/run/port/k3s-server.pid'; if [ -s \"$pid_path\" ] && kill -0 \"$(cat \"$pid_path\")\" 2>/dev/null; then echo k3s-already-running; exit 0; fi; k3s_bin=\"$(command -v k3s 2>/dev/null || true)\"; if [ -z \"$k3s_bin\" ] && [ -x /usr/bin/k3s ]; then k3s_bin=/usr/bin/k3s; fi; if [ -n \"$k3s_bin\" ]; then ( trap '' HUP INT TERM; exec \"$k3s_bin\" server --disable=traefik ) >'/var/log/k3s-server.log' 2>&1 < /dev/null & child=$!; printf '%s\\n' \"$child\" > \"$pid_path\"; echo \"k3s-launched:$child\"; exit 0; fi; installer=/tmp/install-k3s.sh; rm -f \"$installer\"; if command -v curl >/dev/null 2>&1; then curl -fsSL https://get.k3s.io -o \"$installer\"; elif command -v wget >/dev/null 2>&1; then wget -qO \"$installer\" https://get.k3s.io; elif command -v busybox >/dev/null 2>&1; then busybox wget -qO \"$installer\" https://get.k3s.io; else echo 'no supported fetcher found for get.k3s.io' >&2; exit 127; fi; chmod +x \"$installer\"; INSTALL_K3S_VERSION='v1.32.0+k3s1' INSTALL_K3S_EXEC='server --disable=traefik' sh \"$installer\"",
    ]
    steps.append((" ".join(server_install), run_checked(server_install, env=env, cwd=repo_root)))

    token_command = [
        "port",
        "--config",
        str(config_path),
        "guest",
        "exec",
        "--machine",
        "cloud-generic",
        "--",
        "/bin/sh",
        "-lc",
        "attempt=0; while [ \"$attempt\" -lt 120 ]; do if [ -s /var/lib/rancher/k3s/server/node-token ]; then cat /var/lib/rancher/k3s/server/node-token; exit 0; fi; attempt=$((attempt + 1)); sleep 1; done; echo 'timed out waiting for /var/lib/rancher/k3s/server/node-token' >&2; exit 1",
    ]
    token_output = run_checked(token_command, env=env, cwd=repo_root)
    steps.append((" ".join(token_command), token_output))
    join_token = token_output.strip()

    worker_install = [
        "port",
        "--config",
        str(config_path),
        "guest",
        "exec",
        "--machine",
        "cloud-aws",
        "--",
        "/bin/sh",
        "-lc",
        f"set -eu; mkdir -p /run/port /var/log /etc/rancher/k3s /var/lib/rancher/k3s /tmp; pid_path='/run/port/k3s-agent.pid'; if [ -s \"$pid_path\" ] && kill -0 \"$(cat \"$pid_path\")\" 2>/dev/null; then echo k3s-already-running; exit 0; fi; k3s_bin=\"$(command -v k3s 2>/dev/null || true)\"; if [ -z \"$k3s_bin\" ] && [ -x /usr/bin/k3s ]; then k3s_bin=/usr/bin/k3s; fi; if [ -n \"$k3s_bin\" ]; then ( trap '' HUP INT TERM; exec \"$k3s_bin\" agent --server 'https://cloud-generic:6443' --token '{join_token}' --node-label=role=worker ) >'/var/log/k3s-agent.log' 2>&1 < /dev/null & child=$!; printf '%s\\n' \"$child\" > \"$pid_path\"; echo \"k3s-launched:$child\"; exit 0; fi; installer=/tmp/install-k3s.sh; rm -f \"$installer\"; if command -v curl >/dev/null 2>&1; then curl -fsSL https://get.k3s.io -o \"$installer\"; elif command -v wget >/dev/null 2>&1; then wget -qO \"$installer\" https://get.k3s.io; elif command -v busybox >/dev/null 2>&1; then busybox wget -qO \"$installer\" https://get.k3s.io; else echo 'no supported fetcher found for get.k3s.io' >&2; exit 127; fi; chmod +x \"$installer\"; INSTALL_K3S_VERSION='v1.32.0+k3s1' K3S_URL='https://cloud-generic:6443' K3S_TOKEN='{join_token}' INSTALL_K3S_EXEC='agent --node-label=role=worker' sh \"$installer\"",
    ]
    steps.append((" ".join(worker_install), run_checked(worker_install, env=env, cwd=repo_root)))

    kubeconfig_command = [
        "port",
        "--config",
        str(config_path),
        "guest",
        "exec",
        "--machine",
        "cloud-generic",
        "--",
        "/bin/sh",
        "-lc",
        "cat /etc/rancher/k3s/k3s.yaml",
    ]
    steps.append((" ".join(kubeconfig_command), run_checked(kubeconfig_command, env=env, cwd=repo_root)))

    nodes_command = [
        "port",
        "--config",
        str(config_path),
        "guest",
        "exec",
        "--machine",
        "cloud-generic",
        "--",
        "/bin/sh",
        "-lc",
        "k3s kubectl get nodes -o wide",
    ]
    steps.append((" ".join(nodes_command), run_checked(nodes_command, env=env, cwd=repo_root)))

    stop_worker = [
        "port",
        "--config",
        str(config_path),
        "machine",
        "stop",
        "--machine",
        "cloud-aws",
    ]
    steps.append((" ".join(stop_worker), run_checked(stop_worker, env=env, cwd=repo_root)))

    stop_server = [
        "port",
        "--config",
        str(config_path),
        "machine",
        "stop",
        "--machine",
        "cloud-generic",
    ]
    steps.append((" ".join(stop_server), run_checked(stop_server, env=env, cwd=repo_root)))

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
    for process, stdout_handle, stderr_handle in processes:
        if process.poll() is None:
            process.send_signal(signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        stdout_handle.close()
        stderr_handle.close()
    for thread in threads:
        thread.join(timeout=5)
    for pid_path in [generic_paths["pid_path"] if "generic_paths" in locals() else None, aws_paths["pid_path"] if "aws_paths" in locals() else None]:
        if pid_path is None or not pid_path.exists():
            continue
        try:
            os.kill(int(pid_path.read_text(encoding="utf-8").strip()), signal.SIGTERM)
        except (OSError, ValueError):
            pass
    shutil.rmtree(tmpdir, ignore_errors=True)
PY
