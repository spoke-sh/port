use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

use port_model::{
    ArtifactStore, MachineArchitecture, OciRegistryAuth, OciRegistryTransport, PortConfig,
    ProtectionMode,
};
use port_runtime::{ControlPlaneServeRequest, serve_control_plane};
use tempfile::tempdir;

fn port_bin() -> &'static str {
    env!("CARGO_BIN_EXE_port")
}

fn write_config(path: &Path, config: &PortConfig) {
    fs::write(
        path,
        config
            .to_toml_string()
            .expect("config should encode to TOML"),
    )
    .expect("config should write");
}

fn selector_dir(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => "native",
        MachineArchitecture::X86_64 => "x86_64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn concrete_selector_dir(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            other => panic!("unsupported native architecture '{other}'"),
        },
        MachineArchitecture::X86_64 => "x86_64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn architecture_flag(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => "native",
        MachineArchitecture::X86_64 => "x86-64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn protection_dir(mode: ProtectionMode) -> &'static str {
    match mode {
        ProtectionMode::Standard => "standard",
        ProtectionMode::Pvm => "pvm",
    }
}

fn hosted_artifact_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn reserve_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("port should bind");
    let addr = listener.local_addr().expect("addr should exist");
    drop(listener);
    addr.to_string()
}

fn wait_for_tcp_or_server_error(
    addr: &str,
    server_rx: &mpsc::Receiver<anyhow::Result<()>>,
    name: &str,
) {
    for _ in 0..100 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        if let Ok(result) = server_rx.try_recv() {
            match result {
                Ok(()) => panic!("{name} exited before becoming ready at '{addr}'"),
                Err(error) => panic!("{name} failed before becoming ready at '{addr}': {error}"),
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {name} listener at '{addr}'");
}

fn configure_kernel_paths(
    config: &mut PortConfig,
    local_root: &Path,
    store_root: &Path,
    cache_root: &Path,
    protection_mode: ProtectionMode,
) -> (PathBuf, PathBuf, PathBuf) {
    let kernel = config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("sample kernel should exist");
    kernel.distribution.push = ArtifactStore::FileSystem {
        root: store_root.to_path_buf(),
    };
    kernel.distribution.pull = ArtifactStore::FileSystem {
        root: store_root.to_path_buf(),
    };
    kernel.distribution.cache_root = cache_root.to_path_buf();

    for variant in &mut kernel.variants {
        variant.path = local_root
            .join(selector_dir(variant.selector.architecture))
            .join("firecracker")
            .join(protection_dir(variant.selector.protection_mode))
            .join("vmlinux");
    }

    let local_path = local_root
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let store_path = store_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");

    (local_path, cache_path, store_path)
}

fn configure_guest_paths(
    config: &mut PortConfig,
    local_root: &Path,
    store_root: &Path,
    cache_root: &Path,
    protection_mode: ProtectionMode,
) -> (PathBuf, PathBuf, PathBuf) {
    let guest = config
        .artifacts
        .guest_images
        .get_mut("demo-guest")
        .expect("sample guest image should exist");
    guest.distribution.push = ArtifactStore::FileSystem {
        root: store_root.to_path_buf(),
    };
    guest.distribution.pull = ArtifactStore::FileSystem {
        root: store_root.to_path_buf(),
    };
    guest.distribution.cache_root = cache_root.to_path_buf();

    for variant in &mut guest.variants {
        variant.path = local_root
            .join(selector_dir(variant.selector.architecture))
            .join("firecracker")
            .join(protection_dir(variant.selector.protection_mode))
            .join("rootfs.ext4");
    }

    let local_path = local_root
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("rootfs.ext4");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-guest")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("rootfs.ext4");
    let store_path = store_root
        .join("demo-fs")
        .join("port")
        .join("demo-guest")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("rootfs.ext4");

    (local_path, cache_path, store_path)
}

fn configure_hosted_kernel_paths(
    config: &mut PortConfig,
    local_root: &Path,
    cache_root: &Path,
    endpoint: &str,
    architecture: MachineArchitecture,
    protection_mode: ProtectionMode,
) -> (PathBuf, PathBuf, PathBuf) {
    let kernel = config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("sample kernel should exist");
    kernel.distribution.push = ArtifactStore::HostedApi {
        endpoint: endpoint.to_string(),
    };
    kernel.distribution.pull = ArtifactStore::HostedApi {
        endpoint: endpoint.to_string(),
    };
    kernel.distribution.cache_root = cache_root.to_path_buf();

    for variant in &mut kernel.variants {
        variant.path = local_root
            .join(selector_dir(variant.selector.architecture))
            .join("firecracker")
            .join(protection_dir(variant.selector.protection_mode))
            .join("vmlinux");
    }

    let selector_dir = concrete_selector_dir(architecture);
    let local_path = local_root
        .join(selector_dir)
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join(selector_dir)
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let store_path = PathBuf::from(".port/hosted/demo/artifacts")
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join(selector_dir)
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");

    (local_path, cache_path, store_path)
}

fn configure_oci_kernel_paths(
    config: &mut PortConfig,
    local_root: &Path,
    cache_root: &Path,
    protection_mode: ProtectionMode,
) -> (PathBuf, PathBuf, PathBuf) {
    let kernel = config
        .artifacts
        .kernels
        .get_mut("demo-kernel")
        .expect("sample kernel should exist");
    kernel.distribution.push = ArtifactStore::OciRegistry {
        transport: OciRegistryTransport::PlainHttp,
        auth: OciRegistryAuth::Anonymous,
    };
    kernel.distribution.cache_root = cache_root.to_path_buf();

    for variant in &mut kernel.variants {
        variant.path = local_root
            .join(selector_dir(variant.selector.architecture))
            .join("firecracker")
            .join(protection_dir(variant.selector.protection_mode))
            .join("vmlinux");
    }

    let local_path = local_root
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join(protection_dir(protection_mode))
        .join("vmlinux");
    let store_path = PathBuf::from(format!(
        "demo-fs/port/demo-kernel:v1-x86_64-firecracker-{}",
        protection_dir(protection_mode)
    ));

    (local_path, cache_path, store_path)
}

fn install_fake_oras_script(root: &Path, body: &str) -> PathBuf {
    let script_path = root.join("oras");
    fs::create_dir_all(root).expect("fake oras directory should exist");
    fs::write(&script_path, body).expect("fake oras script should write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&script_path)
            .expect("fake oras metadata should exist")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("fake oras permissions should update");
    }
    script_path
}

#[test]
fn cli_artifact_push_and_pull_round_trip_variant_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let store_root = temp.path().join("artifact-store");
    let cache_root = temp.path().join("artifact-cache");

    let mut config = PortConfig::sample();
    let (local_path, cache_path, store_path) = configure_kernel_paths(
        &mut config,
        &local_root,
        &store_root,
        &cache_root,
        ProtectionMode::Standard,
    );
    write_config(&config_path, &config);

    fs::create_dir_all(local_path.parent().expect("local parent"))
        .expect("local parent should exist");
    fs::write(&local_path, "demo-kernel-bytes").expect("local artifact should write");

    let push = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("push")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg("x86-64")
        .output()
        .expect("push command");
    assert!(push.status.success(), "{push:?}");
    assert!(
        String::from_utf8_lossy(&push.stdout).contains("demo-fs/port/demo-kernel:v1"),
        "push output did not include the canonical artifact reference: {}",
        String::from_utf8_lossy(&push.stdout)
    );
    assert_eq!(
        fs::read_to_string(&store_path).expect("store path should exist"),
        "demo-kernel-bytes"
    );
    assert_eq!(
        fs::read_to_string(&cache_path).expect("cache path should exist"),
        "demo-kernel-bytes"
    );

    fs::remove_file(&local_path).expect("local artifact should be removable");
    fs::remove_file(&cache_path).expect("cache artifact should be removable");

    let pull = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("pull")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg("x86-64")
        .output()
        .expect("pull command");
    assert!(pull.status.success(), "{pull:?}");
    assert!(
        String::from_utf8_lossy(&pull.stdout).contains(&store_path.display().to_string()),
        "pull output did not include the resolved store path: {}",
        String::from_utf8_lossy(&pull.stdout)
    );
    assert_eq!(
        fs::read_to_string(&local_path).expect("local path should be restored"),
        "demo-kernel-bytes"
    );
    assert_eq!(
        fs::read_to_string(&cache_path).expect("cache path should be restored"),
        "demo-kernel-bytes"
    );
}

#[test]
fn cli_artifact_build_push_and_pull_round_trip_through_hosted_backend() {
    let _guard = hosted_artifact_lock().lock().expect("lock should work");
    unsafe {
        std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
    }
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let cache_root = temp.path().join("artifact-cache");
    let control_plane_addr = reserve_addr();
    let endpoint = format!("http://{control_plane_addr}");
    let architecture = MachineArchitecture::Native;
    let selector_dir = concrete_selector_dir(architecture);
    let _ = fs::remove_dir_all(".port/hosted/demo");

    let mut config = PortConfig::sample();
    config
        .control_planes
        .get_mut("demo")
        .expect("demo control plane should exist")
        .endpoint = endpoint.clone();
    let (local_path, cache_path, store_path) = configure_hosted_kernel_paths(
        &mut config,
        &local_root,
        &cache_root,
        &endpoint,
        architecture,
        ProtectionMode::Standard,
    );
    write_config(&config_path, &config);

    let build = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("build")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg(architecture_flag(architecture))
        .output()
        .expect("build command");
    assert!(
        build.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(
        build_stdout.contains("built kernel artifact 'demo-kernel'"),
        "{build_stdout}"
    );
    assert!(
        build_stdout.contains(&format!("for {selector_dir}/firecracker/standard")),
        "{build_stdout}"
    );
    let built_bytes = fs::read(&local_path).expect("build should materialize local artifact");

    let (server_tx, server_rx) = mpsc::channel();
    let server_config = config.clone();
    thread::spawn(move || {
        let result = serve_control_plane(
            server_config,
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
                bind: control_plane_addr,
                node_bindings: Vec::new(),
            },
        )
        .map(|_| ());
        let _ = server_tx.send(result);
    });
    wait_for_tcp_or_server_error(&endpoint["http://".len()..], &server_rx, "control plane");

    let push = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("push")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg(architecture_flag(architecture))
        .output()
        .expect("push command");
    assert!(
        push.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&push.stdout),
        String::from_utf8_lossy(&push.stderr)
    );
    let push_stdout = String::from_utf8_lossy(&push.stdout);
    assert!(
        push_stdout.contains("demo-fs/port/demo-kernel:v1"),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&format!("for {selector_dir}/firecracker/standard")),
        "{push_stdout}"
    );
    assert!(push_stdout.contains("backend: hosted-api"), "{push_stdout}");
    assert!(
        push_stdout.contains(&local_path.display().to_string()),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&store_path.display().to_string()),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&cache_path.display().to_string()),
        "{push_stdout}"
    );
    assert_eq!(
        fs::read(&store_path).expect("store path should exist"),
        built_bytes
    );

    fs::remove_file(&local_path).expect("local artifact should be removable");
    fs::remove_file(&cache_path).expect("cache artifact should be removable");

    let pull = Command::new(port_bin())
        .env("PORT_DEMO_TOKEN", "demo-token")
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("pull")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg(architecture_flag(architecture))
        .output()
        .expect("pull command");
    assert!(
        pull.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&pull.stdout),
        String::from_utf8_lossy(&pull.stderr)
    );
    let pull_stdout = String::from_utf8_lossy(&pull.stdout);
    assert!(
        pull_stdout.contains("demo-fs/port/demo-kernel:v1"),
        "{pull_stdout}"
    );
    assert!(
        pull_stdout.contains(&format!("for {selector_dir}/firecracker/standard")),
        "{pull_stdout}"
    );
    assert!(pull_stdout.contains("backend: hosted-api"), "{pull_stdout}");
    assert!(
        pull_stdout.contains(&local_path.display().to_string()),
        "{pull_stdout}"
    );
    assert!(
        pull_stdout.contains(&store_path.display().to_string()),
        "{pull_stdout}"
    );
    assert!(
        pull_stdout.contains(&cache_path.display().to_string()),
        "{pull_stdout}"
    );
    assert_eq!(
        fs::read(&local_path).expect("local path should be restored"),
        built_bytes
    );
    assert_eq!(
        fs::read(&cache_path).expect("cache path should be restored"),
        built_bytes
    );

    let _ = fs::remove_dir_all(".port/hosted/demo");
}

#[test]
fn cli_artifact_push_oci_registry_reports_variant_and_backend_detail() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let cache_root = temp.path().join("artifact-cache");
    let fake_bin = temp.path().join("fake-bin");
    let args_log = temp.path().join("oras-args.log");
    install_fake_oras_script(
        &fake_bin,
        r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "${PORT_TEST_ORAS_ARGS:?}"
"#,
    );

    let mut config = PortConfig::sample();
    let (local_path, cache_path, store_path) =
        configure_oci_kernel_paths(&mut config, &local_root, &cache_root, ProtectionMode::Pvm);
    write_config(&config_path, &config);

    fs::create_dir_all(local_path.parent().expect("local parent"))
        .expect("local parent should exist");
    fs::write(&local_path, "demo-oci-pvm-kernel-bytes").expect("local artifact should write");

    let push = Command::new(port_bin())
        .env(
            "PATH",
            format!(
                "{}:{}",
                fake_bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("PORT_TEST_ORAS_ARGS", &args_log)
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("push")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--substrate")
        .arg("firecracker")
        .arg("--protection-mode")
        .arg("pvm")
        .output()
        .expect("push command");
    assert!(
        push.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&push.stdout),
        String::from_utf8_lossy(&push.stderr)
    );

    let push_stdout = String::from_utf8_lossy(&push.stdout);
    assert!(
        push_stdout.contains("demo-fs/port/demo-kernel:v1"),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains("for x86_64/firecracker/pvm"),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains("backend: oci-registry plain-http anonymous"),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&local_path.display().to_string()),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&store_path.display().to_string()),
        "{push_stdout}"
    );
    assert!(
        push_stdout.contains(&cache_path.display().to_string()),
        "{push_stdout}"
    );
    assert_eq!(
        fs::read_to_string(&cache_path).expect("cache path should exist"),
        "demo-oci-pvm-kernel-bytes"
    );

    let args = fs::read_to_string(&args_log).expect("args log should exist");
    assert!(args.contains("push"), "unexpected args: {args}");
    assert!(args.contains("--plain-http"), "unexpected args: {args}");
    assert!(
        args.contains("demo-fs/port/demo-kernel:v1-x86_64-firecracker-pvm"),
        "unexpected args: {args}"
    );
    assert!(
        args.contains("vmlinux:application/vnd.port.kernel.v1+binary"),
        "unexpected args: {args}"
    );
}

#[test]
fn cli_artifact_build_and_validate_selected_pvm_kernel_variant() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let store_root = temp.path().join("artifact-store");
    let cache_root = temp.path().join("artifact-cache");

    let mut config = PortConfig::sample();
    let (kernel_path, _, _) = configure_kernel_paths(
        &mut config,
        &local_root,
        &store_root,
        &cache_root,
        ProtectionMode::Pvm,
    );
    write_config(&config_path, &config);

    let build = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("build")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--substrate")
        .arg("firecracker")
        .arg("--protection-mode")
        .arg("pvm")
        .output()
        .expect("build command");
    assert!(
        build.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(
        build_stdout.contains("built kernel artifact 'demo-kernel'"),
        "{build_stdout}"
    );
    assert!(
        build_stdout.contains("for x86_64/firecracker/pvm"),
        "{build_stdout}"
    );
    assert!(
        build_stdout.contains(&kernel_path.display().to_string()),
        "{build_stdout}"
    );
    assert!(kernel_path.exists(), "pvm kernel path should exist");

    let validate = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("validate")
        .arg("--artifact")
        .arg("demo-kernel")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--substrate")
        .arg("firecracker")
        .arg("--protection-mode")
        .arg("pvm")
        .output()
        .expect("validate command");
    assert!(
        validate.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    let validate_stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(
        validate_stdout.contains("validated kernel artifact 'demo-kernel'"),
        "{validate_stdout}"
    );
    assert!(
        validate_stdout.contains("for x86_64/firecracker/pvm"),
        "{validate_stdout}"
    );
    assert!(
        validate_stdout.contains(&kernel_path.display().to_string()),
        "{validate_stdout}"
    );
}

#[test]
fn cli_artifact_build_and_validate_selected_pvm_guest_image_variant() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let store_root = temp.path().join("artifact-store");
    let cache_root = temp.path().join("artifact-cache");

    let mut config = PortConfig::sample();
    let (guest_path, _, _) = configure_guest_paths(
        &mut config,
        &local_root,
        &store_root,
        &cache_root,
        ProtectionMode::Pvm,
    );
    write_config(&config_path, &config);

    let build = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("build")
        .arg("--artifact")
        .arg("demo-guest")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--substrate")
        .arg("firecracker")
        .arg("--protection-mode")
        .arg("pvm")
        .output()
        .expect("build command");
    assert!(
        build.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(
        build_stdout.contains("built guest-image artifact 'demo-guest'"),
        "{build_stdout}"
    );
    assert!(
        build_stdout.contains("for x86_64/firecracker/pvm"),
        "{build_stdout}"
    );
    assert!(
        build_stdout.contains(&guest_path.display().to_string()),
        "{build_stdout}"
    );
    assert!(guest_path.exists(), "pvm guest-image path should exist");

    let validate = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("artifacts")
        .arg("validate")
        .arg("--artifact")
        .arg("demo-guest")
        .arg("--architecture")
        .arg("x86-64")
        .arg("--substrate")
        .arg("firecracker")
        .arg("--protection-mode")
        .arg("pvm")
        .output()
        .expect("validate command");
    assert!(
        validate.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    let validate_stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(
        validate_stdout.contains("validated guest-image artifact 'demo-guest'"),
        "{validate_stdout}"
    );
    assert!(
        validate_stdout.contains("for x86_64/firecracker/pvm"),
        "{validate_stdout}"
    );
    assert!(
        validate_stdout.contains(&guest_path.display().to_string()),
        "{validate_stdout}"
    );
}

#[test]
fn cli_artifact_push_and_pull_round_trip_pvm_variant_contract_for_kernel_and_guest_image() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let store_root = temp.path().join("artifact-store");
    let cache_root = temp.path().join("artifact-cache");

    let mut config = PortConfig::sample();
    let (kernel_local_path, kernel_cache_path, kernel_store_path) = configure_kernel_paths(
        &mut config,
        &local_root,
        &store_root,
        &cache_root,
        ProtectionMode::Pvm,
    );
    let (guest_local_path, guest_cache_path, guest_store_path) = configure_guest_paths(
        &mut config,
        &local_root,
        &store_root,
        &cache_root,
        ProtectionMode::Pvm,
    );
    write_config(&config_path, &config);

    fs::create_dir_all(kernel_local_path.parent().expect("kernel parent"))
        .expect("kernel parent should exist");
    fs::create_dir_all(guest_local_path.parent().expect("guest parent"))
        .expect("guest parent should exist");
    fs::write(&kernel_local_path, "demo-pvm-kernel-bytes").expect("kernel artifact should write");
    fs::write(&guest_local_path, "demo-pvm-guest-bytes").expect("guest artifact should write");

    for (artifact, local_path, cache_path, store_path, expected_bytes) in [
        (
            "demo-kernel",
            &kernel_local_path,
            &kernel_cache_path,
            &kernel_store_path,
            "demo-pvm-kernel-bytes",
        ),
        (
            "demo-guest",
            &guest_local_path,
            &guest_cache_path,
            &guest_store_path,
            "demo-pvm-guest-bytes",
        ),
    ] {
        let push = Command::new(port_bin())
            .arg("--config")
            .arg(&config_path)
            .arg("artifacts")
            .arg("push")
            .arg("--artifact")
            .arg(artifact)
            .arg("--architecture")
            .arg("x86-64")
            .arg("--substrate")
            .arg("firecracker")
            .arg("--protection-mode")
            .arg("pvm")
            .output()
            .expect("push command");
        assert!(
            push.status.success(),
            "stdout: {} stderr: {}",
            String::from_utf8_lossy(&push.stdout),
            String::from_utf8_lossy(&push.stderr)
        );
        let push_stdout = String::from_utf8_lossy(&push.stdout);
        assert!(
            push_stdout.contains("for x86_64/firecracker/pvm"),
            "{push_stdout}"
        );
        assert!(
            push_stdout.contains(&local_path.display().to_string()),
            "{push_stdout}"
        );
        assert!(
            push_stdout.contains(&store_path.display().to_string()),
            "{push_stdout}"
        );
        assert!(
            push_stdout.contains(&cache_path.display().to_string()),
            "{push_stdout}"
        );
        assert_eq!(
            fs::read_to_string(store_path).expect("store path should exist"),
            expected_bytes
        );
        assert_eq!(
            fs::read_to_string(cache_path).expect("cache path should exist"),
            expected_bytes
        );

        fs::remove_file(local_path).expect("local artifact should be removable");
        fs::remove_file(cache_path).expect("cache artifact should be removable");

        let pull = Command::new(port_bin())
            .arg("--config")
            .arg(&config_path)
            .arg("artifacts")
            .arg("pull")
            .arg("--artifact")
            .arg(artifact)
            .arg("--architecture")
            .arg("x86-64")
            .arg("--substrate")
            .arg("firecracker")
            .arg("--protection-mode")
            .arg("pvm")
            .output()
            .expect("pull command");
        assert!(
            pull.status.success(),
            "stdout: {} stderr: {}",
            String::from_utf8_lossy(&pull.stdout),
            String::from_utf8_lossy(&pull.stderr)
        );
        let pull_stdout = String::from_utf8_lossy(&pull.stdout);
        assert!(
            pull_stdout.contains("for x86_64/firecracker/pvm"),
            "{pull_stdout}"
        );
        assert!(
            pull_stdout.contains(&store_path.display().to_string()),
            "{pull_stdout}"
        );
        assert!(
            pull_stdout.contains(&cache_path.display().to_string()),
            "{pull_stdout}"
        );
        assert!(
            pull_stdout.contains(&local_path.display().to_string()),
            "{pull_stdout}"
        );
        assert_eq!(
            fs::read_to_string(local_path).expect("local path should be restored"),
            expected_bytes
        );
        assert_eq!(
            fs::read_to_string(cache_path).expect("cache path should be restored"),
            expected_bytes
        );
    }
}
