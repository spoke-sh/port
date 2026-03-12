use std::collections::BTreeMap;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

use port_model::{
    ExecutionSubstrate, HostConnection, HostProvider, MachineArchitecture, MachineVolumeBackend,
    MachineVolumePersistence, MachineVolumeSpec, PortConfig, ProtectionMode,
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

fn hosted_server_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn cleanup_hosted_registration_state() {
    let _ = fs::remove_dir_all(Path::new(".port/hosted/demo"));
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

struct HostedServerHarness {
    _lock: MutexGuard<'static, ()>,
    _control_plane: ChildGuard,
    _node: ChildGuard,
}

fn spawn_hosted_server_harness(
    server_config_path: &Path,
    node_addr: &str,
    control_plane_addr: &str,
    extra_node_env: &[(&str, &Path)],
) -> HostedServerHarness {
    spawn_hosted_server_harness_with_cleanup(
        server_config_path,
        node_addr,
        control_plane_addr,
        extra_node_env,
        true,
    )
}

fn spawn_hosted_server_harness_preserving_state(
    server_config_path: &Path,
    node_addr: &str,
    control_plane_addr: &str,
    extra_node_env: &[(&str, &Path)],
) -> HostedServerHarness {
    spawn_hosted_server_harness_with_cleanup(
        server_config_path,
        node_addr,
        control_plane_addr,
        extra_node_env,
        false,
    )
}

fn spawn_hosted_server_harness_for_node(
    server_config_path: &Path,
    node_name: &str,
    node_addr: &str,
    control_plane_addr: &str,
    extra_node_env: &[(&str, &Path)],
) -> HostedServerHarness {
    let lock = hosted_server_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cleanup_hosted_registration_state();

    let mut control_command = Command::new(port_bin());
    control_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(server_config_path)
        .arg("control-plane")
        .arg("serve")
        .arg("--control-plane")
        .arg("demo")
        .arg("--bind")
        .arg(control_plane_addr)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let control_plane = ChildGuard::spawn("control-plane", control_command);
    wait_for_tcp(control_plane_addr);

    let mut node_command = Command::new(port_bin());
    node_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(server_config_path)
        .arg("node-agent")
        .arg("serve")
        .arg("--node")
        .arg(node_name)
        .arg("--bind")
        .arg(node_addr)
        .arg("--token")
        .arg("node-secret")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in extra_node_env {
        node_command.env(name, value);
    }
    let node = ChildGuard::spawn("node-agent", node_command);
    wait_for_tcp(node_addr);

    HostedServerHarness {
        _lock: lock,
        _control_plane: control_plane,
        _node: node,
    }
}

fn spawn_hosted_server_harness_with_cleanup(
    server_config_path: &Path,
    node_addr: &str,
    control_plane_addr: &str,
    extra_node_env: &[(&str, &Path)],
    cleanup_state: bool,
) -> HostedServerHarness {
    let lock = hosted_server_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cleanup_state {
        cleanup_hosted_registration_state();
    }

    let mut control_command = Command::new(port_bin());
    control_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(server_config_path)
        .arg("control-plane")
        .arg("serve")
        .arg("--control-plane")
        .arg("demo")
        .arg("--bind")
        .arg(control_plane_addr)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let control_plane = ChildGuard::spawn("control-plane", control_command);
    wait_for_tcp(control_plane_addr);

    let mut node_command = Command::new(port_bin());
    node_command
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(server_config_path)
        .arg("node-agent")
        .arg("serve")
        .arg("--node")
        .arg("aws-linux-node")
        .arg("--bind")
        .arg(node_addr)
        .arg("--token")
        .arg("node-secret")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in extra_node_env {
        node_command.env(name, value);
    }
    let node = ChildGuard::spawn("node-agent", node_command);
    wait_for_tcp(node_addr);

    HostedServerHarness {
        _lock: lock,
        _control_plane: control_plane,
        _node: node,
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

fn hosted_multi_node_config(runtime_root: &Path, alternate_runtime_root: &Path) -> PortConfig {
    let mut config = hosted_config(runtime_root);
    let mut alternate = config
        .nodes
        .get("aws-linux-node")
        .expect("aws-linux-node should exist")
        .clone();
    alternate.runtime_root = alternate_runtime_root.to_path_buf();
    config
        .nodes
        .insert(String::from("aws-linux-node-b"), alternate);
    config
        .host_groups
        .get_mut("aws-builders")
        .expect("aws-builders should exist")
        .nodes = vec![
        String::from("aws-linux-node-b"),
        String::from("aws-linux-node"),
    ];
    config
}

fn hosted_three_node_config(
    runtime_root: &Path,
    alternate_runtime_root: &Path,
    imported_only_runtime_root: &Path,
) -> PortConfig {
    let mut config = hosted_multi_node_config(runtime_root, alternate_runtime_root);
    let mut imported_only = config
        .nodes
        .get("aws-linux-node")
        .expect("aws-linux-node should exist")
        .clone();
    imported_only.runtime_root = imported_only_runtime_root.to_path_buf();
    config
        .nodes
        .insert(String::from("aws-linux-node-c"), imported_only);
    config
        .host_groups
        .get_mut("aws-builders")
        .expect("aws-builders should exist")
        .nodes = vec![
        String::from("aws-linux-node-c"),
        String::from("aws-linux-node-b"),
        String::from("aws-linux-node"),
    ];
    config
}

fn generic_hosted_config() -> PortConfig {
    PortConfig::sample()
}

fn prepend_path_env(path: &Path) -> PathBuf {
    let mut entries = vec![path.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        entries.extend(std::env::split_paths(&existing));
    }
    PathBuf::from(std::env::join_paths(entries).expect("PATH should join"))
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
    manifest_path
}

fn write_machine_placement_state(
    control_plane: &str,
    machine_name: &str,
    node_name: &str,
    runtime_root: &Path,
    placement_detail: &str,
) {
    let state_path = Path::new(".port/hosted")
        .join(control_plane)
        .join("machine-placements.json");
    fs::create_dir_all(
        state_path
            .parent()
            .expect("machine placement state path should have parent"),
    )
    .expect("machine placement state dir should exist");
    fs::write(
        &state_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "control_plane": control_plane,
                "machines": {
                    machine_name: {
                        "node_name": node_name,
                        "runtime_root": runtime_root,
                        "placed_at_unix_s": 1,
                        "placement_detail": placement_detail,
                    }
                }
            }))
            .expect("machine placement state should encode")
        ),
    )
    .expect("machine placement state should write");
}

fn write_imported_inventory_state(
    control_plane: &str,
    nodes: BTreeMap<String, port_model::HostedImportedNodeRecord>,
) {
    let state_path = Path::new(".port/hosted")
        .join(control_plane)
        .join("imported-inventory.json");
    fs::create_dir_all(
        state_path
            .parent()
            .expect("imported inventory path should have parent"),
    )
    .expect("imported inventory dir should exist");
    fs::write(
        &state_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "control_plane": control_plane,
                "nodes": nodes,
            }))
            .expect("imported inventory state should encode")
        ),
    )
    .expect("imported inventory state should write");
}

fn write_registered_node_state(
    control_plane: &str,
    nodes: BTreeMap<String, port_model::HostedNodeRegistration>,
) {
    let state_path = Path::new(".port/hosted")
        .join(control_plane)
        .join("registered-nodes.json");
    fs::create_dir_all(
        state_path
            .parent()
            .expect("registered node state path should have parent"),
    )
    .expect("registered node state dir should exist");
    fs::write(
        &state_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "control_plane": control_plane,
                "nodes": nodes,
            }))
            .expect("registered node state should encode")
        ),
    )
    .expect("registered node state should write");
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

fn write_fake_port_wrapper(root: &Path) -> PathBuf {
    let path = root.join("port");
    fs::write(
        &path,
        format!("#!/usr/bin/env bash\nexec '{}' \"$@\"\n", port_bin()),
    )
    .expect("fake port wrapper should write");
    let mut permissions = fs::metadata(&path)
        .expect("fake port wrapper metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("fake port wrapper permissions should update");
    path
}

fn write_fake_ssh_binary(root: &Path) -> PathBuf {
    let path = root.join("ssh");
    fs::write(
        &path,
        r#"#!/usr/bin/env bash
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
"#,
    )
    .expect("fake ssh should write");
    let mut permissions = fs::metadata(&path)
        .expect("fake ssh metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("fake ssh permissions should update");
    path
}

fn sample_ssh_machine_config(root: &Path) -> PortConfig {
    let mut config = PortConfig::sample();
    write_fake_standard_firecracker_artifacts(&mut config, root);
    config.nodes.clear();
    config.host_groups.clear();
    config
        .hosts
        .get_mut("generic-linux")
        .expect("generic-linux host should exist")
        .connection = HostConnection::Ssh {
        destination: String::from("builder.example.internal"),
        user: String::from("ubuntu"),
        port: 2222,
    };
    config
}

fn write_fake_standard_firecracker_artifacts(config: &mut PortConfig, root: &Path) {
    let kernel_path = root.join("standard-vmlinux");
    let guest_path = root.join("standard-rootfs.ext4");
    fs::write(&kernel_path, b"fake-standard-kernel").expect("standard kernel should write");
    fs::write(&guest_path, b"fake-standard-rootfs").expect("standard guest should write");

    config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("demo-kernel should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::Firecracker
                && variant.selector.protection_mode == ProtectionMode::Standard
        })
        .expect("standard kernel variant should exist")
        .path = kernel_path;
    config
        .artifacts
        .guest_images
        .get_mut("demo-guest")
        .expect("demo-guest should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::Firecracker
                && variant.selector.protection_mode == ProtectionMode::Standard
        })
        .expect("standard guest variant should exist")
        .path = guest_path;
}

fn write_fake_attached_volume(config: &mut PortConfig, root: &Path, file_name: &str) -> PathBuf {
    let volume_path = root.join(file_name);
    fs::write(&volume_path, b"fake-attached-volume").expect("attached volume should write");
    config
        .machines
        .get_mut("demo")
        .expect("demo machine should exist")
        .volumes = vec![MachineVolumeSpec {
        name: String::from("data"),
        backend: MachineVolumeBackend::HostFile,
        persistence: MachineVolumePersistence::Persistent,
        path: volume_path.clone(),
    }];
    volume_path
}

fn write_fake_cloud_hypervisor_artifacts(config: &mut PortConfig, root: &Path) {
    let kernel_path = root.join("cloud-hypervisor-vmlinux");
    let guest_path = root.join("cloud-hypervisor-rootfs.ext4");
    fs::write(&kernel_path, b"fake-cloud-hypervisor-kernel")
        .expect("cloud hypervisor kernel should write");
    fs::write(&guest_path, b"fake-cloud-hypervisor-rootfs")
        .expect("cloud hypervisor guest should write");

    config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("demo-kernel should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::CloudHypervisor
                && variant.selector.protection_mode == ProtectionMode::Standard
        })
        .expect("cloud hypervisor kernel variant should exist")
        .path = kernel_path;
    config
        .artifacts
        .guest_images
        .get_mut("demo-guest")
        .expect("demo-guest should exist")
        .variants
        .iter_mut()
        .find(|variant| {
            variant.selector.architecture == MachineArchitecture::X86_64
                && variant.selector.substrate == ExecutionSubstrate::CloudHypervisor
                && variant.selector.protection_mode == ProtectionMode::Standard
        })
        .expect("cloud hypervisor guest variant should exist")
        .path = guest_path;
}

fn write_fake_pvm_firecracker_artifacts(config: &mut PortConfig, root: &Path) {
    let kernel_path = root.join("pvm-vmlinux");
    let guest_path = root.join("pvm-rootfs.ext4");
    fs::write(&kernel_path, b"fake-pvm-kernel").expect("pvm kernel should write");
    fs::write(&guest_path, b"fake-pvm-rootfs").expect("pvm guest should write");

    config
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
    config
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
}

#[test]
fn cli_help_stays_concise_and_links_lane_specific_docs() {
    let output = Command::new(port_bin())
        .arg("--help")
        .output()
        .expect("help command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Examples:"));
    assert!(stdout.contains("port doctor"));
    assert!(stdout.contains("CONFIGURATION.md"));
    assert!(stdout.contains("docs/install.md"));
    assert!(stdout.contains("docs/avf.md"));
    assert!(stdout.contains("docs/operators.md"));
    assert!(stdout.contains("Apple Virtualization Framework"));
    assert!(stdout.contains("PORT_AVF_LAUNCHER"));
    assert!(!stdout.contains("demo-avf"));
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
    assert!(doctor_stdout.contains("PORT_AVF_LAUNCHER"));
    assert!(doctor_stdout.contains("virtualization entitlement"));
    assert!(doctor_stdout.contains("bundled macOS-only"));

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
    assert!(!stderr.contains("fallback to Firecracker"));
}

#[test]
fn cli_machine_launch_status_and_stop_route_cloud_hypervisor_locally() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    let mut config = PortConfig::sample();
    write_fake_cloud_hypervisor_artifacts(&mut config, temp.path());
    write_config(&config_path, &config);
    let fake_binary = write_fake_firecracker_binary(temp.path(), "cloud-hypervisor");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo-ch")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(
        launch_stdout.contains("launched machine: demo-ch"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("hypervisor binary:"),
        "{launch_stdout}"
    );
    assert!(launch_stdout.contains("hypervisor log:"), "{launch_stdout}");
    assert!(
        launch_stdout.contains(fake_binary.to_string_lossy().as_ref()),
        "{launch_stdout}"
    );

    let status = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("demo-ch")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("status command should run");
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(
        status_stdout.contains("machine: demo-ch"),
        "{status_stdout}"
    );
    assert!(status_stdout.contains("state: running"), "{status_stdout}");
    assert!(status_stdout.contains("hypervisor log:"), "{status_stdout}");
    assert!(
        status_stdout.contains("Cloud Hypervisor"),
        "{status_stdout}"
    );

    let stop = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("demo-ch")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("machine: demo-ch"), "{stop_stdout}");
    assert!(
        stop_stdout.contains("previous state: running"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("current state: stopped"),
        "{stop_stdout}"
    );
}

#[test]
fn cli_machine_launch_status_and_stop_round_trip() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    let mut config = PortConfig::sample();
    write_fake_standard_firecracker_artifacts(&mut config, temp.path());
    write_config(&config_path, &config);
    let fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(
        launch_stdout.contains("launched machine: demo"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("hypervisor binary:"),
        "{launch_stdout}"
    );
    assert!(launch_stdout.contains("hypervisor log:"), "{launch_stdout}");
    assert!(
        launch_stdout.contains(fake_binary.to_string_lossy().as_ref()),
        "{launch_stdout}"
    );

    let status = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("status command should run");
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("machine: demo"), "{status_stdout}");
    assert!(status_stdout.contains("state: running"), "{status_stdout}");
    assert!(
        status_stdout.contains("inventory owner: local-runtime-root"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("lifecycle owner: local-port-runtime"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("launch route: direct-local-runtime"),
        "{status_stdout}"
    );

    let stop = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("machine: demo"), "{stop_stdout}");
    assert!(
        stop_stdout.contains("previous state: running"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("current state: stopped"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("lifecycle owner: local-port-runtime"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("stop route: direct-local-runtime"),
        "{stop_stdout}"
    );
}

#[test]
fn cli_machine_launch_status_and_stop_with_attached_volume() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    let mut config = PortConfig::sample();
    write_fake_standard_firecracker_artifacts(&mut config, temp.path());
    let volume_path = write_fake_attached_volume(&mut config, temp.path(), "demo-data.ext4");
    write_config(&config_path, &config);
    let _fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(
        launch_stdout.contains("launched machine: demo"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("attached volume: data"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("backend: host-file"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains(volume_path.to_string_lossy().as_ref()),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("inventory owner: local-runtime-root"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("lifecycle owner: local-port-runtime"),
        "{launch_stdout}"
    );

    let config_json = fs::read_to_string(runtime_root.join("demo/firecracker-config.json"))
        .expect("firecracker config should exist");
    assert!(
        config_json.contains("\"drive_id\": \"rootfs\""),
        "{config_json}"
    );
    assert!(
        config_json.contains("\"drive_id\": \"data\""),
        "{config_json}"
    );
    assert!(
        config_json.contains(volume_path.to_string_lossy().as_ref()),
        "{config_json}"
    );

    let status = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("status command should run");
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("machine: demo"), "{status_stdout}");
    assert!(status_stdout.contains("state: running"), "{status_stdout}");
    assert!(
        status_stdout.contains("attached volume: data"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("backend: host-file"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains(volume_path.to_string_lossy().as_ref()),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("launch route: direct-local-runtime"),
        "{status_stdout}"
    );

    let stop = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("machine: demo"), "{stop_stdout}");
    assert!(
        stop_stdout.contains("attached volume: data"),
        "{stop_stdout}"
    );
    assert!(stop_stdout.contains("backend: host-file"), "{stop_stdout}");
    assert!(
        stop_stdout.contains(volume_path.to_string_lossy().as_ref()),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("stop route: direct-local-runtime"),
        "{stop_stdout}"
    );
}

#[test]
fn cli_attached_volume_route_context() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    let mut config = PortConfig::sample();
    write_fake_standard_firecracker_artifacts(&mut config, temp.path());
    let missing_volume_path = temp.path().join("missing-data.ext4");
    config
        .machines
        .get_mut("demo")
        .expect("demo machine should exist")
        .volumes = vec![MachineVolumeSpec {
        name: String::from("data"),
        backend: MachineVolumeBackend::HostFile,
        persistence: MachineVolumePersistence::Persistent,
        path: missing_volume_path.clone(),
    }];
    write_config(&config_path, &config);
    let _fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(!launch.status.success(), "{launch:?}");
    let stderr = String::from_utf8_lossy(&launch.stderr);
    assert!(stderr.contains("machine 'demo'"), "{stderr}");
    assert!(stderr.contains("volume 'data'"), "{stderr}");
    assert!(stderr.contains("backend 'host-file'"), "{stderr}");
    assert!(
        stderr.contains(missing_volume_path.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(stderr.contains("local-runtime-root"), "{stderr}");
    assert!(stderr.contains("local-port-runtime"), "{stderr}");
    assert!(stderr.contains("direct-local-runtime"), "{stderr}");
}

#[test]
fn cli_machine_launch_surfaces_missing_cloud_hypervisor_binary() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("runtime");
    let mut config = PortConfig::sample();
    write_fake_cloud_hypervisor_artifacts(&mut config, temp.path());
    write_config(&config_path, &config);

    let launch = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("demo-ch")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(!launch.status.success(), "{launch:?}");
    let stderr = String::from_utf8_lossy(&launch.stderr);
    assert!(
        stderr.contains("Cloud Hypervisor local launch requires"),
        "{stderr}"
    );
    assert!(stderr.contains("cloud-hypervisor"), "{stderr}");
    assert!(stderr.contains("demo-ch"), "{stderr}");
}

#[test]
fn cli_ssh_machine_launch_status_and_stop_round_trip() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("remote-runtime");
    let config = sample_ssh_machine_config(temp.path());
    write_config(&config_path, &config);
    write_fake_port_wrapper(temp.path());
    write_fake_ssh_binary(temp.path());
    write_fake_firecracker_binary(temp.path(), "firecracker");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-generic")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(launch_stdout.contains("launched machine: cloud-generic"));
    assert!(
        launch_stdout.contains("host: generic-linux"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("provider: generic-linux"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("launch route: ssh-managed-remote"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("inventory owner: ssh-remote-runtime"),
        "{launch_stdout}"
    );
    assert!(
        launch_stdout.contains("lifecycle owner: ssh-remote-port-runtime"),
        "{launch_stdout}"
    );

    let status = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("cloud-generic")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("status command should run");
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(
        status_stdout.contains("machine: cloud-generic"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("host: generic-linux"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("provider: generic-linux"),
        "{status_stdout}"
    );
    assert!(status_stdout.contains("state: running"), "{status_stdout}");
    assert!(
        status_stdout.contains("status route: ssh-managed-remote"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("inventory owner: ssh-remote-runtime"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("lifecycle owner: ssh-remote-port-runtime"),
        "{status_stdout}"
    );
    assert!(
        status_stdout.contains("builder.example.internal"),
        "{status_stdout}"
    );

    let stop = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("cloud-generic")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(
        stop_stdout.contains("machine: cloud-generic"),
        "{stop_stdout}"
    );
    assert!(stop_stdout.contains("host: generic-linux"), "{stop_stdout}");
    assert!(
        stop_stdout.contains("provider: generic-linux"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("previous state: running"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("current state: stopped"),
        "{stop_stdout}"
    );
    assert!(
        stop_stdout.contains("stop route: ssh-managed-remote"),
        "{stop_stdout}"
    );
}

#[test]
fn cli_ssh_machine_route_context() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let runtime_root = temp.path().join("remote-runtime");
    let mut config = sample_ssh_machine_config(temp.path());
    config
        .hosts
        .get_mut("generic-linux")
        .expect("generic-linux host should exist")
        .provider = HostProvider::Local;
    write_config(&config_path, &config);
    write_fake_port_wrapper(temp.path());
    write_fake_ssh_binary(temp.path());
    write_fake_firecracker_binary(temp.path(), "firecracker");
    let path_env = prepend_path_env(temp.path());

    let launch = Command::new(port_bin())
        .env("PATH", &path_env)
        .arg("--config")
        .arg(&config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-generic")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .output()
        .expect("launch command should run");
    assert!(!launch.status.success(), "{launch:?}");
    let stderr = String::from_utf8_lossy(&launch.stderr);
    assert!(stderr.contains("machine 'cloud-generic'"), "{stderr}");
    assert!(stderr.contains("host 'generic-linux'"), "{stderr}");
    assert!(stderr.contains("provider 'local'"), "{stderr}");
    assert!(stderr.contains("ssh-managed-remote"), "{stderr}");
    assert!(stderr.contains("ssh-remote-runtime"), "{stderr}");
    assert!(stderr.contains("ssh-remote-port-runtime"), "{stderr}");
    assert!(
        stderr.contains("ubuntu@builder.example.internal:2222"),
        "{stderr}"
    );
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
    write_fake_standard_firecracker_artifacts(&mut server_config, temp.path());
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

    let _servers = spawn_hosted_server_harness_preserving_state(
        &server_config_path,
        &node_addr,
        &control_plane_addr,
        &[],
    );

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
    let _lock = hosted_server_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cleanup_hosted_registration_state();
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
fn cli_machine_status_prefers_stored_hosted_placement_over_live_candidate() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let alternate_runtime_root = temp.path().join("hosted/aws-linux-node-b");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();

    let mut server_config = hosted_multi_node_config(&hosted_runtime_root, &alternate_runtime_root);
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
        .runtime_root = temp.path().join("bogus/aws-linux-node");
    client_config
        .nodes
        .get_mut("aws-linux-node-b")
        .expect("aws-linux-node-b should exist")
        .runtime_root = temp.path().join("bogus/aws-linux-node-b");
    write_config(&client_config_path, &client_config);

    let _servers =
        spawn_hosted_server_harness(&server_config_path, &node_addr, &control_plane_addr, &[]);
    let _ = write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424242);
    write_machine_placement_state(
        "demo",
        "cloud-aws",
        "aws-linux-node-b",
        &alternate_runtime_root,
        "Stored on alternate AWS node.",
    );

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("status command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("machine: cloud-aws"));
    assert!(stdout.contains("state: malformed"));
    assert!(stdout.contains("aws-linux-node-b"));
    assert!(stdout.contains("Stored on alternate AWS node."));
    assert!(
        !stdout.contains("control plane 'demo' resolved node 'aws-linux-node'"),
        "status output should not reroute to the currently live candidate"
    );
}

#[test]
#[ignore = "fleet-state render proof is covered by port-cli unit tests and port-runtime state tests"]
fn cli_machine_status_surfaces_hosted_fleet_node_state() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let alternate_runtime_root = temp.path().join("hosted/aws-linux-node-b");
    let imported_only_runtime_root = temp.path().join("hosted/aws-linux-node-c");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();

    let mut server_config = hosted_three_node_config(
        &hosted_runtime_root,
        &alternate_runtime_root,
        &imported_only_runtime_root,
    );
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
        .runtime_root = temp.path().join("bogus/aws-linux-node");
    client_config
        .nodes
        .get_mut("aws-linux-node-b")
        .expect("aws-linux-node-b should exist")
        .runtime_root = temp.path().join("bogus/aws-linux-node-b");
    client_config
        .nodes
        .get_mut("aws-linux-node-c")
        .expect("aws-linux-node-c should exist")
        .runtime_root = temp.path().join("bogus/aws-linux-node-c");
    write_config(&client_config_path, &client_config);

    let imported_only_node = server_config
        .nodes
        .get("aws-linux-node-c")
        .expect("aws-linux-node-c should exist");
    let imported_only_provider = server_config
        .hosts
        .get(&imported_only_node.host)
        .expect("aws-linux-node-c host should exist")
        .provider
        .clone();
    let mut imported_inventory = BTreeMap::new();
    imported_inventory.insert(
        String::from("aws-linux-node-c"),
        port_model::HostedImportedNodeRecord {
            provider: imported_only_provider,
            provenance: String::from("imported/aws-linux-node-c.json"),
            imported_at: 1_700_000_123,
            capability_summary: imported_only_node.capabilities.clone(),
            pvm_host_kit_packages: Vec::new(),
        },
    );
    write_imported_inventory_state("demo", imported_inventory);

    let mut registered_nodes = BTreeMap::new();
    registered_nodes.insert(
        String::from("aws-linux-node-b"),
        port_model::HostedNodeRegistration {
            endpoint: String::from("http://127.0.0.1:39999"),
            token: String::from("stale-node-token"),
            registered_at: 1,
            refreshed_at: 2,
            ttl_seconds: 1,
        },
    );
    write_registered_node_state("demo", registered_nodes);

    let _servers =
        spawn_hosted_server_harness(&server_config_path, &node_addr, &control_plane_addr, &[]);
    let _ = write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424242);

    let output = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("status command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fleet nodes:"), "{stdout}");
    assert!(stdout.contains("node: aws-linux-node"), "{stdout}");
    assert!(stdout.contains("selected: true"), "{stdout}");
    assert!(stdout.contains("freshness: live"), "{stdout}");
    assert!(stdout.contains("routing eligibility: eligible"), "{stdout}");
    assert!(stdout.contains("node: aws-linux-node-b"), "{stdout}");
    assert!(stdout.contains("freshness: stale"), "{stdout}");
    assert!(
        stdout.contains("routing eligibility: stale-registration"),
        "{stdout}"
    );
    assert!(stdout.contains("node: aws-linux-node-c"), "{stdout}");
    assert!(stdout.contains("imported: true"), "{stdout}");
    assert!(stdout.contains("registered: false"), "{stdout}");
    assert!(
        stdout.contains("freshness: missing-registration"),
        "{stdout}"
    );
    assert!(
        stdout.contains("routing eligibility: missing-registration"),
        "{stdout}"
    );
    assert!(
        stdout.contains("import provenance: imported/aws-linux-node-c.json"),
        "{stdout}"
    );
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

    let _servers = spawn_hosted_server_harness(
        &server_config_path,
        &node_addr,
        &control_plane_addr,
        &[("PORT_TEST_CLI_PVM_FIRECRACKER", fake_binary.as_path())],
    );

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
    let placement_state: serde_json::Value = serde_json::from_slice(
        &fs::read(".port/hosted/demo/machine-placements.json")
            .expect("machine placement state should exist"),
    )
    .expect("machine placement state should decode");
    assert_eq!(
        placement_state["machines"]["cloud-aws"]["node_name"].as_str(),
        Some("aws-linux-node")
    );
    assert_eq!(
        placement_state["machines"]["cloud-aws"]["runtime_root"].as_str(),
        Some(hosted_runtime_root.to_string_lossy().as_ref())
    );

    let _ = Command::new("kill").arg(pid.to_string()).status();
}

#[test]
fn cli_control_plane_prepare_pvm_node_enables_generic_hosted_pvm_launch() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/generic-linux-node");
    let bogus_runtime_root = temp.path().join("bogus/generic-linux-node");
    let server_config_path = temp.path().join("server-port.toml");
    let client_config_path = temp.path().join("client-port.toml");
    let node_addr = reserve_addr();
    let control_plane_addr = reserve_addr();

    let mut server_config = generic_hosted_config();
    server_config
        .machines
        .get_mut("cloud-generic")
        .expect("cloud-generic should exist")
        .protection_mode = ProtectionMode::Pvm;
    server_config
        .nodes
        .get_mut("generic-linux-node")
        .expect("generic-linux-node should exist")
        .runtime_root = hosted_runtime_root.clone();
    server_config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = format!("http://{control_plane_addr}");
    write_fake_pvm_firecracker_artifacts(&mut server_config, temp.path());
    let host_kit = server_config
        .hosts
        .get_mut("local")
        .expect("local host should exist")
        .firecracker
        .pvm_lanes
        .iter_mut()
        .find(|lane| lane.architecture == MachineArchitecture::X86_64)
        .expect("local x86_64 PVM lane should exist")
        .host_kit
        .as_mut()
        .expect("local x86_64 PVM lane should define a host-kit");
    host_kit.requires_custom_host_kernel = false;
    host_kit.host_boot_args.clear();
    host_kit.firecracker_binary_env = Some(String::from("PORT_TEST_CLI_PVM_FIRECRACKER"));
    let package = host_kit.package.clone();
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("generic-linux-node")
        .expect("generic-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker-pvm");
    let _servers = spawn_hosted_server_harness_for_node(
        &server_config_path,
        "generic-linux-node",
        &node_addr,
        &control_plane_addr,
        &[("PORT_TEST_CLI_PVM_FIRECRACKER", fake_binary.as_path())],
    );

    let prepare = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("control-plane")
        .arg("prepare-pvm-node")
        .arg("--control-plane")
        .arg("demo")
        .arg("--node")
        .arg("generic-linux-node")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--provenance")
        .arg("inventory/generic-linux-node.json")
        .arg("--package-name")
        .arg(&package.name)
        .arg("--package-version")
        .arg(&package.version)
        .arg("--host-kernel-release")
        .arg(&package.host_kernel_release)
        .arg("--firecracker-build")
        .arg(&package.firecracker_build)
        .output()
        .expect("prepare-pvm-node command should run");
    assert!(prepare.status.success(), "{prepare:?}");
    let prepare_stdout = String::from_utf8_lossy(&prepare.stdout);
    assert!(prepare_stdout.contains("prepared hosted pvm node: generic-linux-node"));
    assert!(prepare_stdout.contains("firecracker-pvm-host-kit@2026.03"));

    let imported_inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(".port/hosted/demo/imported-inventory.json")
            .expect("imported inventory state should exist"),
    )
    .expect("imported inventory state should decode");
    assert_eq!(
        imported_inventory["nodes"]["generic-linux-node"]["provenance"].as_str(),
        Some("inventory/generic-linux-node.json")
    );
    assert_eq!(
        imported_inventory["nodes"]["generic-linux-node"]["capability_summary"]["pvm_lanes"][0]
            ["state"]
            .as_str(),
        Some("ready")
    );

    let launch = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-generic")
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(launch_stdout.contains("launched machine: cloud-generic"));
    assert!(launch_stdout.contains(fake_binary.to_string_lossy().as_ref()));

    let pid_path = hosted_runtime_root.join("cloud-generic/firecracker.pid");
    let pid = fs::read_to_string(&pid_path)
        .expect("pid file should exist")
        .trim()
        .parse::<u32>()
        .expect("pid should parse");
    assert!(
        hosted_runtime_root
            .join("cloud-generic/manifest.json")
            .exists()
    );

    let placement_state: serde_json::Value = serde_json::from_slice(
        &fs::read(".port/hosted/demo/machine-placements.json")
            .expect("machine placement state should exist"),
    )
    .expect("machine placement state should decode");
    assert_eq!(
        placement_state["machines"]["cloud-generic"]["node_name"].as_str(),
        Some("generic-linux-node")
    );
    assert_eq!(
        placement_state["machines"]["cloud-generic"]["runtime_root"].as_str(),
        Some(hosted_runtime_root.to_string_lossy().as_ref())
    );

    let _ = Command::new("kill").arg(pid.to_string()).status();
}

#[test]
fn cli_hosted_standard_cloud_launch_round_trip() {
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
    write_fake_standard_firecracker_artifacts(&mut server_config, temp.path());
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker");
    let joined_path = prepend_path_env(temp.path());
    let _servers = spawn_hosted_server_harness(
        &server_config_path,
        &node_addr,
        &control_plane_addr,
        &[("PATH", joined_path.as_path())],
    );

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

    let placement_state: serde_json::Value = serde_json::from_slice(
        &fs::read(".port/hosted/demo/machine-placements.json")
            .expect("machine placement state should exist"),
    )
    .expect("machine placement state should decode");
    assert_eq!(
        placement_state["machines"]["cloud-aws"]["node_name"].as_str(),
        Some("aws-linux-node")
    );
    assert_eq!(
        placement_state["machines"]["cloud-aws"]["runtime_root"].as_str(),
        Some(hosted_runtime_root.to_string_lossy().as_ref())
    );

    let _ = Command::new("kill").arg(pid.to_string()).status();
}

#[test]
fn cli_hosted_standard_status_and_stop_round_trip() {
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
    write_fake_standard_firecracker_artifacts(&mut server_config, temp.path());
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let _fake_binary = write_fake_firecracker_binary(temp.path(), "firecracker");
    let joined_path = prepend_path_env(temp.path());
    let _servers = spawn_hosted_server_harness(
        &server_config_path,
        &node_addr,
        &control_plane_addr,
        &[("PATH", joined_path.as_path())],
    );

    let launch = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");

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
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("machine: cloud-aws"));
    assert!(status_stdout.contains("detail:"));
    assert!(status_stdout.contains("control plane 'demo'"));
    assert!(status_stdout.contains("node 'aws-linux-node'"));
    assert!(status_stdout.contains("provider 'aws'"));

    let stop = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("machine: cloud-aws"));
    assert!(stop_stdout.contains("detail:"));
    assert!(stop_stdout.contains("control plane 'demo'"));
    assert!(stop_stdout.contains("node 'aws-linux-node'"));
    assert!(stop_stdout.contains("provider 'aws'"));
}

#[test]
fn cli_hosted_cloud_hypervisor_launch_status_and_stop_round_trip() {
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
    server_config
        .machines
        .get_mut("cloud-aws")
        .expect("cloud-aws should exist")
        .substrate = ExecutionSubstrate::CloudHypervisor;
    server_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .capabilities
        .substrates = vec![ExecutionSubstrate::CloudHypervisor];
    write_fake_cloud_hypervisor_artifacts(&mut server_config, temp.path());
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let fake_binary = write_fake_firecracker_binary(temp.path(), "cloud-hypervisor");
    let joined_path = prepend_path_env(temp.path());
    let _servers = spawn_hosted_server_harness(
        &server_config_path,
        &node_addr,
        &control_plane_addr,
        &[("PATH", joined_path.as_path())],
    );

    let launch = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("launch command should run");
    assert!(launch.status.success(), "{launch:?}");
    let launch_stdout = String::from_utf8_lossy(&launch.stdout);
    assert!(launch_stdout.contains("launched machine: cloud-aws"));
    assert!(launch_stdout.contains(fake_binary.to_string_lossy().as_ref()));

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
    assert!(status.status.success(), "{status:?}");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("machine: cloud-aws"));
    assert!(status_stdout.contains("detail:"));
    assert!(status_stdout.contains("Cloud Hypervisor"));
    assert!(status_stdout.contains("control plane 'demo'"));
    assert!(status_stdout.contains("node 'aws-linux-node'"));
    assert!(status_stdout.contains("provider 'aws'"));

    let stop = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("stop")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("stop command should run");
    assert!(stop.status.success(), "{stop:?}");
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("machine: cloud-aws"));
    assert!(stop_stdout.contains("detail:"));
    assert!(stop_stdout.contains("Cloud Hypervisor"));
    assert!(stop_stdout.contains("control plane 'demo'"));
    assert!(stop_stdout.contains("node 'aws-linux-node'"));
    assert!(stop_stdout.contains("provider 'aws'"));
}

#[test]
fn cli_hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback() {
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
    server_config
        .machines
        .get_mut("cloud-aws")
        .expect("cloud-aws should exist")
        .substrate = ExecutionSubstrate::CloudHypervisor;
    write_config(&server_config_path, &server_config);

    let mut client_config = server_config.clone();
    client_config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = bogus_runtime_root;
    write_config(&client_config_path, &client_config);

    let _servers =
        spawn_hosted_server_harness(&server_config_path, &node_addr, &control_plane_addr, &[]);

    let launch = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&client_config_path)
        .arg("machine")
        .arg("launch")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("launch command should run");
    assert!(!launch.status.success(), "{launch:?}");
    let stderr = String::from_utf8_lossy(&launch.stderr);
    assert!(stderr.contains("cloud-aws"), "{stderr}");
    assert!(stderr.contains("control plane 'demo'"), "{stderr}");
    assert!(stderr.contains("aws-linux-node"), "{stderr}");
    assert!(stderr.contains("cloud-hypervisor"), "{stderr}");
    assert!(stderr.contains("rejected nodes"), "{stderr}");
    assert!(
        stderr.contains("requires standard protection on x86_64 via cloud-hypervisor"),
        "{stderr}"
    );
    assert!(
        !stderr.contains(
            "failed to launch machine 'cloud-aws' through the live hosted control-plane route"
        ),
        "{stderr}"
    );
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

    let _servers =
        spawn_hosted_server_harness(&server_config_path, &node_addr, &control_plane_addr, &[]);

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
