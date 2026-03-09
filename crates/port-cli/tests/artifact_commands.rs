use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

use port_model::{ArtifactStore, MachineArchitecture, PortConfig};
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
            .join("standard")
            .join("vmlinux");
    }

    let local_path = local_root
        .join("x86_64")
        .join("firecracker")
        .join("standard")
        .join("vmlinux");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join("standard")
        .join("vmlinux");
    let store_path = store_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join("x86_64")
        .join("firecracker")
        .join("standard")
        .join("vmlinux");

    (local_path, cache_path, store_path)
}

fn configure_hosted_kernel_paths(
    config: &mut PortConfig,
    local_root: &Path,
    cache_root: &Path,
    endpoint: &str,
    architecture: MachineArchitecture,
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
            .join("standard")
            .join("vmlinux");
    }

    let selector_dir = concrete_selector_dir(architecture);
    let local_path = local_root
        .join(selector_dir)
        .join("firecracker")
        .join("standard")
        .join("vmlinux");
    let cache_path = cache_root
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join(selector_dir)
        .join("firecracker")
        .join("standard")
        .join("vmlinux");
    let store_path = PathBuf::from(".port/hosted/demo/artifacts")
        .join("demo-fs")
        .join("port")
        .join("demo-kernel")
        .join("v1")
        .join(selector_dir)
        .join("firecracker")
        .join("standard")
        .join("vmlinux");

    (local_path, cache_path, store_path)
}

#[test]
fn cli_artifact_push_and_pull_round_trip_variant_contract() {
    let temp = tempdir().expect("tempdir should exist");
    let config_path = temp.path().join("port.toml");
    let local_root = temp.path().join("local-artifacts");
    let store_root = temp.path().join("artifact-store");
    let cache_root = temp.path().join("artifact-cache");

    let mut config = PortConfig::sample();
    let (local_path, cache_path, store_path) =
        configure_kernel_paths(&mut config, &local_root, &store_root, &cache_root);
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
