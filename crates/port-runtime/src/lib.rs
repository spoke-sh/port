use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    GuestOperation, OperationResult, RequestEnvelope, ResponseEnvelope, read_frame, write_frame,
};
use port_model::{ArtifactKind, HostConnection, HostPlatform, HostProvider, PortConfig};
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
    pub guest_agent_socket: PathBuf,
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
            guest_agent_socket: runtime_dir.join("guest-agent.sock"),
            runtime_dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuestRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub operation: GuestOperation,
}

#[derive(Debug, Clone)]
pub struct GuestCopyRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub source: &'a Path,
    pub destination: &'a Path,
    pub direction: port_agent_protocol::CopyDirection,
}

#[derive(Debug, Clone)]
pub struct GuestForwardRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub listen: &'a str,
    pub target: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAction {
    Build,
    Validate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMetadata {
    pub name: String,
    pub kind: ArtifactKind,
    pub path: PathBuf,
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
    checks.push(versioned_binary_check(
        "iproute2",
        "ip",
        &["-V"],
        "iproute2",
        local_firecracker_supported,
    ));
    checks.push(versioned_binary_check(
        "iptables",
        "iptables",
        &["--version"],
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

        for (name, host) in &config.hosts {
            if let Some(check) = provider_check(name, host.provider, &host.connection) {
                checks.push(check);
            }
        }
    }

    let mut notes = vec![
        String::from("port doctor reports the host state without mutating runtime directories."),
        String::from(
            "macOS operators should run Port on a Linux host because Firecracker local launch requires Linux and /dev/kvm.",
        ),
        String::from(
            "Windows operators should use WSL or a remote Linux host, then rely on port doctor to confirm whether local Firecracker launch is available.",
        ),
    ];
    if config.is_some() {
        notes.push(String::from(
            "Remote Linux hosts are modeled provider-by-provider, but the MVP launch path is still local Linux only.",
        ));
    }

    DoctorReport {
        host_os,
        local_firecracker_supported,
        checks,
        notes,
    }
}

pub fn build_artifact(config: &PortConfig, name: &str) -> Result<ArtifactMetadata> {
    run_artifact_pipeline(config, name, ArtifactAction::Build)
}

pub fn validate_artifact(config: &PortConfig, name: &str) -> Result<ArtifactMetadata> {
    run_artifact_pipeline(config, name, ArtifactAction::Validate)
}

pub fn launch_local_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
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
            "{}",
            remote_launch_guidance(request.machine_name, &machine.host, host.provider)
        );
    }

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
    prepare_runtime_state(&paths, request.machine_name)?;

    let config_payload = build_firecracker_config(
        kernel.path.clone(),
        guest_image.path.clone(),
        machine.vcpu_count,
        machine.memory_mib,
        machine.kernel_args.clone(),
        machine.rootfs_read_only,
        machine.guest.control_port,
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

fn prepare_runtime_state(paths: &RuntimePaths, machine_name: &str) -> Result<()> {
    if let Some(pid) = live_firecracker_pid(&paths.pid_path, machine_name)? {
        bail!(
            "machine '{}' already appears to be running with pid {} in '{}'; stop it first or choose a different --runtime-root",
            machine_name,
            pid,
            paths.runtime_dir.display()
        );
    }

    remove_stale_runtime_path(&paths.pid_path, "pid file")?;
    remove_stale_runtime_path(&paths.vsock_path, "vsock socket")?;
    remove_stale_runtime_path(&paths.guest_agent_socket, "guest-agent socket")?;

    Ok(())
}

fn live_firecracker_pid(pid_path: &Path, machine_name: &str) -> Result<Option<u32>> {
    let Some(pid) = read_pid_file(pid_path)? else {
        return Ok(None);
    };

    let Some(cmdline) = process_cmdline(pid)? else {
        return Ok(None);
    };

    let is_firecracker = cmdline.contains("firecracker");
    let matches_machine = cmdline.contains(&format!("--id {machine_name}"))
        || cmdline.contains(&format!("--id\0{machine_name}"));

    if is_firecracker && matches_machine {
        Ok(Some(pid))
    } else {
        Ok(None)
    }
}

fn read_pid_file(pid_path: &Path) -> Result<Option<u32>> {
    if !pid_path.exists() {
        return Ok(None);
    }

    let pid = fs::read_to_string(pid_path)
        .with_context(|| format!("failed to read pid file '{}'", pid_path.display()))?;
    let pid = pid
        .trim()
        .parse::<u32>()
        .with_context(|| format!("failed to parse pid file '{}'", pid_path.display()))?;

    Ok(Some(pid))
}

fn process_cmdline(pid: u32) -> Result<Option<String>> {
    let cmdline_path = PathBuf::from("/proc").join(pid.to_string()).join("cmdline");
    if !cmdline_path.exists() {
        return Ok(None);
    }

    let raw = fs::read(&cmdline_path).with_context(|| {
        format!(
            "failed to read process cmdline '{}'",
            cmdline_path.display()
        )
    })?;
    if raw.is_empty() {
        return Ok(None);
    }

    let rendered = raw
        .into_iter()
        .map(|byte| if byte == 0 { ' ' } else { byte as char })
        .collect();

    Ok(Some(rendered))
}

fn remove_stale_runtime_path(path: &Path, label: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
        .with_context(|| format!("failed to remove stale {label} '{}'", path.display()))
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

fn provider_check(
    host_name: &str,
    provider: HostProvider,
    connection: &HostConnection,
) -> Option<DoctorCheck> {
    if matches!(connection, HostConnection::Local) {
        return None;
    }

    let (ok, detail) = match provider {
        HostProvider::Local => (
            false,
            String::from(
                "provider 'local' is reserved for local Linux hosts; remote configs should use an explicit remote provider.",
            ),
        ),
        HostProvider::GenericLinux => (
            true,
            String::from(
                "provider 'generic-linux' is modeled for a future remote Linux control lane, but remote launch is not implemented in the MVP.",
            ),
        ),
        HostProvider::Aws => (
            true,
            String::from(
                "provider 'aws' remains a justified future Firecracker lane, but remote launch is not implemented in the MVP.",
            ),
        ),
        HostProvider::Gcp => (
            true,
            String::from(
                "provider 'gcp' remains a justified future Firecracker lane, but remote launch is not implemented in the MVP.",
            ),
        ),
        HostProvider::Azure => (
            false,
            String::from(
                "provider 'azure' is explicitly unsupported for the Firecracker MVP; do not expect a working launch path.",
            ),
        ),
    };

    Some(DoctorCheck {
        name: format!("host:{host_name}"),
        ok,
        required: false,
        detail,
    })
}

fn remote_launch_guidance(machine_name: &str, host_name: &str, provider: HostProvider) -> String {
    match provider {
        HostProvider::Local => format!(
            "machine '{machine_name}' targets host '{host_name}' through a remote connection, but provider 'local' is reserved for direct local Linux launch"
        ),
        HostProvider::GenericLinux => format!(
            "machine '{machine_name}' targets remote Linux host '{host_name}' (provider 'generic-linux'); the MVP only launches locally. Run Port on that Linux host directly or wait for the remote control lane."
        ),
        HostProvider::Aws => format!(
            "machine '{machine_name}' targets AWS host '{host_name}'; AWS remains a justified future Firecracker lane, but remote launch is not implemented in the MVP. Run Port on the AWS Linux host itself."
        ),
        HostProvider::Gcp => format!(
            "machine '{machine_name}' targets GCP host '{host_name}'; GCP remains a justified future Firecracker lane, but remote launch is not implemented in the MVP. Run Port on the GCP Linux host itself."
        ),
        HostProvider::Azure => format!(
            "machine '{machine_name}' targets Azure host '{host_name}'; Azure is explicitly unsupported for the Firecracker MVP. Move the workload to a generic Linux, AWS, or GCP host."
        ),
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

fn versioned_binary_check(
    name: &str,
    binary: &str,
    args: &[&str],
    needle: &str,
    required: bool,
) -> DoctorCheck {
    match find_binary(binary) {
        Some(path) => match Command::new(&path).args(args).output() {
            Ok(output) => {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if combined.contains(needle) {
                    DoctorCheck {
                        name: name.to_string(),
                        ok: true,
                        required,
                        detail: format!(
                            "Found '{binary}' at '{}' with expected identity.",
                            path.display()
                        ),
                    }
                } else {
                    DoctorCheck {
                        name: name.to_string(),
                        ok: false,
                        required,
                        detail: format!(
                            "Found '{binary}' at '{}', but version output did not contain '{needle}'.",
                            path.display()
                        ),
                    }
                }
            }
            Err(source) => DoctorCheck {
                name: name.to_string(),
                ok: false,
                required,
                detail: format!(
                    "Found '{binary}' at '{}', but failed to inspect it: {source}.",
                    path.display()
                ),
            },
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

fn run_artifact_pipeline(
    config: &PortConfig,
    name: &str,
    action: ArtifactAction,
) -> Result<ArtifactMetadata> {
    let (kind, spec) = config
        .artifacts
        .lookup_named(name)
        .with_context(|| format!("unknown artifact '{name}'"))?;
    let script = artifact_script(kind, action)?;

    let status = Command::new(&script)
        .arg(&spec.path)
        .current_dir(repo_root()?)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to start artifact pipeline '{}'", script.display()))?;

    if !status.success() {
        bail!(
            "artifact pipeline '{}' exited with status {status}",
            script.display()
        );
    }

    Ok(ArtifactMetadata {
        name: name.to_string(),
        kind,
        path: spec.path.clone(),
    })
}

fn artifact_script(kind: ArtifactKind, action: ArtifactAction) -> Result<PathBuf> {
    let script_name = match (kind, action) {
        (ArtifactKind::Kernel, ArtifactAction::Build) => "build-kernel.sh",
        (ArtifactKind::Kernel, ArtifactAction::Validate) => "validate-kernel.sh",
        (ArtifactKind::GuestImage, ArtifactAction::Build) => "build-guest-image.sh",
        (ArtifactKind::GuestImage, ArtifactAction::Validate) => "validate-guest-image.sh",
    };
    let path = repo_root()?.join("scripts/artifacts").join(script_name);
    if path.is_file() {
        Ok(path)
    } else {
        bail!("artifact pipeline script '{}' is missing", path.display())
    }
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("failed to derive repository root from CARGO_MANIFEST_DIR"))
}

pub fn execute_guest_operation(config: &PortConfig, request: GuestRequest<'_>) -> Result<OperationResult> {
    if matches!(
        &request.operation,
        GuestOperation::Copy(_) | GuestOperation::Forward(_)
    ) {
        bail!("copy and forward use dedicated runtime flows");
    }

    let endpoint = resolve_guest_endpoint(config, &request)?;
    let stream = connect_guest_endpoint(&endpoint)?;
    let writer_stream = stream
        .try_clone()
        .context("failed to clone guest agent socket")?;
    let mut writer = BufWriter::new(writer_stream);
    let mut reader = BufReader::new(stream);

    write_frame(
        &mut writer,
        &RequestEnvelope {
            id: 1,
            operation: request.operation,
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;

    let response: ResponseEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;

    match response {
        ResponseEnvelope::Completed {
            exit_code: 0,
            result,
            ..
        } => Ok(result),
        ResponseEnvelope::Completed {
            exit_code, result, ..
        } => {
            bail!("guest operation failed with exit code {exit_code}: {result:?}")
        }
        ResponseEnvelope::Failed { message, .. } => {
            bail!("guest agent returned an error: {message}")
        }
        ResponseEnvelope::Accepted { .. } => {
            bail!("streaming guest operations are not implemented yet")
        }
    }
}

pub fn copy_guest_file(
    config: &PortConfig,
    request: GuestCopyRequest<'_>,
) -> Result<port_agent_protocol::CopyResult> {
    let endpoint = resolve_guest_endpoint(
        config,
        &GuestRequest {
            machine_name: request.machine_name,
            runtime_root: request.runtime_root,
            operation: GuestOperation::Exec(port_agent_protocol::ExecRequest {
                command: vec![String::from("/bin/true")],
                cwd: None,
                env: Default::default(),
            }),
        },
    )?;
    let stream = connect_guest_endpoint(&endpoint)?;
    let writer_stream = stream
        .try_clone()
        .context("failed to clone guest agent socket")?;
    let mut writer = BufWriter::new(writer_stream);
    let mut reader = BufReader::new(stream);

    let size_bytes = match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => Some(
            fs::metadata(request.source)
                .with_context(|| format!("failed to stat '{}'", request.source.display()))?
                .len(),
        ),
        port_agent_protocol::CopyDirection::GuestToHost => None,
    };

    write_frame(
        &mut writer,
        &RequestEnvelope {
            id: 1,
            operation: GuestOperation::Copy(port_agent_protocol::CopyRequest {
                source: request.source.display().to_string(),
                destination: request.destination.display().to_string(),
                direction: request.direction,
                size_bytes,
            }),
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;

    match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            match read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))? {
                ResponseEnvelope::Accepted {
                    stream: port_agent_protocol::StreamKind::Bytes,
                    ..
                } => {}
                ResponseEnvelope::Failed { message, .. } => {
                    bail!("guest agent returned an error: {message}")
                }
                response => bail!("unexpected guest copy handshake response: {response:?}"),
            }

            let mut source = File::open(request.source)
                .with_context(|| format!("failed to open '{}'", request.source.display()))?;
            std::io::copy(&mut source, &mut writer)
                .with_context(|| format!("failed to stream '{}'", request.source.display()))?;
            writer.flush().context("failed to flush copy stream")?;
        }
        port_agent_protocol::CopyDirection::GuestToHost => {
            let size_bytes = match read_frame(&mut reader)
                .map_err(|error| anyhow!("protocol error: {error}"))?
            {
                ResponseEnvelope::Accepted {
                    stream: port_agent_protocol::StreamKind::Bytes,
                    size_bytes: Some(size_bytes),
                    ..
                } => size_bytes,
                ResponseEnvelope::Failed { message, .. } => {
                    bail!("guest agent returned an error: {message}")
                }
                response => bail!("unexpected guest copy handshake response: {response:?}"),
            };

            if let Some(parent) = request.destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            let mut destination = File::create(request.destination)
                .with_context(|| format!("failed to create '{}'", request.destination.display()))?;
            let mut limited = reader.by_ref().take(size_bytes);
            let bytes_copied = std::io::copy(&mut limited, &mut destination)
                .with_context(|| format!("failed to write '{}'", request.destination.display()))?;
            if bytes_copied != size_bytes {
                bail!(
                    "expected {size_bytes} bytes from guest copy, received {bytes_copied}"
                );
            }
        }
    }

    let response: ResponseEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;

    match response {
        ResponseEnvelope::Completed {
            exit_code: 0,
            result: OperationResult::Copy(result),
            ..
        } => Ok(result),
        ResponseEnvelope::Completed {
            exit_code, result, ..
        } => bail!("guest copy failed with exit code {exit_code}: {result:?}"),
        ResponseEnvelope::Failed { message, .. } => {
            bail!("guest agent returned an error: {message}")
        }
        ResponseEnvelope::Accepted { .. } => {
            bail!("unexpected second streaming response from guest copy")
        }
    }
}

pub struct GuestForwardSession {
    listener: TcpListener,
    endpoint: GuestEndpoint,
    target: String,
}

impl GuestForwardSession {
    #[must_use]
    pub fn listen_addr(&self) -> String {
        self.listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| String::from("<unknown>"))
    }

    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn serve(self) -> Result<()> {
        for inbound in self.listener.incoming() {
            let inbound = inbound.context("failed to accept forwarded host connection")?;
            let endpoint = self.endpoint.clone();
            let target = self.target.clone();
            thread::spawn(move || {
                if let Err(error) = proxy_guest_forward_connection(endpoint, target, inbound) {
                    eprintln!("port guest forward connection failed: {error}");
                }
            });
        }

        Ok(())
    }
}

pub fn prepare_guest_forward(
    config: &PortConfig,
    request: GuestForwardRequest<'_>,
) -> Result<GuestForwardSession> {
    let endpoint = resolve_guest_endpoint(
        config,
        &GuestRequest {
            machine_name: request.machine_name,
            runtime_root: request.runtime_root,
            operation: GuestOperation::Exec(port_agent_protocol::ExecRequest {
                command: vec![String::from("/bin/true")],
                cwd: None,
                env: Default::default(),
            }),
        },
    )?;
    let listener = TcpListener::bind(request.listen)
        .with_context(|| format!("failed to bind '{}'", request.listen))?;
    Ok(GuestForwardSession {
        listener,
        endpoint,
        target: request.target.to_string(),
    })
}

fn proxy_guest_forward_connection(
    endpoint: GuestEndpoint,
    target: String,
    inbound: TcpStream,
) -> Result<()> {
    let stream = connect_guest_endpoint(&endpoint)?;
    let writer_stream = stream
        .try_clone()
        .context("failed to clone guest transport stream")?;
    let mut writer = BufWriter::new(writer_stream);
    let mut reader = BufReader::new(stream);

    write_frame(
        &mut writer,
        &RequestEnvelope {
            id: 1,
            operation: GuestOperation::Forward(port_agent_protocol::ForwardRequest {
                listen: String::new(),
                target: target.clone(),
            }),
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;

    match read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))? {
        ResponseEnvelope::Accepted {
            stream: port_agent_protocol::StreamKind::Bytes,
            ..
        } => {}
        ResponseEnvelope::Failed { message, .. } => {
            bail!("guest agent returned an error: {message}")
        }
        response => bail!("unexpected guest forward handshake response: {response:?}"),
    };

    let buffered = reader.buffer().to_vec();
    let guest_stream = reader.into_inner();
    let mut guest_write = guest_stream
        .try_clone()
        .context("failed to clone guest forward stream")?;
    let mut guest_read = PrefixedReader::new(buffered, guest_stream);
    let mut inbound_read = inbound
        .try_clone()
        .context("failed to clone inbound forward socket")?;
    let mut inbound_write = inbound;

    let first = thread::spawn(move || {
        let result = std::io::copy(&mut inbound_read, &mut guest_write);
        let _ = guest_write.shutdown(Shutdown::Write);
        result
    });
    let second = thread::spawn(move || {
        let result = std::io::copy(&mut guest_read, &mut inbound_write);
        let _ = inbound_write.shutdown(Shutdown::Write);
        result
    });

    let _ = first.join();
    let _ = second.join();
    Ok(())
}

struct PrefixedReader<R> {
    prefix: Cursor<Vec<u8>>,
    inner: R,
}

impl<R> PrefixedReader<R> {
    fn new(prefix: Vec<u8>, inner: R) -> Self {
        Self {
            prefix: Cursor::new(prefix),
            inner,
        }
    }
}

impl<R: Read> Read for PrefixedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let prefix_bytes = self.prefix.read(buf)?;
        if prefix_bytes > 0 {
            return Ok(prefix_bytes);
        }

        self.inner.read(buf)
    }
}

#[derive(Debug, Clone)]
enum GuestEndpoint {
    RuntimeSocket(PathBuf),
    FirecrackerVsock {
        host_socket_path: PathBuf,
        guest_port: u32,
    },
}

fn resolve_guest_endpoint(config: &PortConfig, request: &GuestRequest<'_>) -> Result<GuestEndpoint> {
    let paths = RuntimePaths::for_machine(request.runtime_root, request.machine_name);
    if paths.guest_agent_socket.exists() {
        return Ok(GuestEndpoint::RuntimeSocket(paths.guest_agent_socket));
    }

    if paths.vsock_path.exists() {
        let machine = config
            .machines
            .get(request.machine_name)
            .with_context(|| format!("unknown machine '{}'", request.machine_name))?;
        return Ok(GuestEndpoint::FirecrackerVsock {
            host_socket_path: paths.vsock_path,
            guest_port: u32::from(machine.guest.control_port),
        });
    }

    if paths.manifest_path.exists() {
        bail!(
            "launched machine '{}' does not expose a live guest transport socket at '{}'; inspect the runtime logs or relaunch the VM",
            request.machine_name,
            paths.vsock_path.display()
        );
    }

    bail!(
        "guest agent socket '{}' does not exist for machine '{}'",
        paths.guest_agent_socket.display(),
        request.machine_name
    );
}

fn connect_guest_endpoint(endpoint: &GuestEndpoint) -> Result<UnixStream> {
    match endpoint {
        GuestEndpoint::RuntimeSocket(socket_path) => {
            UnixStream::connect(socket_path).with_context(|| {
                format!(
                    "failed to connect to guest agent socket '{}'",
                    socket_path.display()
                )
            })
        }
        GuestEndpoint::FirecrackerVsock {
            host_socket_path,
            guest_port,
        } => connect_firecracker_vsock(host_socket_path, *guest_port),
    }
}

fn connect_firecracker_vsock(host_socket_path: &Path, guest_port: u32) -> Result<UnixStream> {
    let mut stream = UnixStream::connect(host_socket_path).with_context(|| {
        format!(
            "failed to connect to Firecracker guest transport socket '{}'",
            host_socket_path.display()
        )
    })?;
    stream
        .write_all(format!("CONNECT {guest_port}\n").as_bytes())
        .with_context(|| {
            format!(
                "failed to request Firecracker guest transport port {} via '{}'",
                guest_port,
                host_socket_path.display()
            )
        })?;
    stream.flush().context("failed to flush Firecracker handshake")?;

    let reader_stream = stream
        .try_clone()
        .context("failed to clone Firecracker guest transport socket")?;
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();
    reader.read_line(&mut line).with_context(|| {
        format!(
            "failed to read Firecracker response from '{}'",
            host_socket_path.display()
        )
    })?;

    if !line.starts_with("OK") {
        let detail = line.trim();
        bail!(
            "Firecracker refused to establish a guest transport tunnel to port {} via '{}': {}",
            guest_port,
            host_socket_path.display(),
            if detail.is_empty() {
                "empty response"
            } else {
                detail
            }
        );
    }

    Ok(stream)
}

fn build_firecracker_config(
    kernel_image_path: PathBuf,
    rootfs_path: PathBuf,
    vcpu_count: u8,
    mem_size_mib: u32,
    boot_args: String,
    rootfs_read_only: bool,
    guest_control_port: u16,
    guest_cid: u32,
    uds_path: PathBuf,
) -> FirecrackerConfig {
    let boot_args = format!("{boot_args} init=/init port.guest_control_port={guest_control_port}");

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
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::os::unix::net::UnixListener;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    use tempfile::tempdir;

    use super::{
        ArtifactAction, DoctorCheck, GuestCopyRequest, GuestForwardRequest, GuestRequest,
        LaunchRequest, RuntimePaths, artifact_script, build_firecracker_config,
        collect_doctor_report, copy_guest_file, execute_guest_operation, launch_local_machine,
        path_check, prepare_guest_forward, prepare_runtime_state, read_pid_file, repo_root,
    };
    use port_agent_protocol::{
        CopyDirection, ExecRequest, ExecResult, GuestOperation, OperationResult, RequestEnvelope,
        ResponseEnvelope, StreamKind, read_frame, write_frame,
    };
    use port_model::{ArtifactKind, PortConfig};

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
            7000,
            52,
            "/tmp/guest.vsock".into(),
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(json.contains("\"boot-source\""));
        assert!(json.contains("\"/tmp/vmlinux\""));
        assert!(json.contains("\"rootfs\""));
        assert!(json.contains("\"guest_cid\": 52"));
        assert!(json.contains("init=/init"));
        assert!(json.contains("port.guest_control_port=7000"));
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

    #[test]
    fn artifact_scripts_resolve_from_repository_root() {
        let root = repo_root().expect("repo root should resolve");

        assert_eq!(
            artifact_script(ArtifactKind::Kernel, ArtifactAction::Build)
                .expect("kernel build script should resolve"),
            root.join("scripts/artifacts/build-kernel.sh")
        );
        assert_eq!(
            artifact_script(ArtifactKind::GuestImage, ArtifactAction::Validate)
                .expect("guest image validate script should resolve"),
            root.join("scripts/artifacts/validate-guest-image.sh")
        );
    }

    #[test]
    fn prepare_runtime_state_cleans_stale_socket_and_pid_files() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.pid_path, "0\n").expect("pid file should write");
        fs::write(&paths.vsock_path, "").expect("stale vsock placeholder should write");
        fs::write(&paths.guest_agent_socket, "").expect("stale guest socket should write");

        prepare_runtime_state(&paths, "demo").expect("stale runtime state should be cleaned");

        assert_eq!(
            read_pid_file(&paths.pid_path).expect("pid read should work"),
            None
        );
        assert!(!paths.vsock_path.exists());
        assert!(!paths.guest_agent_socket.exists());
    }

    #[test]
    fn prepare_runtime_state_rejects_live_matching_firecracker_process() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.vsock_path, "").expect("vsock placeholder should write");

        let mut command = Command::new("bash");
        command
            .args(["-lc", "exec -a firecracker /bin/sh -c 'sleep 30' --id demo"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .expect("fake firecracker process should start");
        fs::write(&paths.pid_path, format!("{}\n", child.id())).expect("pid file should write");
        thread::sleep(Duration::from_millis(100));

        let error = prepare_runtime_state(&paths, "demo")
            .expect_err("live matching firecracker should block relaunch");
        let message = error.to_string();
        assert!(message.contains("already appears to be running"));
        assert!(message.contains("stop it first"));
        assert!(paths.vsock_path.exists());

        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn doctor_report_includes_provider_aware_remote_host_checks() {
        let report = collect_doctor_report(Some(&PortConfig::sample()));

        let generic = report
            .checks
            .iter()
            .find(|check| check.name == "host:generic-linux")
            .expect("generic remote host check should exist");
        let aws = report
            .checks
            .iter()
            .find(|check| check.name == "host:aws-linux")
            .expect("aws host check should exist");
        let gcp = report
            .checks
            .iter()
            .find(|check| check.name == "host:gcp-linux")
            .expect("gcp host check should exist");
        let azure = report
            .checks
            .iter()
            .find(|check| check.name == "host:azure-linux")
            .expect("azure host check should exist");

        assert!(generic.ok);
        assert!(generic.detail.contains("generic-linux"));
        assert!(aws.ok);
        assert!(aws.detail.contains("future Firecracker lane"));
        assert!(gcp.ok);
        assert!(gcp.detail.contains("future Firecracker lane"));
        assert!(!azure.ok);
        assert!(azure.detail.contains("unsupported"));
        assert!(
            report
                .notes
                .iter()
                .any(|note| note.contains("local Linux only"))
        );
    }

    #[test]
    fn remote_launch_rejects_aws_hosts_with_provider_guidance() {
        let tempdir = tempdir().expect("tempdir should exist");
        let error = launch_local_machine(
            &PortConfig::sample(),
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("remote AWS launch should fail fast");

        let message = error.to_string();
        assert!(message.contains("cloud-aws"));
        assert!(message.contains("AWS"));
        assert!(message.contains("not implemented"));
        assert!(message.contains("Run Port on the AWS Linux host itself"));
    }

    #[test]
    fn guest_operations_explain_missing_live_vm_transport_socket() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.manifest_path, "{}\n").expect("manifest marker should write");

        let error = execute_guest_operation(&PortConfig::sample(), GuestRequest {
            machine_name: "demo",
            runtime_root: tempdir.path(),
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("true"),
                ],
                cwd: None,
                env: Default::default(),
            }),
        })
        .expect_err("missing guest socket should fail");

        let message = error.to_string();
        assert!(message.contains("does not expose a live guest transport socket"));
        assert!(message.contains("relaunch the VM"));
    }

    #[test]
    fn guest_exec_uses_firecracker_vsock_tunnel_when_runtime_socket_is_absent() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener = UnixListener::bind(&paths.vsock_path).expect("vsock listener should bind");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("should accept guest transport");
            let reader_stream = stream.try_clone().expect("stream should clone");
            let mut reader = BufReader::new(reader_stream);

            let mut handshake = String::new();
            reader
                .read_line(&mut handshake)
                .expect("handshake line should read");
            assert_eq!(handshake, "CONNECT 7000\n");
            stream.write_all(b"OK\n").expect("should acknowledge handshake");
            stream.flush().expect("should flush handshake response");

            let request: RequestEnvelope = read_frame(&mut reader).expect("request should decode");
            match request.operation {
                GuestOperation::Exec(request) => {
                    assert_eq!(request.command, vec![String::from("/bin/echo"), String::from("live-ok")]);
                }
                other => panic!("unexpected operation over live guest transport: {other:?}"),
            }

            write_frame(
                &mut stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Exec(ExecResult {
                        stdout: String::from("live-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("response should encode");
        });

        let result = execute_guest_operation(
            &PortConfig::sample(),
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("live-ok")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("live guest exec should succeed");

        match result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "live-ok\n"),
            other => panic!("unexpected result: {other:?}"),
        }

        server.join().expect("server thread should complete");
    }

    #[test]
    fn copy_guest_file_uses_firecracker_vsock_tunnel_in_both_directions() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");

        let host_source = tempdir.path().join("host.txt");
        fs::write(&host_source, "copy-ok").expect("host source should write");
        let host_destination = tempdir.path().join("downloaded.txt");
        let host_destination_for_server = host_destination.clone();

        let listener = UnixListener::bind(&paths.vsock_path).expect("vsock listener should bind");
        let server = thread::spawn(move || {
            let (mut upload_stream, _) = listener.accept().expect("upload accept");
            let upload_reader_stream = upload_stream.try_clone().expect("upload clone");
            let mut upload_reader = BufReader::new(upload_reader_stream);
            let mut handshake = String::new();
            upload_reader
                .read_line(&mut handshake)
                .expect("upload handshake should read");
            assert_eq!(handshake, "CONNECT 7000\n");
            upload_stream.write_all(b"OK\n").expect("upload ack");
            let upload_request: RequestEnvelope =
                read_frame(&mut upload_reader).expect("upload request should decode");
            let GuestOperation::Copy(upload_request) = upload_request.operation else {
                panic!("unexpected upload operation");
            };
            assert_eq!(upload_request.direction, CopyDirection::HostToGuest);
            assert_eq!(upload_request.size_bytes, Some(7));
            write_frame(
                &mut upload_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: None,
                },
            )
            .expect("upload accepted should encode");
            let mut uploaded = Vec::new();
            upload_reader
                .by_ref()
                .take(7)
                .read_to_end(&mut uploaded)
                .expect("upload bytes should read");
            assert_eq!(uploaded, b"copy-ok");
            write_frame(
                &mut upload_stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Copy(port_agent_protocol::CopyResult {
                        bytes_copied: 7,
                        path: String::from("/workspace/copied.txt"),
                        direction: CopyDirection::HostToGuest,
                    }),
                },
            )
            .expect("upload completion should encode");
            drop(upload_stream);

            let (mut download_stream, _) = listener.accept().expect("download accept");
            let download_reader_stream = download_stream.try_clone().expect("download clone");
            let mut download_reader = BufReader::new(download_reader_stream);
            let mut handshake = String::new();
            download_reader
                .read_line(&mut handshake)
                .expect("download handshake should read");
            assert_eq!(handshake, "CONNECT 7000\n");
            download_stream.write_all(b"OK\n").expect("download ack");
            let download_request: RequestEnvelope =
                read_frame(&mut download_reader).expect("download request should decode");
            let GuestOperation::Copy(download_request) = download_request.operation else {
                panic!("unexpected download operation");
            };
            assert_eq!(download_request.direction, CopyDirection::GuestToHost);
            write_frame(
                &mut download_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: Some(7),
                },
            )
            .expect("download accepted should encode");
            download_stream
                .write_all(b"copy-ok")
                .expect("download bytes should write");
            write_frame(
                &mut download_stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Copy(port_agent_protocol::CopyResult {
                        bytes_copied: 7,
                        path: host_destination_for_server.display().to_string(),
                        direction: CopyDirection::GuestToHost,
                    }),
                },
            )
            .expect("download completion should encode");
        });

        let upload = copy_guest_file(
            &PortConfig::sample(),
            GuestCopyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                source: &host_source,
                destination: Path::new("/workspace/copied.txt"),
                direction: CopyDirection::HostToGuest,
            },
        )
        .expect("upload should succeed");
        assert_eq!(upload.bytes_copied, 7);

        let download = copy_guest_file(
            &PortConfig::sample(),
            GuestCopyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                source: Path::new("/workspace/copied.txt"),
                destination: &host_destination,
                direction: CopyDirection::GuestToHost,
            },
        )
        .expect("download should succeed");
        assert_eq!(download.bytes_copied, 7);
        assert_eq!(download.path, host_destination.display().to_string());
        assert_eq!(
            fs::read_to_string(&host_destination).expect("downloaded file should read"),
            "copy-ok"
        );

        server.join().expect("copy server thread should complete");
    }

    #[test]
    fn guest_forward_session_proxies_through_firecracker_vsock_tunnel() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener = UnixListener::bind(&paths.vsock_path).expect("vsock listener should bind");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("forward accept");
            let reader_stream = stream.try_clone().expect("forward clone");
            let mut reader = BufReader::new(reader_stream);
            let mut handshake = String::new();
            reader
                .read_line(&mut handshake)
                .expect("forward handshake should read");
            assert_eq!(handshake, "CONNECT 7000\n");
            stream.write_all(b"OK\n").expect("forward ack");
            let request: RequestEnvelope = read_frame(&mut reader).expect("forward request");
            let GuestOperation::Forward(request) = request.operation else {
                panic!("unexpected forward operation");
            };
            assert_eq!(request.target, "127.0.0.1:8081");
            write_frame(
                &mut stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: None,
                },
            )
            .expect("forward accepted should encode");
            stream
                .write_all(b"ready")
                .expect("forward eager bytes should write");
            stream.flush().expect("forward eager bytes should flush");
            let mut echoed = [0_u8; 16];
            let len = reader.read(&mut echoed).expect("forward bytes should read");
            stream
                .write_all(&echoed[..len])
                .expect("forward bytes should echo");
        });

        let session = prepare_guest_forward(
            &PortConfig::sample(),
            GuestForwardRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                listen: "127.0.0.1:0",
                target: "127.0.0.1:8081",
            },
        )
        .expect("forward session should prepare");
        let listen_addr = session.listen_addr();
        let serve_thread = thread::spawn(move || session.serve().expect("forward serve should run"));

        let mut forwarded: Option<TcpStream> = None;
        for _ in 0..100 {
            match TcpStream::connect(&listen_addr) {
                Ok(stream) => {
                    forwarded = Some(stream);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
        let mut forwarded = forwarded.expect("should connect to forwarded listener");
        let mut eager = [0_u8; 5];
        forwarded
            .read_exact(&mut eager)
            .expect("forward eager bytes should read");
        assert_eq!(&eager, b"ready");
        forwarded.write_all(b"forward-ok").expect("forward write");
        forwarded.shutdown(Shutdown::Write).expect("forward shutdown");
        let mut echoed = Vec::new();
        forwarded
            .read_to_end(&mut echoed)
            .expect("forward read should complete");
        assert_eq!(echoed, b"forward-ok");

        let _ = serve_thread.thread().id();
        server.join().expect("forward server thread should complete");
    }
}
