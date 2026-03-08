use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use port_model::PortConfig;
use serde_json::json;
use tempfile::tempdir;

fn write_config(path: &Path, config: &PortConfig) {
    fs::write(path, config.to_toml_string().expect("config should encode"))
        .expect("config should write");
}

fn port_bin() -> &'static str {
    env!("CARGO_BIN_EXE_port")
}

fn reserve_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("port should bind");
    let addr = listener.local_addr().expect("addr should exist");
    drop(listener);
    addr.to_string()
}

fn wait_for_tcp(addr: &str) {
    for _ in 0..100 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for tcp listener at '{addr}'");
}

fn runtime_socket(runtime_root: &Path, machine: &str) -> PathBuf {
    runtime_root.join(machine).join("guest-agent.sock")
}

fn spawn_agent(socket: &Path, root: &Path) {
    if let Some(parent) = socket.parent() {
        fs::create_dir_all(parent).expect("runtime dir should exist");
    }
    let socket = socket.to_path_buf();
    let serve_socket = socket.clone();
    let root = root.to_path_buf();
    thread::spawn(move || {
        port_guest_agent::serve(&serve_socket, root).expect("agent should serve");
    });

    for _ in 0..100 {
        if socket.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }

    panic!(
        "guest agent socket did not appear at '{}'",
        socket.display()
    );
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

#[derive(Debug)]
struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn spawn(name: &'static str, mut command: Command) -> Self {
        let child = command.spawn().unwrap_or_else(|error| {
            panic!("failed to spawn {name}: {error}");
        });
        Self { child }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn cli_routes_hosted_machine_and_guest_commands_through_live_http_transport() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let good_runtime_root = temp.path().join("hosted/aws-linux-node");
    let bogus_runtime_root = temp.path().join("bogus/aws-linux-node");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    fs::write(guest_root.join("var/log/app.log"), "first\nsecond\n").expect("log file");
    fs::create_dir_all(&bogus_runtime_root).expect("bogus root should exist");

    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();

    let mut server_config = PortConfig::sample();
    server_config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");
    server_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = good_runtime_root.clone();
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root.clone();
    write_config(&client_config_path, &client_config);

    let socket_path = runtime_socket(&good_runtime_root, "cloud-aws");
    spawn_agent(&socket_path, &guest_root);
    write_machine_manifest(&good_runtime_root, "cloud-aws", 424242);

    let mut node_command = Command::new(port_bin());
    node_command
        .arg("--config")
        .arg(&server_config_path)
        .arg("node-agent")
        .arg("serve")
        .arg("--node")
        .arg("aws-linux-node")
        .arg("--bind")
        .arg(&node_addr)
        .arg("--token")
        .arg("node-secret")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _node = ChildGuard::spawn("node-agent", node_command);
    wait_for_tcp(&node_addr);

    let mut control_command = Command::new(port_bin());
    control_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&server_config_path)
        .arg("control-plane")
        .arg("serve")
        .arg("--control-plane")
        .arg("demo")
        .arg("--bind")
        .arg(&control_plane_addr)
        .arg("--node-binding")
        .arg(format!("aws-linux-node=http://{node_addr},node-secret"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _control_plane = ChildGuard::spawn("control-plane", control_command);
    wait_for_tcp(&control_plane_addr);

    let status = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("status command should run");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(
        status_stdout.contains(&format!(
            "runtime dir: {}",
            good_runtime_root.join("cloud-aws").display()
        )),
        "stdout: {status_stdout}"
    );
    assert!(
        !status_stdout.contains(&bogus_runtime_root.display().to_string()),
        "stdout: {status_stdout}"
    );

    let exec = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("guest")
        .arg("exec")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf hosted-http-ok")
        .output()
        .expect("guest exec command should run");
    assert_eq!(
        exec.status.code(),
        Some(0),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&exec.stdout),
        String::from_utf8_lossy(&exec.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&exec.stdout), "hosted-http-ok");
}
