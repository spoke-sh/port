use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use port_model::{ExecutionSubstrate, MachineArchitecture, PortConfig, ProtectionMode};
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

fn hosted_config(runtime_root: &Path) -> PortConfig {
    let mut config = PortConfig::sample();
    config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = runtime_root.to_path_buf();
    config
}

fn generic_hosted_config() -> PortConfig {
    PortConfig::sample()
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

fn write_fake_firecracker_binary(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, "#!/usr/bin/env bash\nsleep 30\n").expect("fake firecracker should write");
    let mut permissions = fs::metadata(&path)
        .expect("fake firecracker metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("fake firecracker permissions should update");
    path
}

#[test]
fn cli_help_mentions_native_avf_workflow_and_boundaries() {
    let output = Command::new(port_bin())
        .arg("--help")
        .output()
        .expect("help command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("demo-avf"));
    assert!(stdout.contains("PORT_AVF_LAUNCHER"));
    assert!(stdout.contains("machine launch"));
    assert!(stdout.contains("--machine demo-avf"));
    assert!(stdout.contains("Firecracker launch stays Linux-only"));
}

#[test]
fn cli_doctor_and_launch_surface_sample_avf_workflow_boundary() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    write_config(&config_path, &PortConfig::sample());

    let doctor = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("doctor")
        .output()
        .expect("doctor command should run");
    assert!(doctor.status.success());
    let doctor_stdout = String::from_utf8_lossy(&doctor.stdout);
    assert!(doctor_stdout.contains("avf:demo-avf:host-platform"));
    assert!(doctor_stdout.contains("avf:demo-avf:runtime-availability"));

    let launch = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo-avf")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(
        !launch.status.success(),
        "non-macOS CI host should reject demo-avf launch"
    );
    let stderr = String::from_utf8_lossy(&launch.stderr);
    assert!(stderr.contains("AVF local launch requires running Port on macOS"));
}

#[test]
fn cli_machine_monitor_reports_hosted_runtime_context() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let bogus_runtime_root = temp.path().join("bogus/aws-linux-node");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();
    let mut server_config = hosted_config(&hosted_runtime_root);
    server_config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");
    write_config(&server_config_path, &server_config);
    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root.clone();
    write_config(&client_config_path, &client_config);
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

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
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
fn cli_machine_status_surfaces_hosted_pvm_placement_denial() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("client-port.toml");
    let control_plane_addr = reserve_addr();
    let mut config = generic_hosted_config();
    config
        .machines
        .get_mut("cloud-generic")
        .expect("cloud-generic should exist")
        .protection_mode = port_model::ProtectionMode::Pvm;
    config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");
    write_config(&config_path, &config);

    let mut control_command = Command::new(port_bin());
    control_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&config_path)
        .arg("control-plane")
        .arg("serve")
        .arg("--control-plane")
        .arg("demo")
        .arg("--bind")
        .arg(&control_plane_addr)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _control_plane = ChildGuard::spawn("control-plane", control_command);
    wait_for_tcp(&control_plane_addr);

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("cloud-generic")
        .output()
        .expect("status command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("machine: cloud-generic"));
    assert!(stdout.contains("state: malformed"));
    assert!(stdout.contains("generic-linux-node"));
    assert!(stdout.contains("planned"));
    assert!(stdout.contains("PVM"));
}

#[test]
fn cli_machine_launch_rejects_unplaceable_hosted_pvm_machine() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let mut config = PortConfig::sample();
    config
        .machines
        .get_mut("cloud-generic")
        .expect("cloud-generic should exist")
        .protection_mode = port_model::ProtectionMode::Pvm;
    write_config(&config_path, &config);

    let output = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-generic")
        .output()
        .expect("launch command should run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cloud-generic"));
    assert!(stderr.contains("generic-linux-node"));
    assert!(stderr.contains("planned"));
    assert!(stderr.contains("PVM"));
}

#[test]
fn cli_machine_launch_routes_hosted_pvm_through_live_control_plane() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let bogus_runtime_root = temp.path().join("bogus/aws-linux-node");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();

    let mut server_config = hosted_config(&hosted_runtime_root);
    server_config
        .machines
        .get_mut("cloud-aws")
        .expect("cloud-aws should exist")
        .protection_mode = ProtectionMode::Pvm;
    server_config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");

    let kernel_path = temp.path().join("pvm-vmlinux");
    let guest_path = temp.path().join("pvm-rootfs.ext4");
    fs::write(&kernel_path, b"fake-kernel").expect("kernel variant should write");
    fs::write(&guest_path, b"fake-rootfs").expect("guest variant should write");

    server_config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("demo-kernel should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::Firecracker
                && variant.selector.protection_mode == ProtectionMode::Pvm
        })
        .expect("pvm kernel variant should exist")
        .path = kernel_path;
    server_config
        .artifacts
        .guest_images
        .get_mut("demo-guest")
        .expect("demo-guest should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::Firecracker
                && variant.selector.protection_mode == ProtectionMode::Pvm
        })
        .expect("pvm guest variant should exist")
        .path = guest_path;

    let host_kit = server_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .capabilities
        .pvm_lanes[0]
        .host_kit
        .as_mut()
        .expect("aws node should declare a host-kit");
    host_kit.requires_custom_host_kernel = false;
    host_kit.host_boot_args.clear();
    host_kit.firecracker_binary_env = Some(String::from("PORT_TEST_CLI_PVM_FIRECRACKER"));

    let fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker-pvm");
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let mut node_command = Command::new(port_bin());
    node_command
        .env("PORT_TEST_CLI_PVM_FIRECRACKER", &fake_binary)
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

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("launch command should run");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("launched machine: cloud-aws"));
    assert!(stdout.contains(fake_binary.to_string_lossy().as_ref()));

    let pid_path = hosted_runtime_root.join("cloud-aws/firecracker.pid");
    let pid = fs::read_to_string(&pid_path)
        .expect("pid file should exist")
        .trim()
        .parse::<u32>()
        .expect("pid should parse");
    assert!(hosted_runtime_root.join("cloud-aws/manifest.json").exists());

    let _ = Command::new("kill").arg(pid.to_string()).status();
}

#[test]
fn cli_machine_top_reports_hypervisor_and_forward_entries() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let bogus_runtime_root = temp.path().join("bogus/aws-linux-node");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();
    let mut server_config = hosted_config(&hosted_runtime_root);
    server_config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");
    write_config(&server_config_path, &server_config);
    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root.clone();
    write_config(&client_config_path, &client_config);

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

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
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
