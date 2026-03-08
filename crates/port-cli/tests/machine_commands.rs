use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

fn hosted_config(runtime_root: &Path) -> PortConfig {
    let mut config = PortConfig::sample();
    config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = runtime_root.to_path_buf();
    config
}

fn write_machine_manifest(runtime_root: &Path, machine: &str, pid: u32) -> PathBuf {
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
    manifest_path
}

fn write_forward_manifest(
    runtime_root: &Path,
    machine: &str,
    name: &str,
    pid: u32,
    listen: &str,
    target: &str,
) {
    let runtime_dir = runtime_root.join(machine);
    let forwards_dir = runtime_dir.join("forwards");
    fs::create_dir_all(&forwards_dir).expect("forwards dir should exist");
    let manifest = json!({
        "name": name,
        "machine": machine,
        "pid": pid,
        "listen": listen,
        "target": target,
        "stdout_log": runtime_dir.join(format!("{name}.forward.stdout.log")),
        "stderr_log": runtime_dir.join(format!("{name}.forward.stderr.log")),
    });
    fs::write(
        forwards_dir.join(format!("{name}.json")),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("manifest should encode")
        ),
    )
    .expect("forward manifest should write");
}

#[test]
fn cli_machine_monitor_reports_hosted_runtime_context() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    write_config(&config_path, &hosted_config(&hosted_runtime_root));
    let _ = write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424242);

    let mut command = Command::new("bash");
    command
        .args([
            "-lc",
            "exec -a port-forward /bin/sh -c 'sleep 30' -- cloud-aws-web",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn().expect("forward helper should start");
    thread::sleep(Duration::from_millis(100));
    write_forward_manifest(
        &hosted_runtime_root,
        "cloud-aws",
        "web",
        child.id(),
        "127.0.0.1:8081",
        "127.0.0.1:80",
    );

    let output = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("monitor")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("monitor command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("machine: cloud-aws"));
    assert!(stdout.contains("control plane: demo"));
    assert!(stdout.contains("node: aws-linux-node"));
    assert!(stdout.contains("host groups:"));
    assert!(stdout.contains("aws-builders"));
    assert!(stdout.contains("remote-linux"));
    assert!(stdout.contains("monitor route: hosted-control-plane"));
    assert!(stdout.contains("top route: hosted-control-plane"));
    assert!(stdout.contains("forward: web"));
    assert!(stdout.contains("listen: 127.0.0.1:8081"));
    assert!(stdout.contains("target: 127.0.0.1:80"));

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn cli_machine_top_reports_hypervisor_and_forward_entries() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    write_config(&config_path, &hosted_config(&hosted_runtime_root));

    let runtime_dir = hosted_runtime_root.join("cloud-aws");
    fs::create_dir_all(&runtime_dir).expect("runtime dir should exist");

    let mut firecracker = Command::new("bash");
    firecracker
        .args([
            "-lc",
            "exec -a firecracker /bin/sh -c 'sleep 30' --id cloud-aws",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut firecracker = firecracker
        .spawn()
        .expect("fake firecracker process should start");
    thread::sleep(Duration::from_millis(100));

    let _ = write_machine_manifest(&hosted_runtime_root, "cloud-aws", firecracker.id());
    fs::write(
        runtime_dir.join("firecracker.pid"),
        format!("{}\n", firecracker.id()),
    )
    .expect("pid should write");

    let mut forward = Command::new("bash");
    forward
        .args([
            "-lc",
            "exec -a port-forward /bin/sh -c 'sleep 30' -- cloud-aws-web",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut forward = forward.spawn().expect("forward helper should start");
    thread::sleep(Duration::from_millis(100));
    write_forward_manifest(
        &hosted_runtime_root,
        "cloud-aws",
        "web",
        forward.id(),
        "127.0.0.1:8081",
        "127.0.0.1:80",
    );

    let output = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("top")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("top command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("machine: cloud-aws"));
    assert!(stdout.contains("control plane: demo"));
    assert!(stdout.contains("node: aws-linux-node"));
    assert!(stdout.contains("entry kind: hypervisor"));
    assert!(stdout.contains("name: firecracker"));
    assert!(stdout.contains("entry kind: detached-forward"));
    assert!(stdout.contains("name: web"));
    assert!(stdout.contains("command:"));
    assert!(stdout.contains("port-forward"));

    let _ = forward.kill();
    let _ = forward.wait();
    let _ = firecracker.kill();
    let _ = firecracker.wait();
}
