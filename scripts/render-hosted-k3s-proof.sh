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


def cargo_target_root() -> pathlib.Path:
    configured = os.environ.get("CARGO_TARGET_DIR")
    if not configured:
        return repo_root / "target"
    candidate = pathlib.Path(configured)
    if candidate.is_absolute():
        return candidate
    return (repo_root / candidate).resolve()


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
    )
    if result.returncode != 0:
        raise RuntimeError(
            "command failed: {}\nstdout:\n{}\nstderr:\n{}".format(
                " ".join(argv),
                result.stdout,
                result.stderr,
            )
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


def running_managed_service_status(name: str) -> dict[str, object]:
    return {
        "result": "status",
        "name": name,
        "kind": "service",
        "state": "running",
        "restart_count": 0,
        "pid": 4242,
        "exit_code": None,
        "health_state": "unknown",
        "stdout_path": f"/run/port/services/{name}.stdout.log",
        "stderr_path": f"/run/port/services/{name}.stderr.log",
        "detail": "managed process is running",
    }


def k3s_service_command(
    role: str,
    args: list[str],
    *,
    bootstrap_flag: str | None = None,
    server_url: str | None = None,
    join_token: str | None = None,
) -> list[str]:
    command = ["/usr/bin/k3s", role]
    if bootstrap_flag is not None:
        command.append(bootstrap_flag)
    if server_url is not None:
        command.extend(["--server", server_url])
    if join_token is not None:
        command.extend(["--token", join_token])
    command.extend(args)
    return command


def k3s_service_healthcheck_command(role: str) -> list[str]:
    k3s = "/usr/bin/k3s"
    if role == "server":
        shell = (
            f"{k3s} crictl info >/dev/null 2>&1 && "
            f"{k3s} kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml "
            "--request-timeout=10s get --raw=/readyz >/dev/null 2>&1"
        )
    else:
        shell = (
            f"{k3s} crictl info >/dev/null 2>&1 && "
            f"{k3s} kubectl --kubeconfig /var/lib/rancher/k3s/agent/kubelet.kubeconfig "
            "--request-timeout=10s get --raw=/readyz >/dev/null 2>&1"
        )
    return ["/bin/sh", "-lc", shell]


def k3s_service_policy(role: str) -> dict[str, object]:
    return {
        "restart": "always",
        "healthcheck": {
            "policy": "command",
            "command": k3s_service_healthcheck_command(role),
        },
    }


def spawn_exec_sequence_server(
    paths: dict[str, pathlib.Path], expected: list[dict[str, object]]
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
            for expected_operation in expected:
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
                    operation = request["operation"]
                    expected_type = expected_operation["type"]
                    if expected_type == "exec":
                        if operation["type"] != "exec":
                            raise AssertionError(
                                f"unexpected hosted guest operation: {operation!r}"
                            )
                        expected_command = expected_operation["command"]
                        if operation["command"] != expected_command:
                            raise AssertionError(
                                f"unexpected command {operation['command']!r}; expected {expected_command!r}"
                            )
                        response = {
                            "status": "completed",
                            "id": request["id"],
                            "exit_code": 0,
                            "result": {
                                "type": "exec",
                                "stdout": expected_operation["stdout"],
                                "stderr": "",
                            },
                        }
                    elif expected_type == "managed-service-start":
                        if operation["type"] != "managed-service":
                            raise AssertionError(
                                f"unexpected hosted guest operation: {operation!r}"
                            )
                        service_operation = operation["operation"]
                        if service_operation["verb"] != "start":
                            raise AssertionError(
                                f"unexpected managed-service verb: {service_operation!r}"
                            )
                        if service_operation["name"] != expected_operation["name"]:
                            raise AssertionError(
                                f"unexpected service name {service_operation['name']!r}; expected {expected_operation['name']!r}"
                            )
                        if service_operation["kind"] != "service":
                            raise AssertionError(
                                f"unexpected managed-service kind: {service_operation!r}"
                            )
                        if service_operation["command"] != expected_operation["command"]:
                            raise AssertionError(
                                f"unexpected managed-service command {service_operation['command']!r}; expected {expected_operation['command']!r}"
                            )
                        if service_operation["env"] != {}:
                            raise AssertionError(
                                f"expected empty managed-service env, got {service_operation['env']!r}"
                            )
                        if service_operation["cwd"] is not None:
                            raise AssertionError(
                                f"expected managed-service cwd to be null, got {service_operation['cwd']!r}"
                            )
                        if service_operation["policy"] != expected_operation["policy"]:
                            raise AssertionError(
                                f"unexpected managed-service policy {service_operation['policy']!r}; expected {expected_operation['policy']!r}"
                            )
                        response = {
                            "status": "completed",
                            "id": request["id"],
                            "exit_code": 0,
                            "result": {
                                "type": "managed-service",
                                **running_managed_service_status(
                                    str(expected_operation["name"])
                                ),
                            },
                        }
                    elif expected_type == "managed-service-status":
                        if operation["type"] != "managed-service":
                            raise AssertionError(
                                f"unexpected hosted guest operation: {operation!r}"
                            )
                        service_operation = operation["operation"]
                        if service_operation["verb"] != "status":
                            raise AssertionError(
                                f"unexpected managed-service verb: {service_operation!r}"
                            )
                        if service_operation["name"] != expected_operation["name"]:
                            raise AssertionError(
                                f"unexpected service name {service_operation['name']!r}; expected {expected_operation['name']!r}"
                            )
                        response = {
                            "status": "completed",
                            "id": request["id"],
                            "exit_code": 0,
                            "result": {
                                "type": "managed-service",
                                **running_managed_service_status(
                                    str(expected_operation["name"])
                                ),
                            },
                        }
                    elif expected_type == "managed-service-list":
                        if operation["type"] != "managed-service":
                            raise AssertionError(
                                f"unexpected hosted guest operation: {operation!r}"
                            )
                        service_operation = operation["operation"]
                        if service_operation["verb"] != "list":
                            raise AssertionError(
                                f"unexpected managed-service verb: {service_operation!r}"
                            )
                        response = {
                            "status": "completed",
                            "id": request["id"],
                            "exit_code": 0,
                            "result": {
                                "type": "managed-service",
                                "result": "list",
                                "services": expected_operation["services"],
                            },
                        }
                    else:
                        raise AssertionError(
                            f"unsupported expected operation type: {expected_type!r}"
                        )
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


api_endpoint = "https://demo-k3s.internal:6443"
control_plane_name = f"proofdemo{os.getpid()}"


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

        [control_planes.{control_plane_name}]
        endpoint = "http://{control_addr}"
        audience = "port-hosted-demo"

        [control_planes.{control_plane_name}.auth]
        scheme = "bearer"
        header = "authorization"

        [control_planes.{control_plane_name}.auth.source]
        kind = "env"
        variable = "PORT_DEMO_TOKEN"

        [hosts.generic-linux]
        platform = "linux"
        provider = "generic-linux"

        [hosts.generic-linux.connection]
        mode = "hosted-control-plane"
        control_plane = "{control_plane_name}"

        [hosts.generic-linux.firecracker]
        local_launch = false
        notes = ["Generic Linux hosted K3s server node for the first stateless cluster slice."]

        [hosts.aws-linux]
        platform = "linux"
        provider = "aws"

        [hosts.aws-linux.connection]
        mode = "hosted-control-plane"
        control_plane = "{control_plane_name}"

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

        [machines.cloud-generic.network]
        enabled = false

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

        [machines.cloud-aws.network]
        enabled = false

        [k3s_clusters.demo]
        control_plane = "{control_plane_name}"
        host_group = "remote-linux"
        server_machine = "cloud-generic"
        server_machines = ["cloud-generic"]
        worker_machines = ["cloud-aws"]
        api_endpoint = "{api_endpoint}"
        control_plane_scheduler = "spread"
        server_args = ["--disable=traefik"]
        worker_args = ["--node-label=role=worker"]
        """
    ).strip()
    + "\n",
    encoding="utf-8",
)

port_bin = cargo_target_root() / "debug" / "port"
write_executable(bin_dir / "port", f"#!/usr/bin/env bash\nexec '{port_bin}' \"$@\"\n")
write_executable(bin_dir / "firecracker", "#!/usr/bin/env bash\nsleep 30\n")
write_executable(
    bin_dir / "ip",
    """#!/usr/bin/env bash
if [[ "${1:-}" == "-V" || "${1:-}" == "--version" ]]; then
  echo "ip utility, iproute2-6.12.0"
  exit 0
fi
if [[ "${1:-}" == "route" && "${2:-}" == "show" && "${3:-}" == "default" ]]; then
  echo "default via 192.0.2.1 dev eth0"
  exit 0
fi
exit 0
""",
)
write_executable(
    bin_dir / "iptables",
    """#!/usr/bin/env bash
if [[ "${1:-}" == "-V" || "${1:-}" == "--version" ]]; then
  echo "iptables v1.8.11"
  exit 0
fi
exit 0
""",
)

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
                control_plane_name,
                "--bind",
                control_addr,
            ],
        )
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

    for name, argv in [
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

    wait_for_tcp(generic_addr)
    wait_for_tcp(aws_addr)

    generic_paths = machine_paths(generic_runtime_root, "cloud-generic")
    aws_paths = machine_paths(aws_runtime_root, "cloud-aws")
    join_token = "demo-join-token"
    proof_runtime_root = tmpdir / "proof-runtime"

    threads.append(
        spawn_exec_sequence_server(
            generic_paths,
            [
                {
                    "type": "managed-service-start",
                    "name": "k3s-server",
                    "command": k3s_service_command(
                        "server",
                        ["--disable=traefik", "--node-name", "cloud-generic"],
                        bootstrap_flag="--cluster-init",
                    ),
                    "policy": k3s_service_policy("server"),
                },
                {
                    "type": "exec",
                    "command": [
                        "/bin/sh",
                        "-lc",
                        "cat /var/lib/rancher/k3s/server/token 2>/dev/null || cat /var/lib/rancher/k3s/server/node-token",
                    ],
                    "stdout": f"{join_token}\n",
                },
                {
                    "type": "managed-service-status",
                    "name": "k3s-server",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "cat /etc/rancher/k3s/k3s.yaml"],
                    "stdout": f"apiVersion: v1\nclusters:\n- cluster:\n    server: {api_endpoint}\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME           STATUS   ROLES                  AGE   VERSION\ncloud-generic   Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws       Ready    <none>                 1m    v1.35.2+k3s1\n",
                },
                {
                    "type": "managed-service-list",
                    "services": [running_managed_service_status("k3s-server")],
                },
                {
                    "type": "exec",
                    "command": [
                        "/bin/sh",
                        "-lc",
                        "set -eu; for path in '/run/port/k3s-server.pid' '/var/log/k3s-server.log'; do if [ -e \"$path\" ]; then printf '%s\\n' \"$path\"; fi; done",
                    ],
                    "stdout": "",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "cat /etc/rancher/k3s/k3s.yaml"],
                    "stdout": f"apiVersion: v1\nclusters:\n- cluster:\n    server: {api_endpoint}\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME           STATUS   ROLES                  AGE   VERSION\ncloud-generic   Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws       Ready    <none>                 1m    v1.35.2+k3s1\n",
                },
                {
                    "type": "managed-service-list",
                    "services": [running_managed_service_status("k3s-server")],
                },
                {
                    "type": "exec",
                    "command": [
                        "/bin/sh",
                        "-lc",
                        "set -eu; for path in '/run/port/k3s-server.pid' '/var/log/k3s-server.log'; do if [ -e \"$path\" ]; then printf '%s\\n' \"$path\"; fi; done",
                    ],
                    "stdout": "",
                },
            ],
        )
    )
    threads.append(
        spawn_exec_sequence_server(
            aws_paths,
            [
                {
                    "type": "managed-service-start",
                    "name": "k3s-agent",
                    "command": k3s_service_command(
                        "agent",
                        ["--node-label=role=worker", "--node-name", "cloud-aws"],
                        server_url=api_endpoint,
                        join_token=join_token,
                    ),
                    "policy": k3s_service_policy("agent"),
                },
                {
                    "type": "managed-service-status",
                    "name": "k3s-agent",
                },
                {
                    "type": "managed-service-list",
                    "services": [running_managed_service_status("k3s-agent")],
                },
                {
                    "type": "managed-service-list",
                    "services": [running_managed_service_status("k3s-agent")],
                },
            ],
        )
    )

    cluster_up = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "up",
        "--cluster",
        "demo",
        "--runtime-root",
        str(proof_runtime_root),
    ]
    steps.append(
        (
            " ".join(cluster_up),
            filter_lines(
                run_checked(cluster_up, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "control plane:",
                    "host group:",
                    "api endpoint:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "control-plane machines:",
                    "worker machines:",
                    "control-plane launch:",
                    "worker launch:",
                    "boundary:",
                ],
            ),
        )
    )

    server_status = [
        "port",
        "--config",
        str(config_path),
        "service",
        "status",
        "--machine",
        "cloud-generic",
        "--name",
        "k3s-server",
    ]
    steps.append(
        (
            " ".join(server_status),
            filter_lines(
                run_checked(server_status, env=env, cwd=repo_root),
                [
                    "machine:",
                    "name:",
                    "desired state:",
                    "runtime state:",
                    "lifecycle owner:",
                    "service route:",
                    "control plane:",
                    "node:",
                    "host groups:",
                    "target host group:",
                    "restart policy:",
                    "health policy:",
                    "runtime record:",
                    "runtime pid:",
                    "stdout log:",
                    "stderr log:",
                    "detail:",
                ],
            ),
        )
    )

    worker_status = [
        "port",
        "--config",
        str(config_path),
        "service",
        "status",
        "--machine",
        "cloud-aws",
        "--name",
        "k3s-agent",
    ]
    steps.append(
        (
            " ".join(worker_status),
            filter_lines(
                run_checked(worker_status, env=env, cwd=repo_root),
                [
                    "machine:",
                    "name:",
                    "desired state:",
                    "runtime state:",
                    "lifecycle owner:",
                    "service route:",
                    "control plane:",
                    "node:",
                    "host groups:",
                    "target host group:",
                    "restart policy:",
                    "health policy:",
                    "runtime record:",
                    "runtime pid:",
                    "stdout log:",
                    "stderr log:",
                    "detail:",
                ],
            ),
        )
    )

    cluster_status = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "status",
        "--cluster",
        "demo",
        "--runtime-root",
        str(proof_runtime_root),
    ]
    steps.append(
        (
            " ".join(cluster_status),
            filter_lines(
                run_checked(cluster_status, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "control plane:",
                    "host group:",
                    "control-plane machines:",
                    "worker machines:",
                    "api endpoint:",
                    "machine truth:",
                    "managed-service truth:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "legacy-runtime drift:",
                    "legacy-runtime detail:",
                    "managed-service detail:",
                    "control-plane placement:",
                ],
            ),
        )
    )

    kubeconfig_command = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "kubeconfig",
        "--cluster",
        "demo",
        "--runtime-root",
        str(proof_runtime_root),
    ]
    steps.append(
        (
            " ".join(kubeconfig_command),
            filter_lines(
                run_checked(kubeconfig_command, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "control plane:",
                    "host group:",
                    "api endpoint:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "kubeconfig surface:",
                    "server:",
                ],
            ),
        )
    )

    cluster_down = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "down",
        "--cluster",
        "demo",
        "--runtime-root",
        str(proof_runtime_root),
    ]
    steps.append(
        (
            " ".join(cluster_down),
            filter_lines(
                run_checked(cluster_down, env=env, cwd=repo_root),
                ["control-plane stop:", "worker stop:"],
            ),
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
