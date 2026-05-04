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
cast_path = output_dir / "hosted-k3s-ha-failover-workflow.cast"
gif_path = output_dir / "ac-1.gif"
tmpdir = pathlib.Path(tempfile.mkdtemp(prefix="phk3sha-", dir="/tmp"))
control_plane_name = f"proofdemoha{os.getpid()}"


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
        stdout = result.stdout.strip()
        stderr = result.stderr.strip()
        raise RuntimeError(
            "\n".join(
                [
                    f"command failed with exit code {result.returncode}: {' '.join(argv)}",
                    f"cwd: {cwd}",
                    f"stdout:\n{stdout if stdout else '(empty)'}",
                    f"stderr:\n{stderr if stderr else '(empty)'}",
                ]
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
        raise RuntimeError(f"timed out waiting for tcp listener at {addr}\n{details}")
    raise RuntimeError(f"timed out waiting for tcp listener at {addr}")


def machine_paths(runtime_root: pathlib.Path, machine_name: str) -> dict[str, pathlib.Path]:
    runtime_dir = runtime_root / machine_name
    return {
        "runtime_dir": runtime_dir,
        "manifest_path": runtime_dir / "manifest.json",
        "pid_path": runtime_dir / "firecracker.pid",
        "vsock_path": runtime_dir / "guest.vsock",
    }


def write_manifest(paths: dict[str, pathlib.Path], machine_name: str, pid: int) -> None:
    paths["runtime_dir"].mkdir(parents=True, exist_ok=True)
    manifest = {
        "machine_name": machine_name,
        "pid": pid,
        "launched_at_unix_s": 1,
        "runtime_dir": str(paths["runtime_dir"]),
        "firecracker_binary": "/usr/bin/firecracker",
        "config_path": str(paths["runtime_dir"] / "firecracker-config.json"),
        "log_path": str(paths["runtime_dir"] / "firecracker.log"),
        "stdout_path": str(paths["runtime_dir"] / "console.stdout.log"),
        "stderr_path": str(paths["runtime_dir"] / "console.stderr.log"),
        "manifest_path": str(paths["manifest_path"]),
        "attached_volumes": [],
    }
    paths["manifest_path"].write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def hosted_state_root(control_plane: str) -> pathlib.Path:
    return repo_root / ".port" / "hosted" / control_plane


def write_registered_node_state(control_plane: str, nodes: dict[str, dict[str, object]]) -> None:
    state_path = hosted_state_root(control_plane) / "registered-nodes.json"
    state_path.parent.mkdir(parents=True, exist_ok=True)
    state_path.write_text(
        json.dumps({"control_plane": control_plane, "nodes": nodes}, indent=2) + "\n",
        encoding="utf-8",
    )


def write_machine_placement_state(
    control_plane: str, placements: dict[str, tuple[str, pathlib.Path, str]]
) -> None:
    state_path = hosted_state_root(control_plane) / "machine-placements.json"
    state_path.parent.mkdir(parents=True, exist_ok=True)
    machines = {
        machine_name: {
            "node_name": node_name,
            "runtime_root": str(runtime_root),
            "placed_at_unix_s": 1,
            "placement_detail": detail,
        }
        for machine_name, (node_name, runtime_root, detail) in placements.items()
    }
    state_path.write_text(
        json.dumps({"control_plane": control_plane, "machines": machines}, indent=2) + "\n",
        encoding="utf-8",
    )


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
                    if expected_operation["type"] == "exec":
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
                    elif expected_operation["type"] == "managed-service-list":
                        if operation["type"] != "managed-service":
                            raise AssertionError(
                                f"unexpected hosted guest operation: {operation!r}"
                            )
                        if operation["operation"]["verb"] != "list":
                            raise AssertionError(
                                f"unexpected managed-service verb: {operation['operation']!r}"
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
                            f"unsupported expected operation type: {expected_operation['type']!r}"
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


def spawn_sleep_process(command: list[str]) -> subprocess.Popen[str]:
    return subprocess.Popen(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    )


control_addr = reserve_addr()
node_a_addr = reserve_addr()
node_b_addr = reserve_addr()
node_c_addr = reserve_addr()

bin_dir = tmpdir / "bin"
bin_dir.mkdir(parents=True, exist_ok=True)
runtime_root = tmpdir / "runtime"
runtime_a = runtime_root / "aws-linux-node"
runtime_b = runtime_root / "aws-linux-node-b"
runtime_c = runtime_root / "aws-linux-node-c"
runtime_a.mkdir(parents=True, exist_ok=True)
runtime_b.mkdir(parents=True, exist_ok=True)
runtime_c.mkdir(parents=True, exist_ok=True)

kernel_path = tmpdir / "demo-vmlinux"
guest_path = tmpdir / "demo-rootfs.ext4"
kernel_path.write_bytes(b"fake-kernel")
guest_path.write_bytes(b"fake-rootfs")

config_path = tmpdir / "port-ha.toml"
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

        [hosts.aws-linux]
        platform = "linux"
        provider = "aws"

        [hosts.aws-linux.connection]
        mode = "hosted-control-plane"
        control_plane = "{control_plane_name}"

        [hosts.aws-linux.firecracker]
        local_launch = false
        notes = ["AWS hosted control-plane nodes for the first hosted HA endpoint proof."]

        [nodes.aws-linux-node]
        host = "aws-linux"
        runtime_root = "{runtime_a}"
        notes = ["Primary hosted AWS execution node for the HA failover proof."]

        [nodes.aws-linux-node.capabilities]
        providers = ["aws"]
        platforms = ["linux"]
        substrates = ["firecracker"]
        architectures = ["x86_64"]
        protection_modes = ["standard"]

        [nodes.aws-linux-node-b]
        host = "aws-linux"
        runtime_root = "{runtime_b}"
        notes = ["Secondary hosted AWS execution node for the HA failover proof."]

        [nodes.aws-linux-node-b.capabilities]
        providers = ["aws"]
        platforms = ["linux"]
        substrates = ["firecracker"]
        architectures = ["x86_64"]
        protection_modes = ["standard"]

        [nodes.aws-linux-node-c]
        host = "aws-linux"
        runtime_root = "{runtime_c}"
        notes = ["Tertiary hosted AWS execution node for the HA failover proof."]

        [nodes.aws-linux-node-c.capabilities]
        providers = ["aws"]
        platforms = ["linux"]
        substrates = ["firecracker"]
        architectures = ["x86_64"]
        protection_modes = ["standard"]

        [host_groups.aws-builders]
        placement = "explicit-membership"
        scheduler = "deterministic-first-fit"
        nodes = ["aws-linux-node", "aws-linux-node-b", "aws-linux-node-c"]
        notes = ["Three hosted AWS execution nodes for the HA failover proof."]

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
        vsock_cid = 60
        control_port = 7000
        console_log = "runtime/cloud-aws/console.log"

        [machines.cloud-aws-b]
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

        [machines.cloud-aws-b.guest]
        vsock_cid = 61
        control_port = 7002
        console_log = "runtime/cloud-aws-b/console.log"

        [machines.cloud-aws-c]
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

        [machines.cloud-aws-c.guest]
        vsock_cid = 62
        control_port = 7004
        console_log = "runtime/cloud-aws-c/console.log"

        [machines.cloud-aws-worker]
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

        [machines.cloud-aws-worker.guest]
        vsock_cid = 63
        control_port = 7006
        console_log = "runtime/cloud-aws-worker/console.log"

        [k3s_clusters.demo]
        control_plane = "{control_plane_name}"
        host_group = "aws-builders"
        server_machine = "cloud-aws"
        server_machines = ["cloud-aws", "cloud-aws-b", "cloud-aws-c"]
        worker_machines = ["cloud-aws-worker"]
        api_endpoint = "https://demo-k3s.internal:6443"
        control_plane_scheduler = "spread"
        server_args = ["--disable=traefik"]
        worker_args = []
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

subprocess.run(
    ["cargo", "build", "-q", "-p", "port", "--bin", "port"],
    cwd=repo_root,
    env=env,
    check=True,
)

processes: list[tuple[subprocess.Popen[str], object, object]] = []
threads: list[threading.Thread] = []
extra_processes: list[subprocess.Popen[str]] = []
steps: list[tuple[str, str]] = [("export PORT_DEMO_TOKEN=demo-token", "")]
primary_paths = machine_paths(runtime_a, "cloud-aws")
secondary_paths = machine_paths(runtime_b, "cloud-aws-b")
tertiary_paths = machine_paths(runtime_c, "cloud-aws-c")
worker_paths = machine_paths(runtime_c, "cloud-aws-worker")
registered_at = int(time.time())

write_registered_node_state(
    control_plane_name,
    {
        "aws-linux-node": {
            "endpoint": f"http://{node_a_addr}",
            "token": "node-secret",
            "registered_at": registered_at,
            "refreshed_at": registered_at,
            "ttl_seconds": 30,
        },
        "aws-linux-node-b": {
            "endpoint": f"http://{node_b_addr}",
            "token": "node-secret",
            "registered_at": registered_at,
            "refreshed_at": registered_at,
            "ttl_seconds": 30,
        },
        "aws-linux-node-c": {
            "endpoint": f"http://{node_c_addr}",
            "token": "node-secret",
            "registered_at": registered_at,
            "refreshed_at": registered_at,
            "ttl_seconds": 30,
        },
    },
)

try:
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

    for name, bind, node in [
        ("node-agent-a", node_a_addr, "aws-linux-node"),
        ("node-agent-b", node_b_addr, "aws-linux-node-b"),
        ("node-agent-c", node_c_addr, "aws-linux-node-c"),
    ]:
        argv = [
            "port",
            "--config",
            str(config_path),
            "node-agent",
            "serve",
            "--node",
            node,
            "--bind",
            bind,
            "--token",
            "node-secret",
        ]
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

    wait_for_tcp(node_a_addr)
    wait_for_tcp(node_b_addr)
    wait_for_tcp(node_c_addr)

    write_registered_node_state(
        control_plane_name,
        {
            "aws-linux-node": {
                "endpoint": f"http://{node_a_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
            "aws-linux-node-b": {
                "endpoint": f"http://{node_b_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
            "aws-linux-node-c": {
                "endpoint": f"http://{node_c_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
        },
    )

    primary_process = spawn_sleep_process([str(bin_dir / "firecracker")])
    secondary_process = spawn_sleep_process([str(bin_dir / "firecracker")])
    tertiary_process = spawn_sleep_process([str(bin_dir / "firecracker")])
    worker_process = spawn_sleep_process([str(bin_dir / "firecracker")])
    extra_processes.extend(
        [primary_process, secondary_process, tertiary_process, worker_process]
    )
    write_manifest(primary_paths, "cloud-aws", primary_process.pid)
    write_manifest(secondary_paths, "cloud-aws-b", secondary_process.pid)
    write_manifest(tertiary_paths, "cloud-aws-c", tertiary_process.pid)
    write_manifest(worker_paths, "cloud-aws-worker", worker_process.pid)

    write_machine_placement_state(
        control_plane_name,
        {
            "cloud-aws": (
                "aws-linux-node",
                runtime_a,
                "Stored on the primary hosted AWS node.",
            ),
            "cloud-aws-b": (
                "aws-linux-node-b",
                runtime_b,
                "Stored on the secondary hosted AWS node.",
            ),
            "cloud-aws-c": (
                "aws-linux-node-c",
                runtime_c,
                "Stored on the tertiary hosted AWS node.",
            ),
            "cloud-aws-worker": (
                "aws-linux-node-c",
                runtime_c,
                "Stored on the hosted AWS worker node runtime.",
            ),
        },
    )
    steps.append(
        (
            "seed hosted control-plane placements",
            textwrap.dedent(
                f"""
                primary guest pid: {primary_process.pid}
                control-plane placement: cloud-aws -> aws-linux-node
                control-plane placement: cloud-aws-b -> aws-linux-node-b
                control-plane placement: cloud-aws-c -> aws-linux-node-c
                worker placement: cloud-aws-worker -> aws-linux-node-c
                """
            ).lstrip(),
        )
    )

    threads.append(
        spawn_exec_sequence_server(
            primary_paths,
            [
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "cat /etc/rancher/k3s/k3s.yaml"],
                    "stdout": "apiVersion: v1\nclusters:\n- cluster:\n    server: https://cloud-aws:6443\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME             STATUS   ROLES                  AGE   VERSION\ncloud-aws        Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-b      Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-c      Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-worker Ready    <none>                 1m    v1.35.4+k3s1\n",
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
                    "stdout": "apiVersion: v1\nclusters:\n- cluster:\n    server: https://cloud-aws:6443\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME             STATUS   ROLES                  AGE   VERSION\ncloud-aws        Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-b      Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-c      Ready    control-plane,master   1m    v1.35.4+k3s1\ncloud-aws-worker Ready    <none>                 1m    v1.35.4+k3s1\n",
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
    for paths, service_name in [
        (secondary_paths, "k3s-server"),
        (tertiary_paths, "k3s-server"),
        (worker_paths, "k3s-agent"),
    ]:
        threads.append(
            spawn_exec_sequence_server(
                paths,
                [
                    {
                        "type": "managed-service-list",
                        "services": [running_managed_service_status(service_name)],
                    },
                    {
                        "type": "managed-service-list",
                        "services": [running_managed_service_status(service_name)],
                    },
                    {
                        "type": "managed-service-list",
                        "services": [running_managed_service_status(service_name)],
                    },
                    {
                        "type": "managed-service-list",
                        "services": [running_managed_service_status(service_name)],
                    },
                ],
            )
        )

    status_before = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "status",
        "--cluster",
        "demo",
        "--runtime-root",
        str(tmpdir / "ignored-runtime"),
    ]
    steps.append(
        (
            " ".join(status_before),
            filter_lines(
                run_checked(status_before, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "api endpoint:",
                    "machine truth:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "real-ha status:",
                    "cloud-aws-worker",
                    "control-plane placement:",
                ],
            ),
        )
    )

    kubeconfig_before = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "kubeconfig",
        "--cluster",
        "demo",
        "--runtime-root",
        str(tmpdir / "ignored-runtime"),
    ]
    steps.append(
        (
            " ".join(kubeconfig_before),
            filter_lines(
                run_checked(kubeconfig_before, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "api endpoint:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "server:",
                ],
            ),
        )
    )

    primary_process.send_signal(signal.SIGTERM)
    primary_process.wait(timeout=5)
    extra_processes.remove(primary_process)
    replacement_primary = spawn_sleep_process([str(bin_dir / "firecracker")])
    extra_processes.append(replacement_primary)
    write_manifest(primary_paths, "cloud-aws", replacement_primary.pid)

    write_machine_placement_state(
        control_plane_name,
        {
            "cloud-aws": (
                "aws-linux-node",
                runtime_a,
                "Stored on the replacement hosted AWS node runtime after primary guest replacement.",
            ),
            "cloud-aws-b": (
                "aws-linux-node-b",
                runtime_b,
                "Stored on the secondary hosted AWS node.",
            ),
            "cloud-aws-c": (
                "aws-linux-node-c",
                runtime_c,
                "Stored on the tertiary hosted AWS node.",
            ),
            "cloud-aws-worker": (
                "aws-linux-node-c",
                runtime_c,
                "Stored on the hosted AWS worker node runtime.",
            ),
        },
    )
    steps.append(
        (
            "simulate primary control-plane guest replacement",
            textwrap.dedent(
                f"""
                previous primary guest pid: {primary_process.pid}
                replacement primary guest pid: {replacement_primary.pid}
                control-plane placement: cloud-aws -> aws-linux-node
                replacement detail: Stored on the replacement hosted AWS node runtime after primary guest replacement.
                """
            ).lstrip(),
        )
    )
    write_registered_node_state(
        control_plane_name,
        {
            "aws-linux-node": {
                "endpoint": f"http://{node_a_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
            "aws-linux-node-b": {
                "endpoint": f"http://{node_b_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
            "aws-linux-node-c": {
                "endpoint": f"http://{node_c_addr}",
                "token": "node-secret",
                "registered_at": registered_at,
                "refreshed_at": registered_at,
                "ttl_seconds": 30,
            },
        },
    )

    threads.append(
        spawn_exec_sequence_server(
            primary_paths,
            [
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "cat /etc/rancher/k3s/k3s.yaml"],
                    "stdout": "apiVersion: v1\nclusters:\n- cluster:\n    server: https://cloud-aws:6443\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME             STATUS   ROLES                  AGE   VERSION\ncloud-aws        Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-b      Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-c      Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-worker Ready    <none>                 2m    v1.35.4+k3s1\n",
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
                    "stdout": "apiVersion: v1\nclusters:\n- cluster:\n    server: https://cloud-aws:6443\n",
                },
                {
                    "type": "exec",
                    "command": ["/bin/sh", "-lc", "k3s kubectl get nodes -o wide"],
                    "stdout": "NAME             STATUS   ROLES                  AGE   VERSION\ncloud-aws        Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-b      Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-c      Ready    control-plane,master   2m    v1.35.4+k3s1\ncloud-aws-worker Ready    <none>                 2m    v1.35.4+k3s1\n",
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

    status_after = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "status",
        "--cluster",
        "demo",
        "--runtime-root",
        str(tmpdir / "ignored-runtime"),
    ]
    steps.append(
        (
            " ".join(status_after),
            filter_lines(
                run_checked(status_after, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "api endpoint:",
                    "machine truth:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "real-ha status:",
                    "cloud-aws-worker",
                    "control-plane placement:",
                ],
            ),
        )
    )

    kubeconfig_after = [
        "port",
        "--config",
        str(config_path),
        "cluster",
        "kubeconfig",
        "--cluster",
        "demo",
        "--runtime-root",
        str(tmpdir / "ignored-runtime"),
    ]
    steps.append(
        (
            " ".join(kubeconfig_after),
            filter_lines(
                run_checked(kubeconfig_after, env=env, cwd=repo_root),
                [
                    "cluster:",
                    "api endpoint:",
                    "stable-endpoint posture:",
                    "stable-endpoint detail:",
                    "server:",
                ],
            ),
        )
    )

    write_cast(cast_path, steps, width=144, height=40)
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
            "40",
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
    shutil.rmtree(hosted_state_root(control_plane_name), ignore_errors=True)
    for thread in threads:
        thread.join(timeout=5)
    for process in extra_processes:
        if process.poll() is None:
            process.send_signal(signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
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
    shutil.rmtree(tmpdir, ignore_errors=True)
PY
