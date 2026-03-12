use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use port_guest_agent::serve as serve_guest_agent;
use port_model::PortConfig;
use port_runtime::{
    ControlPlaneServeRequest, NodeAgentServeRequest, serve_control_plane, serve_node_agent,
};
use serde_json::json;
use tempfile::tempdir;

fn write_config(path: &Path, config: &PortConfig) {
    fs::write(path, config.to_toml_string().expect("config should encode"))
        .expect("config should write");
}

fn port_bin() -> &'static str {
    env!("CARGO_BIN_EXE_port")
}

fn port_command(config_path: &Path) -> Command {
    let mut command = Command::new(port_bin());
    command.env("PORT_DEMO_TOKEN", "demo-token");
    command.arg("--config").arg(config_path);
    command
}

fn hosted_config(runtime_root: &Path, control_plane_endpoint: &str) -> PortConfig {
    let mut config = PortConfig::sample();
    config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = runtime_root.to_path_buf();
    config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = control_plane_endpoint.to_string();
    config
}

fn write_machine_manifest(runtime_root: &Path, machine: &str, pid: u32) {
    let runtime_dir = runtime_root.join(machine);
    fs::create_dir_all(&runtime_dir).expect("runtime dir should exist");
    let manifest_path = runtime_dir.join("manifest.json");
    let manifest = json!({
        "machine_name": machine,
        "pid": pid,
        "launched_at_unix_s": 1,
        "runtime_dir": runtime_dir,
        "firecracker_binary": "/usr/bin/firecracker",
        "config_path": runtime_dir.join("firecracker-config.json"),
        "log_path": runtime_dir.join("firecracker.log"),
        "stdout_path": runtime_dir.join("console.stdout.log"),
        "stderr_path": runtime_dir.join("console.stderr.log"),
        "manifest_path": manifest_path,
        "attached_volumes": [],
    });
    fs::write(
        &manifest_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("manifest should encode")
        ),
    )
    .expect("manifest should write");
}

fn reserve_addr() -> String {
    TcpListener::bind("127.0.0.1:0")
        .expect("listener should bind")
        .local_addr()
        .expect("addr should exist")
        .to_string()
}

fn wait_for_port(addr: &str, rx: &mpsc::Receiver<anyhow::Result<()>>, label: &str) {
    for _ in 0..100 {
        if let Ok(result) = rx.try_recv() {
            result.unwrap_or_else(|error| panic!("{label} failed: {error}"));
        }
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("{label} did not become reachable in time");
}

#[test]
fn cli_service_secret_backend_commands_cover_hosted_secret_service_and_sandbox_contracts() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let control_plane_addr = reserve_addr();
    let control_plane_endpoint = format!("http://{control_plane_addr}");
    let _ = fs::remove_dir_all(Path::new(".port/hosted/demo"));
    unsafe {
        std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
    }
    let config_path = temp.path().join("port.toml");
    let config = hosted_config(&hosted_runtime_root, &control_plane_endpoint);
    write_config(&config_path, &config);
    write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424242);
    let guest_root = temp.path().join("guest");
    fs::create_dir_all(guest_root.join("workspace")).expect("guest workspace should exist");
    let guest_socket = hosted_runtime_root.join("cloud-aws/guest-agent.sock");
    let guest_socket_for_thread = guest_socket.clone();
    let guest_root_for_thread = guest_root.clone();
    thread::spawn(move || {
        serve_guest_agent(&guest_socket_for_thread, guest_root_for_thread)
            .expect("guest agent should serve");
    });
    for _ in 0..100 {
        if guest_socket.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    let (control_tx, control_rx) = mpsc::channel();
    let control_config = config.clone();
    let control_bind = control_plane_addr.clone();
    thread::spawn(move || {
        let result = serve_control_plane(
            control_config,
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
                bind: control_bind,
                node_bindings: Vec::new(),
            },
        )
        .map(|_| ());
        let _ = control_tx.send(result);
    });
    wait_for_port(&control_plane_addr, &control_rx, "control-plane");

    let node_addr = reserve_addr();
    let (node_tx, node_rx) = mpsc::channel();
    let node_config = config.clone();
    let node_bind = node_addr.clone();
    thread::spawn(move || {
        let result = serve_node_agent(
            node_config,
            NodeAgentServeRequest {
                node_name: String::from("aws-linux-node"),
                bind: node_bind,
                token: String::from("node-secret"),
            },
        )
        .map(|_| ());
        let _ = node_tx.send(result);
    });
    wait_for_port(&node_addr, &node_rx, "node-agent");

    let secret_put = port_command(&config_path)
        .arg("service")
        .arg("secret")
        .arg("put")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("demo-token")
        .arg("--value")
        .arg("s3cr3t")
        .output()
        .expect("secret put should run");
    assert!(secret_put.status.success());
    let secret_stdout = String::from_utf8_lossy(&secret_put.stdout);
    assert!(secret_stdout.contains("secret: demo-token"));
    assert!(secret_stdout.contains("backend: runtime-file"));
    assert!(secret_stdout.contains("materialization: env"));
    assert!(secret_stdout.contains("service route: hosted-control-plane"));
    assert!(secret_stdout.contains("control plane: demo"));
    assert!(secret_stdout.contains("node: aws-linux-node"));
    assert!(secret_stdout.contains("backend path: "));
    assert!(secret_stdout.contains("services/secrets/runtime-file/demo-token"));

    let apply_service = port_command(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--host-group")
        .arg("aws-builders")
        .arg("--name")
        .arg("api")
        .arg("--kind")
        .arg("service")
        .arg("--restart")
        .arg("on-failure")
        .arg("--health")
        .arg("command")
        .arg("--health-command")
        .arg("/bin/true")
        .arg("--secret")
        .arg("API_TOKEN=demo-token")
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf '%s\\n' \"$API_TOKEN\" >&2; trap 'exit 0' TERM; while :; do sleep 1; done")
        .output()
        .expect("service apply should run");
    assert!(apply_service.status.success());
    let service_stdout = String::from_utf8_lossy(&apply_service.stdout);
    assert!(service_stdout.contains("name: api"));
    assert!(service_stdout.contains("kind: service"));
    assert!(service_stdout.contains("desired state: active"));
    assert!(service_stdout.contains("runtime state: running"));
    assert!(service_stdout.contains("target host group: aws-builders"));
    assert!(service_stdout.contains("scheduler: deterministic-first-fit"));
    assert!(service_stdout.contains("restart policy: on-failure"));
    assert!(service_stdout.contains("health policy: command"));
    assert!(service_stdout.contains("health command: /bin/true"));
    assert!(
        service_stdout.contains("runtime record: ")
            && service_stdout.contains("hosted/aws-linux-node/cloud-aws/services/runtime/api.json")
    );
    assert!(service_stdout.contains("secret bindings: API_TOKEN=demo-token"));
    assert!(
        service_stdout.contains("secret sources: ")
            && service_stdout.contains("API_TOKEN<=demo-token via runtime-file/env @ ")
    );
    assert!(service_stdout.contains("stderr log: /run/port/services/api.stderr.log"));

    let apply_sandbox = port_command(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--host-group")
        .arg("aws-builders")
        .arg("--name")
        .arg("buildbox")
        .arg("--kind")
        .arg("sandbox")
        .arg("--secret")
        .arg("API_TOKEN=demo-token")
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf sandbox-ok; exit 0")
        .output()
        .expect("sandbox apply should run");
    assert!(apply_sandbox.status.success());
    let sandbox_stdout = String::from_utf8_lossy(&apply_sandbox.stdout);
    assert!(sandbox_stdout.contains("name: buildbox"));
    assert!(sandbox_stdout.contains("kind: sandbox"));

    let list = port_command(&config_path)
        .arg("service")
        .arg("list")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("service list should run");
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("name: api"));
    assert!(list_stdout.contains("name: buildbox"));
    assert!(list_stdout.contains("target host group: aws-builders"));

    let status = port_command(&config_path)
        .arg("service")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service status should run");
    assert!(status.status.success());
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("guest broker: control-plane-node-agent-tunnel"));
    assert!(status_stdout.contains("service route: hosted-control-plane"));
    assert!(status_stdout.contains("runtime state: running"));
    assert!(status_stdout.contains("target host group: aws-builders"));
    assert!(status_stdout.contains("scheduler: deterministic-first-fit"));
    assert!(status_stdout.contains("restart policy: on-failure"));
    assert!(status_stdout.contains("health policy: command"));
    assert!(status_stdout.contains("health command: /bin/true"));
    assert!(
        status_stdout.contains("runtime record: ")
            && status_stdout.contains("hosted/aws-linux-node/cloud-aws/services/runtime/api.json")
    );
    assert!(status_stdout.contains("command: /bin/sh -lc"));
    assert!(
        status_stdout.contains("secret sources: ")
            && status_stdout.contains("API_TOKEN<=demo-token via runtime-file/env @ ")
    );

    let stop = port_command(&config_path)
        .arg("service")
        .arg("stop")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service stop should run");
    assert!(stop.status.success());
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("desired state: stopped"));
    assert!(stop_stdout.contains("runtime state: stopped"));
    assert!(stop_stdout.contains("target host group: aws-builders"));
    assert!(stop_stdout.contains("scheduler: deterministic-first-fit"));

    let runtime_record = hosted_runtime_root.join("cloud-aws/services/runtime/api.json");
    let runtime_record_text =
        fs::read_to_string(runtime_record).expect("runtime record should read");
    assert!(runtime_record_text.contains("\"state\": \"stopped\""));
    assert!(!runtime_record_text.contains("s3cr3t"));
}

#[test]
fn cli_service_secret_status_projects_restart_health_and_provenance_for_hosted_runtime() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let control_plane_addr = reserve_addr();
    let control_plane_endpoint = format!("http://{control_plane_addr}");
    let _ = fs::remove_dir_all(Path::new(".port/hosted/demo"));
    unsafe {
        std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
    }
    let config_path = temp.path().join("port.toml");
    let config = hosted_config(&hosted_runtime_root, &control_plane_endpoint);
    write_config(&config_path, &config);
    write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424243);

    let guest_root = temp.path().join("guest");
    fs::create_dir_all(guest_root.join("workspace")).expect("guest workspace should exist");
    let guest_socket = hosted_runtime_root.join("cloud-aws/guest-agent.sock");
    let guest_socket_for_thread = guest_socket.clone();
    let guest_root_for_thread = guest_root.clone();
    thread::spawn(move || {
        serve_guest_agent(&guest_socket_for_thread, guest_root_for_thread)
            .expect("guest agent should serve");
    });
    for _ in 0..100 {
        if guest_socket.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    let (control_tx, control_rx) = mpsc::channel();
    let control_plane_config = config.clone();
    let control_plane_addr_for_thread = control_plane_addr.clone();
    thread::spawn(move || {
        let result = serve_control_plane(
            control_plane_config,
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
                bind: control_plane_addr_for_thread,
                node_bindings: Vec::new(),
            },
        )
        .map(|_| ());
        let _ = control_tx.send(result);
    });
    wait_for_port(&control_plane_addr, &control_rx, "control plane");

    let node_bind = reserve_addr();
    let node_bind_for_thread = node_bind.clone();
    let (node_tx, node_rx) = mpsc::channel();
    let node_config = config.clone();
    thread::spawn(move || {
        let result = serve_node_agent(
            node_config,
            NodeAgentServeRequest {
                node_name: String::from("aws-linux-node"),
                bind: node_bind_for_thread,
                token: String::from("node-secret"),
            },
        )
        .map(|_| ());
        let _ = node_tx.send(result);
    });
    wait_for_port(&node_bind, &node_rx, "node-agent");

    let secret_put = port_command(&config_path)
        .arg("service")
        .arg("secret")
        .arg("put")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("demo-token")
        .arg("--value")
        .arg("s3cr3t")
        .output()
        .expect("secret put should run");
    assert!(secret_put.status.success());

    let apply = port_command(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--host-group")
        .arg("aws-builders")
        .arg("--name")
        .arg("api")
        .arg("--kind")
        .arg("service")
        .arg("--restart")
        .arg("on-failure")
        .arg("--health")
        .arg("command")
        .arg("--health-command")
        .arg("/bin/test")
        .arg("--health-command=-f")
        .arg("--health-command")
        .arg("workspace/healthy")
        .arg("--secret")
        .arg("API_TOKEN=demo-token")
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("count_file=workspace/restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; if [ \"$count\" -eq 1 ]; then sleep 0.2; exit 23; fi; trap 'exit 0' TERM; while :; do sleep 1; done")
        .output()
        .expect("service apply should run");
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );

    thread::sleep(Duration::from_millis(350));

    let first_status = port_command(&config_path)
        .arg("service")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service status should run");
    assert!(first_status.status.success());
    let first_stdout = String::from_utf8_lossy(&first_status.stdout);
    assert!(first_stdout.contains("restart count: 1"), "{first_stdout}");
    assert!(
        first_stdout.contains("last exit code: 23"),
        "{first_stdout}"
    );
    assert!(
        first_stdout.contains("last exit detail: managed process exited with code 23"),
        "{first_stdout}"
    );
    assert!(
        first_stdout.contains("health state: unhealthy"),
        "{first_stdout}"
    );
    assert!(
        first_stdout.contains("health detail: health command exited with code 1"),
        "{first_stdout}"
    );
    assert!(
        first_stdout.contains("secret sources: ")
            && first_stdout.contains("API_TOKEN<=demo-token via runtime-file/env @ "),
        "{first_stdout}"
    );
    assert!(!first_stdout.contains("s3cr3t"), "{first_stdout}");

    fs::write(guest_root.join("workspace/healthy"), "ok").expect("healthy marker should write");
    let second_status = port_command(&config_path)
        .arg("service")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service status should run");
    assert!(second_status.status.success());
    let second_stdout = String::from_utf8_lossy(&second_status.stdout);
    assert!(
        second_stdout.contains("restart count: 1"),
        "{second_stdout}"
    );
    assert!(
        second_stdout.contains("health state: healthy"),
        "{second_stdout}"
    );
    assert!(
        second_stdout.contains("health detail: (none)"),
        "{second_stdout}"
    );
    assert!(
        second_stdout.contains("services/secrets/runtime-file/demo-token"),
        "{second_stdout}"
    );
    assert!(!second_stdout.contains("s3cr3t"), "{second_stdout}");
}

#[test]
fn cli_service_policy_invalid_combinations_reject_before_runtime_lookup() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    write_config(&config_path, &port_model::PortConfig::sample());

    let invalid_restart = port_command(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("demo")
        .arg("--name")
        .arg("buildbox")
        .arg("--kind")
        .arg("sandbox")
        .arg("--restart")
        .arg("always")
        .arg("--")
        .arg("/bin/true")
        .output()
        .expect("invalid restart command should run");
    assert!(!invalid_restart.status.success());
    let invalid_restart_stderr = String::from_utf8_lossy(&invalid_restart.stderr);
    assert!(
        invalid_restart_stderr.contains("sandbox services only support restart policy 'never'"),
        "{invalid_restart_stderr}"
    );

    let missing_health_command = port_command(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("demo")
        .arg("--name")
        .arg("api")
        .arg("--health")
        .arg("command")
        .arg("--")
        .arg("/bin/true")
        .output()
        .expect("missing health command should run");
    assert!(!missing_health_command.status.success());
    let missing_health_stderr = String::from_utf8_lossy(&missing_health_command.stderr);
    assert!(
        missing_health_stderr
            .contains("health policy 'command' requires at least one health command token"),
        "{missing_health_stderr}"
    );
}
