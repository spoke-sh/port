use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use port_model::{ArtifactStore, MachineArchitecture, PortConfig};
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
