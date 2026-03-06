use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use port_model::{HostConnection, HostPlatform, PortConfig};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub host_os: String,
    pub local_firecracker_supported: bool,
    pub checks: Vec<DoctorCheck>,
    pub notes: Vec<String>,
}

impl DoctorReport {
    #[must_use]
    pub fn blocking_failures(&self) -> Vec<&DoctorCheck> {
        self.checks
            .iter()
            .filter(|check| check.required && !check.ok)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub required: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct LaunchRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub boot_wait: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaunchMetadata {
    pub machine_name: String,
    pub pid: u32,
    pub launched_at_unix_s: u64,
    pub runtime_dir: PathBuf,
    pub firecracker_binary: PathBuf,
    pub config_path: PathBuf,
    pub log_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    pub runtime_dir: PathBuf,
    pub config_path: PathBuf,
    pub firecracker_log: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub manifest_path: PathBuf,
    pub pid_path: PathBuf,
    pub vsock_path: PathBuf,
}

impl RuntimePaths {
    #[must_use]
    pub fn for_machine(runtime_root: impl AsRef<Path>, machine_name: &str) -> Self {
        let runtime_dir = runtime_root.as_ref().join(machine_name);

        Self {
            config_path: runtime_dir.join("firecracker-config.json"),
            firecracker_log: runtime_dir.join("firecracker.log"),
            stdout_log: runtime_dir.join("console.stdout.log"),
            stderr_log: runtime_dir.join("console.stderr.log"),
            manifest_path: runtime_dir.join("manifest.json"),
            pid_path: runtime_dir.join("firecracker.pid"),
            vsock_path: runtime_dir.join("guest.vsock"),
            runtime_dir,
        }
    }
}

pub fn collect_doctor_report(config: Option<&PortConfig>) -> DoctorReport {
    let host_os = env::consts::OS.to_string();
    let local_firecracker_supported = host_os == "linux";
    let mut checks = Vec::new();

    checks.push(DoctorCheck {
        name: String::from("host-platform"),
        ok: local_firecracker_supported,
        required: true,
        detail: if local_firecracker_supported {
            String::from("Local Firecracker launch is available on Linux hosts.")
        } else {
            format!(
                "Local Firecracker launch is unsupported on {host_os}; use a remote Linux host."
            )
        },
    });

    checks.push(path_check(
        "kvm-device",
        Path::new("/dev/kvm"),
        local_firecracker_supported,
        "Found /dev/kvm for KVM acceleration.",
        "Missing /dev/kvm.",
    ));
    checks.push(binary_check(
        "firecracker-binary",
        "firecracker",
        local_firecracker_supported,
    ));
    checks.push(binary_check("iproute2", "ip", local_firecracker_supported));
    checks.push(binary_check(
        "iptables",
        "iptables",
        local_firecracker_supported,
    ));

    if let Some(config) = config {
        for (name, artifact) in config.artifacts.all() {
            checks.push(path_check(
                format!("artifact:{name}"),
                &artifact.path,
                true,
                &format!("Artifact path '{}' exists.", artifact.path.display()),
                &format!(
                    "Artifact path '{}' is missing. Build or fetch the artifact first.",
                    artifact.path.display()
                ),
            ));
        }
    }

    let notes = vec![
        String::from("port doctor reports the host state without mutating runtime directories."),
        String::from(
            "macOS and Windows operators should target remote Linux hosts for Firecracker execution.",
        ),
    ];

    DoctorReport {
        host_os,
        local_firecracker_supported,
        checks,
        notes,
    }
}

pub fn launch_local_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    let report = collect_doctor_report(Some(config));
    let failures = report.blocking_failures();
    if !failures.is_empty() {
        let details = failures
            .into_iter()
            .map(|failure| format!("{}: {}", failure.name, failure.detail))
            .collect::<Vec<_>>()
            .join("; ");
        bail!("host preflight failed: {details}");
    }

    let machine = config
        .machines
        .get(request.machine_name)
        .with_context(|| format!("unknown machine '{}'", request.machine_name))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;

    if host.platform != HostPlatform::Linux {
        bail!(
            "machine '{}' targets host '{}' with platform {:?}; local launch requires a Linux host",
            request.machine_name,
            machine.host,
            host.platform
        );
    }

    if !matches!(host.connection, HostConnection::Local) {
        bail!(
            "machine '{}' targets host '{}' via a remote connection; local launch requires connection.mode = local",
            request.machine_name,
            machine.host
        );
    }

    let kernel = config
        .artifact(&machine.kernel)
        .with_context(|| format!("unknown kernel artifact '{}'", machine.kernel))?;
    let guest_image = config
        .artifact(&machine.guest_image)
        .with_context(|| format!("unknown guest image artifact '{}'", machine.guest_image))?;
    let firecracker_binary = find_binary("firecracker")
        .context("firecracker binary was not found on PATH after preflight")?;

    let paths = RuntimePaths::for_machine(request.runtime_root, request.machine_name);
    fs::create_dir_all(&paths.runtime_dir).with_context(|| {
        format!(
            "failed to create runtime directory '{}'",
            paths.runtime_dir.display()
        )
    })?;

    let config_payload = build_firecracker_config(
        kernel.path.clone(),
        guest_image.path.clone(),
        machine.vcpu_count,
        machine.memory_mib,
        machine.kernel_args.clone(),
        machine.rootfs_read_only,
        machine.guest.vsock_cid,
        paths.vsock_path.clone(),
    );
    let config_json =
        serde_json::to_string_pretty(&config_payload).context("failed to encode config JSON")?;
    fs::write(&paths.config_path, format!("{config_json}\n")).with_context(|| {
        format!(
            "failed to write Firecracker config '{}'",
            paths.config_path.display()
        )
    })?;

    let stdout = File::create(&paths.stdout_log)
        .with_context(|| format!("failed to create '{}'", paths.stdout_log.display()))?;
    let stderr = File::create(&paths.stderr_log)
        .with_context(|| format!("failed to create '{}'", paths.stderr_log.display()))?;

    let mut child = Command::new(&firecracker_binary)
        .arg("--no-api")
        .arg("--id")
        .arg(request.machine_name)
        .arg("--config-file")
        .arg(&paths.config_path)
        .arg("--log-path")
        .arg(&paths.firecracker_log)
        .arg("--level")
        .arg("Info")
        .arg("--show-level")
        .arg("--show-log-origin")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("failed to start '{}'", firecracker_binary.display()))?;

    if let Some(status) = wait_for_boot(&mut child, request.boot_wait)? {
        bail!(
            "firecracker exited before boot wait elapsed with status {status}; inspect '{}' and '{}'",
            paths.stdout_log.display(),
            paths.stderr_log.display()
        );
    }

    let launched_at_unix_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_secs();

    fs::write(&paths.pid_path, format!("{}\n", child.id()))
        .with_context(|| format!("failed to write pid file '{}'", paths.pid_path.display()))?;

    let metadata = LaunchMetadata {
        machine_name: request.machine_name.to_string(),
        pid: child.id(),
        launched_at_unix_s,
        runtime_dir: paths.runtime_dir.clone(),
        firecracker_binary,
        config_path: paths.config_path.clone(),
        log_path: paths.firecracker_log.clone(),
        stdout_path: paths.stdout_log.clone(),
        stderr_path: paths.stderr_log.clone(),
        manifest_path: paths.manifest_path.clone(),
    };

    let manifest = serde_json::to_string_pretty(&metadata).context("failed to encode manifest")?;
    fs::write(&paths.manifest_path, format!("{manifest}\n")).with_context(|| {
        format!(
            "failed to write manifest '{}'",
            paths.manifest_path.display()
        )
    })?;

    Ok(metadata)
}

fn wait_for_boot(
    child: &mut std::process::Child,
    boot_wait: Duration,
) -> Result<Option<std::process::ExitStatus>> {
    let step = Duration::from_millis(200);
    let mut waited = Duration::ZERO;

    while waited < boot_wait {
        if let Some(status) = child
            .try_wait()
            .context("failed to poll Firecracker process")?
        {
            return Ok(Some(status));
        }
        thread::sleep(step);
        waited += step;
    }

    child
        .try_wait()
        .context("failed to poll Firecracker process after boot wait")
}

fn path_check(
    name: impl Into<String>,
    path: &Path,
    required: bool,
    ok_detail: &str,
    fail_detail: &str,
) -> DoctorCheck {
    DoctorCheck {
        name: name.into(),
        ok: path.exists(),
        required,
        detail: if path.exists() {
            ok_detail.to_string()
        } else {
            fail_detail.to_string()
        },
    }
}

fn binary_check(name: &str, binary: &str, required: bool) -> DoctorCheck {
    match find_binary(binary) {
        Some(path) => DoctorCheck {
            name: name.to_string(),
            ok: true,
            required,
            detail: format!("Found '{binary}' at '{}'.", path.display()),
        },
        None => DoctorCheck {
            name: name.to_string(),
            ok: false,
            required,
            detail: format!("Missing '{binary}' on PATH."),
        },
    }
}

fn find_binary(binary: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    env::split_paths(&path)
        .map(|entry| entry.join(binary))
        .find(|candidate| candidate.is_file())
}

fn build_firecracker_config(
    kernel_image_path: PathBuf,
    rootfs_path: PathBuf,
    vcpu_count: u8,
    mem_size_mib: u32,
    boot_args: String,
    rootfs_read_only: bool,
    guest_cid: u32,
    uds_path: PathBuf,
) -> FirecrackerConfig {
    FirecrackerConfig {
        boot_source: BootSourceConfig {
            kernel_image_path,
            boot_args,
        },
        drives: vec![DriveConfig {
            drive_id: String::from("rootfs"),
            path_on_host: rootfs_path,
            is_root_device: true,
            is_read_only: rootfs_read_only,
        }],
        machine_config: MachineConfig {
            vcpu_count,
            mem_size_mib,
            smt: false,
        },
        vsock: VsockConfig {
            guest_cid,
            uds_path,
        },
    }
}

#[derive(Debug, Serialize)]
struct FirecrackerConfig {
    #[serde(rename = "boot-source")]
    boot_source: BootSourceConfig,
    drives: Vec<DriveConfig>,
    #[serde(rename = "machine-config")]
    machine_config: MachineConfig,
    vsock: VsockConfig,
}

#[derive(Debug, Serialize)]
struct BootSourceConfig {
    kernel_image_path: PathBuf,
    boot_args: String,
}

#[derive(Debug, Serialize)]
struct DriveConfig {
    drive_id: String,
    path_on_host: PathBuf,
    is_root_device: bool,
    is_read_only: bool,
}

#[derive(Debug, Serialize)]
struct MachineConfig {
    vcpu_count: u8,
    mem_size_mib: u32,
    smt: bool,
}

#[derive(Debug, Serialize)]
struct VsockConfig {
    guest_cid: u32,
    uds_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{DoctorCheck, RuntimePaths, build_firecracker_config, path_check};

    #[test]
    fn runtime_paths_are_deterministic() {
        let paths = RuntimePaths::for_machine("/tmp/port-runtime", "demo");

        assert_eq!(paths.runtime_dir, Path::new("/tmp/port-runtime/demo"));
        assert_eq!(
            paths.config_path,
            Path::new("/tmp/port-runtime/demo/firecracker-config.json")
        );
        assert_eq!(
            paths.manifest_path,
            Path::new("/tmp/port-runtime/demo/manifest.json")
        );
    }

    #[test]
    fn firecracker_config_contains_kernel_rootfs_and_vsock() {
        let config = build_firecracker_config(
            "/tmp/vmlinux".into(),
            "/tmp/rootfs.ext4".into(),
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            false,
            52,
            "/tmp/guest.vsock".into(),
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(json.contains("\"boot-source\""));
        assert!(json.contains("\"/tmp/vmlinux\""));
        assert!(json.contains("\"rootfs\""));
        assert!(json.contains("\"guest_cid\": 52"));
    }

    #[test]
    fn path_checks_report_missing_artifacts() {
        let tempdir = tempdir().expect("tempdir should exist");
        let existing = tempdir.path().join("present");
        fs::write(&existing, "ok").expect("artifact should be writable");

        let existing_check = path_check("artifact:present", &existing, true, "present", "missing");
        let missing_check = path_check(
            "artifact:missing",
            &tempdir.path().join("missing"),
            true,
            "present",
            "missing",
        );

        assert_eq!(
            existing_check,
            DoctorCheck {
                name: String::from("artifact:present"),
                ok: true,
                required: true,
                detail: String::from("present"),
            }
        );
        assert!(!missing_check.ok);
    }
}
