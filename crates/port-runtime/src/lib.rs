use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    ForwardEndpoint, GuestOperation, OperationResult, RequestEnvelope, ResponseEnvelope,
    parse_forward_endpoint, read_frame, render_forward_endpoint, write_frame,
};
use port_hosted_protocol::HostedSuccess;
use port_model::{
    ArtifactKind, ArtifactReference, ArtifactSelector, ArtifactStore, ArtifactVariant,
    ExecutionSubstrate, HostConnection, HostPlatform, HostProvider, HostedApiIdentityContract,
    MachineArchitecture, MachineControlContract, PortConfig, ProtectionMode, PvmHostKit,
};
use port_sdk::HostedClient;
use serde::{Deserialize, Serialize};

mod hosted_control_plane;

pub use hosted_control_plane::{
    ControlPlaneServeRequest, HostedNodeBinding, NodeAgentServeRequest, serve_control_plane,
    serve_node_agent,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorHostFacts {
    host_os: String,
    host_architecture: String,
    proc_cmdline: Option<String>,
    pvm_firecracker_binary: Option<PathBuf>,
}

impl DoctorHostFacts {
    fn collect() -> Self {
        Self {
            host_os: env::consts::OS.to_string(),
            host_architecture: env::consts::ARCH.to_string(),
            proc_cmdline: fs::read_to_string("/proc/cmdline").ok(),
            pvm_firecracker_binary: find_pvm_firecracker_binary(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LaunchRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub boot_wait: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeState {
    Running,
    Stopped,
    Stale,
    Malformed,
}

impl std::fmt::Display for MachineRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Stale => "stale",
            Self::Malformed => "malformed",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineStatus {
    pub machine_name: String,
    pub state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub control: MachineControlContract,
    pub runtime_dir: PathBuf,
    pub config_path: PathBuf,
    pub manifest_path: PathBuf,
    pub pid_path: PathBuf,
    pub firecracker_log: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopResult {
    pub machine_name: String,
    pub previous_state: MachineRuntimeState,
    pub current_state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub control: MachineControlContract,
    pub runtime_dir: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetachedForwardStatus {
    pub name: String,
    pub state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub listen: String,
    pub target: String,
    pub manifest_path: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineMonitorReport {
    pub machine_name: String,
    pub state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub runtime_dir: PathBuf,
    pub config_path: PathBuf,
    pub manifest_path: PathBuf,
    pub pid_path: PathBuf,
    pub firecracker_log: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub detached_forwards: Vec<DetachedForwardStatus>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineTopEntryKind {
    Hypervisor,
    DetachedForward,
}

impl std::fmt::Display for MachineTopEntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hypervisor => f.write_str("hypervisor"),
            Self::DetachedForward => f.write_str("detached-forward"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineTopEntry {
    pub kind: MachineTopEntryKind,
    pub name: String,
    pub state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub command: Option<String>,
    pub source: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineTopReport {
    pub machine_name: String,
    pub state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub runtime_dir: PathBuf,
    pub detail: String,
    pub entries: Vec<MachineTopEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Service,
    Sandbox,
}

impl std::fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service => f.write_str("service"),
            Self::Sandbox => f.write_str("sandbox"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceDesiredState {
    Active,
    Stopped,
}

impl std::fmt::Display for ServiceDesiredState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => f.write_str("active"),
            Self::Stopped => f.write_str("stopped"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSecretBinding {
    pub env: String,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct SecretPutRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Clone)]
pub struct ServiceApplyRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub name: &'a str,
    pub kind: ServiceKind,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MachineSecretSummary {
    pub machine_name: String,
    pub name: String,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub path: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceDefinitionStatus {
    pub machine_name: String,
    pub name: String,
    pub kind: ServiceKind,
    pub desired_state: ServiceDesiredState,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub manifest_path: PathBuf,
    pub detail: String,
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
pub enum MachineDriverKind {
    FirecrackerLocal,
    HostedControlPlane,
}

trait MachineDriver {
    #[allow(dead_code)]
    fn kind(&self) -> MachineDriverKind;

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata>;

    fn list_machines(&self, config: &PortConfig, runtime_root: &Path)
    -> Result<Vec<MachineStatus>>;

    fn machine_status(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus>;

    fn stop_machine(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult>;

    fn machine_monitor(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport>;

    fn machine_top(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport>;

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint>;
}

#[derive(Debug, Default, Clone, Copy)]
struct FirecrackerLocalDriver;

#[derive(Debug, Default, Clone, Copy)]
struct HostedControlPlaneDriver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAction {
    Build,
    Validate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactRequest<'a> {
    pub name: &'a str,
    pub architecture: MachineArchitecture,
    pub substrate: ExecutionSubstrate,
    pub protection_mode: ProtectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMetadata {
    pub name: String,
    pub kind: ArtifactKind,
    pub reference: ArtifactReference,
    pub selector: ArtifactSelector,
    pub path: PathBuf,
    pub cache_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTransfer {
    pub artifact: ArtifactMetadata,
    pub store_path: PathBuf,
    pub bytes_copied: u64,
}

pub fn collect_doctor_report(config: Option<&PortConfig>) -> DoctorReport {
    collect_doctor_report_with_facts(config, &DoctorHostFacts::collect())
}

fn collect_doctor_report_with_facts(
    config: Option<&PortConfig>,
    facts: &DoctorHostFacts,
) -> DoctorReport {
    let host_os = facts.host_os.clone();
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
            if let Some(variant) = resolve_native_standard_variant(artifact) {
                checks.push(path_check(
                    format!("artifact:{name}"),
                    &variant.path,
                    true,
                    &format!("Artifact variant '{}' exists.", variant.path.display()),
                    &format!(
                        "Artifact variant '{}' is missing. Build or pull the native variant first.",
                        variant.path.display()
                    ),
                ));
            } else {
                checks.push(DoctorCheck {
                    name: format!("artifact:{name}"),
                    ok: false,
                    required: true,
                    detail: String::from(
                        "Artifact does not define a native Firecracker/standard variant for this host.",
                    ),
                });
            }
        }

        for (name, host) in &config.hosts {
            if let Some(check) = provider_check(name, host.provider, &host.connection) {
                checks.push(check);
            }
            checks.extend(local_pvm_lane_checks(name, host, facts));
        }
        checks.extend(hosted_pvm_lane_checks(config));
        for (name, control_plane) in &config.control_planes {
            checks.push(control_plane_check(name, control_plane));
        }

        for (name, machine) in &config.machines {
            let host = config
                .hosts
                .get(&machine.host)
                .expect("sampled machines should reference a known host");
            let kernel = config
                .artifact(&machine.kernel)
                .expect("sampled machines should reference a known kernel");
            let guest_image = config
                .artifact(&machine.guest_image)
                .expect("sampled machines should reference a known guest image");
            checks.push(machine_contract_check(
                name,
                host,
                machine,
                kernel,
                guest_image,
            ));
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
        notes.push(String::from(
            "Firecracker/PVM readiness is reported as a dedicated host-kit lane; failing PVM checks do not imply the standard Firecracker lane is a compatible fallback.",
        ));
    }

    DoctorReport {
        host_os,
        local_firecracker_supported,
        checks,
        notes,
    }
}

pub fn build_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactMetadata> {
    run_artifact_pipeline(config, request, ArtifactAction::Build)
}

pub fn validate_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactMetadata> {
    run_artifact_pipeline(config, request, ArtifactAction::Validate)
}

pub fn push_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactTransfer> {
    let artifact = resolve_artifact_metadata(config, request)?;
    let store_path = push_store_path(config, &artifact)?;
    let bytes_copied = copy_file(&artifact.path, &store_path)?;
    let _ = copy_file(&artifact.path, &artifact.cache_path)?;
    Ok(ArtifactTransfer {
        artifact,
        store_path,
        bytes_copied,
    })
}

pub fn pull_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactTransfer> {
    let artifact = resolve_artifact_metadata(config, request)?;
    let store_path = pull_store_path(config, &artifact)?;
    let bytes_copied = copy_file(&store_path, &artifact.cache_path)?;
    let _ = copy_file(&artifact.cache_path, &artifact.path)?;
    Ok(ArtifactTransfer {
        artifact,
        store_path,
        bytes_copied,
    })
}

pub fn launch_local_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    driver_for_machine(config, request.machine_name)?.launch(config, request)
}

fn firecracker_local_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    config
        .validate()
        .map_err(|error| anyhow!("invalid port config: {error}"))?;

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

    if !matches!(&host.connection, HostConnection::Local) {
        let hosted_identity = config
            .hosted_api_identity_contract(request.machine_name)
            .with_context(|| {
                format!(
                    "failed to resolve hosted API identity for machine '{}'",
                    request.machine_name
                )
            })?;
        bail!(
            "{}",
            remote_launch_guidance(
                request.machine_name,
                &machine.host,
                host.provider,
                hosted_identity.as_ref(),
            )
        );
    }

    let kernel = config
        .artifact(&machine.kernel)
        .with_context(|| format!("unknown kernel artifact '{}'", machine.kernel))?;
    let guest_image = config
        .artifact(&machine.guest_image)
        .with_context(|| format!("unknown guest image artifact '{}'", machine.guest_image))?;
    let machine_check =
        machine_contract_check(request.machine_name, host, machine, kernel, guest_image);
    if !machine_check.ok {
        bail!("machine contract failed: {}", machine_check.detail);
    }
    let resolved_architecture = resolve_machine_architecture(machine.architecture)?;
    let kernel_variant = kernel
        .variant(
            resolved_architecture,
            machine.substrate,
            machine.protection_mode,
        )
        .with_context(|| {
            format!(
                "kernel artifact '{}' is missing a variant for {:?}/{:?}/{:?}",
                machine.kernel, resolved_architecture, machine.substrate, machine.protection_mode
            )
        })?;
    let guest_variant = guest_image
        .variant(
            resolved_architecture,
            machine.substrate,
            machine.protection_mode,
        )
        .with_context(|| {
            format!(
                "guest image artifact '{}' is missing a variant for {:?}/{:?}/{:?}",
                machine.guest_image,
                resolved_architecture,
                machine.substrate,
                machine.protection_mode
            )
        })?;

    let facts = DoctorHostFacts::collect();

    if machine.protection_mode == ProtectionMode::Pvm {
        let lane_prefix = format!(
            "pvm:{}:{}:",
            machine.host,
            architecture_dir(resolved_architecture)
        );
        let pvm_failures = local_pvm_lane_checks(&machine.host, host, &facts)
            .into_iter()
            .filter(|check| check.name.starts_with(&lane_prefix) && !check.ok)
            .map(|check| format!("{}: {}", check.name, check.detail))
            .collect::<Vec<_>>();
        if !pvm_failures.is_empty() {
            bail!("pvm host-kit preflight failed: {}", pvm_failures.join("; "));
        }
    }

    let failures = launch_preflight_checks(machine, &kernel_variant.path, &guest_variant.path)
        .into_iter()
        .filter(|check| check.required && !check.ok)
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        let details = failures
            .into_iter()
            .map(|failure| format!("{}: {}", failure.name, failure.detail))
            .collect::<Vec<_>>()
            .join("; ");
        bail!("host preflight failed: {details}");
    }

    let pvm_host_kit = if machine.protection_mode == ProtectionMode::Pvm {
        host.firecracker
            .pvm_lane_for(resolved_architecture)
            .and_then(|lane| lane.host_kit.as_ref())
    } else {
        None
    };
    let pvm_firecracker_binary = pvm_host_kit.and_then(find_pvm_firecracker_binary_for_host_kit);
    let firecracker_binary = select_firecracker_binary(
        machine.protection_mode,
        find_binary("firecracker"),
        pvm_firecracker_binary.or_else(|| facts.pvm_firecracker_binary.clone()),
        pvm_host_kit,
    )?;

    let paths = RuntimePaths::for_machine(request.runtime_root, request.machine_name);
    fs::create_dir_all(&paths.runtime_dir).with_context(|| {
        format!(
            "failed to create runtime directory '{}'",
            paths.runtime_dir.display()
        )
    })?;
    prepare_runtime_state(&paths, request.machine_name)?;

    let config_payload = build_firecracker_config(
        kernel_variant.path.clone(),
        guest_variant.path.clone(),
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

pub fn list_machines(config: &PortConfig, runtime_root: &Path) -> Result<Vec<MachineStatus>> {
    let mut machines = BTreeMap::new();
    for machine in local_runtime_driver().list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }
    for machine in hosted_control_plane_driver().list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }

    Ok(machines.into_values().collect())
}

fn firecracker_local_list_machines(runtime_root: &Path) -> Result<Vec<MachineStatus>> {
    if !runtime_root.exists() {
        return Ok(Vec::new());
    }

    let mut machines = Vec::new();
    for entry in fs::read_dir(runtime_root)
        .with_context(|| format!("failed to read runtime root '{}'", runtime_root.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to read an entry from runtime root '{}'",
                runtime_root.display()
            )
        })?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
            .is_dir()
        {
            continue;
        }

        let machine_name = entry.file_name().to_string_lossy().into_owned();
        machines.push(inspect_machine(
            runtime_root,
            &machine_name,
            MachineControlContract::local_runtime_root(),
        )?);
    }
    machines.sort_by(|left, right| left.machine_name.cmp(&right.machine_name));

    Ok(machines)
}

pub fn machine_status(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineStatus> {
    if config.machines.contains_key(machine_name) {
        return driver_for_machine(config, machine_name)?.machine_status(
            config,
            runtime_root,
            machine_name,
        );
    }

    firecracker_local_machine_status(runtime_root, machine_name)
}

fn firecracker_local_machine_status(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineStatus> {
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);
    if !paths.runtime_dir.exists() {
        bail!(
            "runtime state for machine '{}' does not exist under '{}'",
            machine_name,
            runtime_root.display()
        );
    }

    inspect_machine(
        runtime_root,
        machine_name,
        MachineControlContract::local_runtime_root(),
    )
}

fn firecracker_local_machine_monitor(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    let status = firecracker_local_machine_status(runtime_root, machine_name)?;
    machine_monitor_report(status, None, None, Vec::new())
}

fn firecracker_local_machine_top(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineTopReport> {
    let status = firecracker_local_machine_status(runtime_root, machine_name)?;
    machine_top_report(status, None, None, Vec::new())
}

pub fn machine_monitor(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    if config.machines.contains_key(machine_name) {
        return driver_for_machine(config, machine_name)?.machine_monitor(
            config,
            runtime_root,
            machine_name,
        );
    }

    firecracker_local_machine_monitor(runtime_root, machine_name)
}

pub fn machine_top(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineTopReport> {
    if config.machines.contains_key(machine_name) {
        return driver_for_machine(config, machine_name)?.machine_top(
            config,
            runtime_root,
            machine_name,
        );
    }

    firecracker_local_machine_top(runtime_root, machine_name)
}

pub fn put_machine_secret(
    config: &PortConfig,
    request: SecretPutRequest<'_>,
) -> Result<MachineSecretSummary> {
    let context =
        resolve_service_runtime_context(config, request.runtime_root, request.machine_name)?;
    validate_identifier(request.name, "secret name")?;
    let dir = service_secret_dir(&context.status.runtime_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create secret directory '{}'", dir.display()))?;
    let record = MachineSecretRecord {
        name: request.name.to_string(),
        value: request.value.to_string(),
    };
    let path = dir.join(format!("{}.json", request.name));
    write_json_file(&path, &record)?;
    Ok(MachineSecretSummary {
        machine_name: request.machine_name.to_string(),
        name: request.name.to_string(),
        control: context.status.control,
        control_plane: context.control_plane,
        node_name: context.node_name,
        host_groups: context.host_groups,
        path,
        detail: String::from(
            "stored secret reference under the resolved machine runtime; future service execution will materialize it through the same runtime owner",
        ),
    })
}

pub fn list_machine_secrets(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<MachineSecretSummary>> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name)?;
    let dir = service_secret_dir(&context.status.runtime_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut secrets = Vec::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("failed to read secret directory '{}'", dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to inspect '{}'", dir.display()))?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let record: MachineSecretRecord = read_json_file(&entry.path())?;
        secrets.push(MachineSecretSummary {
            machine_name: machine_name.to_string(),
            name: record.name,
            control: context.status.control.clone(),
            control_plane: context.control_plane.clone(),
            node_name: context.node_name.clone(),
            host_groups: context.host_groups.clone(),
            path: entry.path(),
            detail: String::from("secret reference is available to service and sandbox bindings"),
        });
    }
    secrets.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(secrets)
}

pub fn delete_machine_secret(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    secret_name: &str,
) -> Result<MachineSecretSummary> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name)?;
    validate_identifier(secret_name, "secret name")?;
    let references = service_references_secret(&context.status.runtime_dir, secret_name)?;
    if !references.is_empty() {
        bail!(
            "cannot remove secret '{}' because it is referenced by service definitions: {}",
            secret_name,
            references.join(", ")
        );
    }
    let path = service_secret_dir(&context.status.runtime_dir).join(format!("{secret_name}.json"));
    let record: MachineSecretRecord = read_json_file(&path)?;
    fs::remove_file(&path)
        .with_context(|| format!("failed to remove secret '{}'", path.display()))?;
    Ok(MachineSecretSummary {
        machine_name: machine_name.to_string(),
        name: record.name,
        control: context.status.control,
        control_plane: context.control_plane,
        node_name: context.node_name,
        host_groups: context.host_groups,
        path,
        detail: String::from("removed secret reference from the resolved machine runtime"),
    })
}

pub fn apply_machine_service(
    config: &PortConfig,
    request: ServiceApplyRequest<'_>,
) -> Result<ServiceDefinitionStatus> {
    let context =
        resolve_service_runtime_context(config, request.runtime_root, request.machine_name)?;
    validate_identifier(request.name, "service name")?;
    if request.command.is_empty() {
        bail!("service apply requires a command");
    }
    validate_secret_bindings(&request.secret_bindings)?;
    for binding in &request.secret_bindings {
        let path = service_secret_dir(&context.status.runtime_dir)
            .join(format!("{}.json", binding.secret));
        if !path.exists() {
            bail!(
                "secret '{}' referenced by '{}' does not exist for machine '{}'",
                binding.secret,
                binding.env,
                request.machine_name
            );
        }
    }

    let dir = service_definition_dir(&context.status.runtime_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create service directory '{}'", dir.display()))?;
    let path = dir.join(format!("{}.json", request.name));
    let record = ServiceDefinitionRecord {
        machine_name: request.machine_name.to_string(),
        name: request.name.to_string(),
        kind: request.kind,
        desired_state: ServiceDesiredState::Active,
        command: request.command,
        secret_bindings: request.secret_bindings,
        control: context.status.control.clone(),
        control_plane: context.control_plane.clone(),
        node_name: context.node_name.clone(),
        host_groups: context.host_groups.clone(),
        created_at_unix_s: unix_timestamp_now()?,
        detail: String::from(
            "service definition is stored under the resolved runtime owner; guest execution remains a follow-on control-plane and node-agent slice",
        ),
    };
    write_json_file(&path, &record)?;
    Ok(service_status_from_record(record, path))
}

pub fn list_machine_services(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<ServiceDefinitionStatus>> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name)?;
    let dir = service_definition_dir(&context.status.runtime_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut services = Vec::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("failed to read service directory '{}'", dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to inspect '{}'", dir.display()))?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let record: ServiceDefinitionRecord = read_json_file(&entry.path())?;
        services.push(service_status_from_record(record, entry.path()));
    }
    services.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(services)
}

pub fn machine_service_status(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name)?;
    validate_identifier(service_name, "service name")?;
    let path =
        service_definition_dir(&context.status.runtime_dir).join(format!("{service_name}.json"));
    let record: ServiceDefinitionRecord = read_json_file(&path)?;
    Ok(service_status_from_record(record, path))
}

pub fn stop_machine_service(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name)?;
    validate_identifier(service_name, "service name")?;
    let path =
        service_definition_dir(&context.status.runtime_dir).join(format!("{service_name}.json"));
    let mut record: ServiceDefinitionRecord = read_json_file(&path)?;
    record.desired_state = ServiceDesiredState::Stopped;
    record.detail = String::from(
        "service definition is retained with desired state stopped; hosted execution and teardown remain a follow-on slice",
    );
    write_json_file(&path, &record)?;
    Ok(service_status_from_record(record, path))
}

pub fn stop_machine(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    if config.machines.contains_key(machine_name) {
        return driver_for_machine(config, machine_name)?.stop_machine(
            config,
            runtime_root,
            machine_name,
            timeout,
        );
    }

    firecracker_local_stop_machine(runtime_root, machine_name, timeout)
}

fn firecracker_local_stop_machine(
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let status = firecracker_local_machine_status(runtime_root, machine_name)?;
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);

    match status.state {
        MachineRuntimeState::Running => {
            let pid = status
                .pid
                .context("running machine status did not include a pid")?;
            signal_process(pid, libc::SIGTERM).with_context(|| {
                format!("failed to stop machine '{}' with SIGTERM", machine_name)
            })?;
            if !wait_for_process_exit(pid, machine_name, timeout)? {
                signal_process(pid, libc::SIGKILL).with_context(|| {
                    format!(
                        "failed to force-stop machine '{}' with SIGKILL",
                        machine_name
                    )
                })?;
                if !wait_for_process_exit(pid, machine_name, Duration::from_secs(1))? {
                    bail!(
                        "machine '{}' did not stop after SIGTERM/SIGKILL for pid {}",
                        machine_name,
                        pid
                    );
                }
            }
            cleanup_runtime_transient_paths(&paths)?;

            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Running,
                current_state: MachineRuntimeState::Stopped,
                pid: Some(pid),
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                detail: String::from("sent SIGTERM to pid and cleaned stale runtime sockets"),
            })
        }
        MachineRuntimeState::Stopped => {
            cleanup_runtime_transient_paths(&paths)?;
            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Stopped,
                current_state: MachineRuntimeState::Stopped,
                pid: status.pid,
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                detail: String::from("machine was already stopped"),
            })
        }
        MachineRuntimeState::Stale => {
            cleanup_runtime_transient_paths(&paths)?;
            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Stale,
                current_state: MachineRuntimeState::Stopped,
                pid: status.pid,
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                detail: String::from("cleaned stale runtime sockets for already-stopped machine"),
            })
        }
        MachineRuntimeState::Malformed => bail!(
            "runtime state for machine '{}' is malformed: {}",
            machine_name,
            status.detail
        ),
    }
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

fn inspect_machine(
    runtime_root: &Path,
    machine_name: &str,
    control: MachineControlContract,
) -> Result<MachineStatus> {
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);
    let pid_from_file = match read_pid_file(&paths.pid_path) {
        Ok(pid) => pid,
        Err(error) => {
            return Ok(malformed_machine_status(
                machine_name,
                &paths,
                control.clone(),
                error.to_string(),
            ));
        }
    };

    if !paths.manifest_path.exists() {
        return Ok(malformed_machine_status(
            machine_name,
            &paths,
            control.clone(),
            format!(
                "runtime manifest '{}' is missing",
                paths.manifest_path.display()
            ),
        ));
    }

    let manifest = match read_launch_metadata(&paths.manifest_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(malformed_machine_status(
                machine_name,
                &paths,
                control.clone(),
                format!(
                    "failed to parse manifest '{}': {error}",
                    paths.manifest_path.display()
                ),
            ));
        }
    };

    if manifest.machine_name != machine_name {
        return Ok(malformed_machine_status(
            machine_name,
            &paths,
            control.clone(),
            format!(
                "manifest machine name '{}' does not match runtime directory '{}'",
                manifest.machine_name, machine_name
            ),
        ));
    }

    let live_pid = resolve_live_machine_pid(machine_name, pid_from_file, Some(manifest.pid))?;
    let pid = live_pid.or(pid_from_file).or(Some(manifest.pid));
    let (state, detail) = match live_pid {
        Some(_) => (
            MachineRuntimeState::Running,
            String::from("live Firecracker process matches runtime manifest"),
        ),
        None if pid_from_file.is_some() => (
            MachineRuntimeState::Stale,
            String::from("recorded Firecracker pid is no longer live"),
        ),
        None => (
            MachineRuntimeState::Stopped,
            String::from("launch manifest exists but no live Firecracker process is recorded"),
        ),
    };

    Ok(MachineStatus {
        machine_name: machine_name.to_string(),
        state,
        pid,
        control,
        runtime_dir: paths.runtime_dir,
        config_path: paths.config_path,
        manifest_path: paths.manifest_path,
        pid_path: paths.pid_path,
        firecracker_log: paths.firecracker_log,
        stdout_log: paths.stdout_log,
        stderr_log: paths.stderr_log,
        detail,
    })
}

fn live_firecracker_pid(pid_path: &Path, machine_name: &str) -> Result<Option<u32>> {
    let Some(pid) = read_pid_file(pid_path)? else {
        return Ok(None);
    };

    if is_live_firecracker_pid(pid, machine_name)? {
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

fn process_exists(pid: u32) -> Result<bool> {
    // SAFETY: `kill(pid, 0)` is the standard existence probe for a process id.
    // The call does not mutate memory; it only asks the kernel whether the pid
    // is valid and signalable.
    let status = unsafe { libc::kill(pid as i32, 0) };
    if status == 0 {
        return Ok(true);
    }

    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(anyhow!("failed to probe pid {}: {}", pid, error)),
    }
}

fn read_launch_metadata(path: &Path) -> Result<LaunchMetadata> {
    let file = File::open(path)
        .with_context(|| format!("failed to open manifest '{}'", path.display()))?;
    serde_json::from_reader(file)
        .with_context(|| format!("failed to decode manifest '{}'", path.display()))
}

fn resolve_live_machine_pid(
    machine_name: &str,
    pid_from_file: Option<u32>,
    manifest_pid: Option<u32>,
) -> Result<Option<u32>> {
    if let Some(pid) = pid_from_file {
        if is_live_firecracker_pid(pid, machine_name)? {
            return Ok(Some(pid));
        }
    }

    if let Some(pid) = manifest_pid {
        if Some(pid) != pid_from_file && is_live_firecracker_pid(pid, machine_name)? {
            return Ok(Some(pid));
        }
    }

    Ok(None)
}

fn is_live_firecracker_pid(pid: u32, machine_name: &str) -> Result<bool> {
    let Some(cmdline) = process_cmdline(pid)? else {
        return Ok(false);
    };

    Ok(matches_firecracker_process(&cmdline, machine_name))
}

fn matches_firecracker_process(cmdline: &str, machine_name: &str) -> bool {
    let is_firecracker = cmdline.contains("firecracker");
    let matches_machine = cmdline.contains(&format!("--id {machine_name}"))
        || cmdline.contains(&format!("--id\0{machine_name}"));

    is_firecracker && matches_machine
}

fn wait_for_process_exit(pid: u32, machine_name: &str, timeout: Duration) -> Result<bool> {
    let step = Duration::from_millis(100);
    let mut waited = Duration::ZERO;

    while waited < timeout {
        if !is_live_firecracker_pid(pid, machine_name)? {
            return Ok(true);
        }
        thread::sleep(step);
        waited += step;
    }

    Ok(!is_live_firecracker_pid(pid, machine_name)?)
}

fn signal_process(pid: u32, signal: i32) -> Result<()> {
    // SAFETY: `libc::kill` is the POSIX process-signal interface. The call does not
    // alias Rust references, and we only pass the target pid plus a fixed signal.
    let status = unsafe { libc::kill(pid as i32, signal) };
    if status == 0 {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        bail!("signal delivery failed for pid {}: {}", pid, error);
    }
}

fn cleanup_runtime_transient_paths(paths: &RuntimePaths) -> Result<()> {
    remove_stale_runtime_path(&paths.pid_path, "pid file")?;
    remove_stale_runtime_path(&paths.vsock_path, "vsock socket")?;
    remove_stale_runtime_path(&paths.guest_agent_socket, "guest-agent socket")?;
    Ok(())
}

fn malformed_machine_status(
    machine_name: &str,
    paths: &RuntimePaths,
    control: MachineControlContract,
    detail: String,
) -> MachineStatus {
    synthetic_machine_status(
        machine_name,
        paths,
        control,
        MachineRuntimeState::Malformed,
        detail,
    )
}

fn synthetic_machine_status(
    machine_name: &str,
    paths: &RuntimePaths,
    control: MachineControlContract,
    state: MachineRuntimeState,
    detail: String,
) -> MachineStatus {
    MachineStatus {
        machine_name: machine_name.to_string(),
        state,
        pid: None,
        control,
        runtime_dir: paths.runtime_dir.clone(),
        config_path: paths.config_path.clone(),
        manifest_path: paths.manifest_path.clone(),
        pid_path: paths.pid_path.clone(),
        firecracker_log: paths.firecracker_log.clone(),
        stdout_log: paths.stdout_log.clone(),
        stderr_log: paths.stderr_log.clone(),
        detail,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DetachedForwardManifestRecord {
    name: String,
    machine: String,
    pid: u32,
    listen: String,
    target: String,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
}

fn machine_monitor_report(
    status: MachineStatus,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
) -> Result<MachineMonitorReport> {
    let detached_forwards =
        load_detached_forward_statuses(&status.runtime_dir, &status.machine_name)?;

    Ok(MachineMonitorReport {
        machine_name: status.machine_name,
        state: status.state,
        pid: status.pid,
        control: status.control,
        control_plane,
        node_name,
        host_groups,
        runtime_dir: status.runtime_dir,
        config_path: status.config_path,
        manifest_path: status.manifest_path,
        pid_path: status.pid_path,
        firecracker_log: status.firecracker_log,
        stdout_log: status.stdout_log,
        stderr_log: status.stderr_log,
        detached_forwards,
        detail: status.detail,
    })
}

fn machine_top_report(
    status: MachineStatus,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
) -> Result<MachineTopReport> {
    let firecracker_command = match status.pid {
        Some(pid) => process_cmdline(pid)?,
        None => None,
    };
    let mut entries = Vec::new();
    if status.pid.is_some() || status.manifest_path.exists() {
        entries.push(MachineTopEntry {
            kind: MachineTopEntryKind::Hypervisor,
            name: String::from("firecracker"),
            state: status.state,
            pid: status.pid,
            command: firecracker_command,
            source: status.manifest_path.clone(),
            detail: status.detail.clone(),
        });
    }

    for forward in load_detached_forward_statuses(&status.runtime_dir, &status.machine_name)? {
        let command = match forward.pid {
            Some(pid) => process_cmdline(pid)?,
            None => None,
        };
        entries.push(MachineTopEntry {
            kind: MachineTopEntryKind::DetachedForward,
            name: forward.name,
            state: forward.state,
            pid: forward.pid,
            command,
            source: forward.manifest_path,
            detail: format!(
                "{} listen={} target={}",
                forward.detail, forward.listen, forward.target
            ),
        });
    }
    entries.sort_by(|left, right| {
        machine_top_entry_rank(left.kind)
            .cmp(&machine_top_entry_rank(right.kind))
            .then(left.name.cmp(&right.name))
    });

    Ok(MachineTopReport {
        machine_name: status.machine_name,
        state: status.state,
        pid: status.pid,
        control: status.control,
        control_plane,
        node_name,
        host_groups,
        runtime_dir: status.runtime_dir,
        detail: status.detail,
        entries,
    })
}

fn load_detached_forward_statuses(
    runtime_dir: &Path,
    machine_name: &str,
) -> Result<Vec<DetachedForwardStatus>> {
    let forwards_dir = runtime_dir.join("forwards");
    if !forwards_dir.exists() {
        return Ok(Vec::new());
    }

    let mut forwards = Vec::new();
    for entry in fs::read_dir(&forwards_dir)
        .with_context(|| format!("failed to read forward state '{}'", forwards_dir.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to inspect '{}'", forwards_dir.display()))?;
        let path = entry.path();
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", path.display()))?
            .is_file()
        {
            continue;
        }

        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read forward manifest '{}'", path.display()))?;
        let manifest: DetachedForwardManifestRecord = match serde_json::from_slice(&bytes) {
            Ok(manifest) => manifest,
            Err(error) => {
                forwards.push(DetachedForwardStatus {
                    name: path
                        .file_stem()
                        .map(|value| value.to_string_lossy().into_owned())
                        .unwrap_or_else(|| String::from("(unknown)")),
                    state: MachineRuntimeState::Malformed,
                    pid: None,
                    listen: String::new(),
                    target: String::new(),
                    manifest_path: path.clone(),
                    stdout_log: PathBuf::new(),
                    stderr_log: PathBuf::new(),
                    detail: format!("failed to parse detached forward manifest: {error}"),
                });
                continue;
            }
        };

        if manifest.machine != machine_name {
            forwards.push(DetachedForwardStatus {
                name: manifest.name,
                state: MachineRuntimeState::Malformed,
                pid: Some(manifest.pid),
                listen: manifest.listen,
                target: manifest.target,
                manifest_path: path.clone(),
                stdout_log: manifest.stdout_log,
                stderr_log: manifest.stderr_log,
                detail: format!(
                    "detached forward manifest targets machine '{}' instead of '{}'",
                    manifest.machine, machine_name
                ),
            });
            continue;
        }

        let state = if process_exists(manifest.pid)? {
            MachineRuntimeState::Running
        } else {
            MachineRuntimeState::Stale
        };
        let detail = match state {
            MachineRuntimeState::Running => String::from("detached forward process is live"),
            MachineRuntimeState::Stale => {
                String::from("recorded detached forward pid is no longer live")
            }
            _ => unreachable!("detached forward state should be running or stale"),
        };
        forwards.push(DetachedForwardStatus {
            name: manifest.name,
            state,
            pid: Some(manifest.pid),
            listen: manifest.listen,
            target: manifest.target,
            manifest_path: path,
            stdout_log: manifest.stdout_log,
            stderr_log: manifest.stderr_log,
            detail,
        });
    }
    forwards.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(forwards)
}

fn machine_top_entry_rank(kind: MachineTopEntryKind) -> u8 {
    match kind {
        MachineTopEntryKind::Hypervisor => 0,
        MachineTopEntryKind::DetachedForward => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MachineSecretRecord {
    name: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceDefinitionRecord {
    machine_name: String,
    name: String,
    kind: ServiceKind,
    desired_state: ServiceDesiredState,
    command: Vec<String>,
    secret_bindings: Vec<ServiceSecretBinding>,
    control: MachineControlContract,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
    created_at_unix_s: u64,
    detail: String,
}

#[derive(Debug, Clone)]
struct ResolvedMachineRuntime {
    status: MachineStatus,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
}

fn service_state_dir(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("services")
}

fn service_secret_dir(runtime_dir: &Path) -> PathBuf {
    service_state_dir(runtime_dir).join("secrets")
}

fn service_definition_dir(runtime_dir: &Path) -> PathBuf {
    service_state_dir(runtime_dir).join("definitions")
}

fn resolve_machine_runtime(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<ResolvedMachineRuntime> {
    if config.machines.contains_key(machine_name) {
        let machine = config
            .machines
            .get(machine_name)
            .with_context(|| format!("unknown machine '{}'", machine_name))?;
        let host = config
            .hosts
            .get(&machine.host)
            .with_context(|| format!("unknown host '{}'", machine.host))?;
        return match &host.connection {
            HostConnection::Local => Ok(ResolvedMachineRuntime {
                status: firecracker_local_machine_status(runtime_root, machine_name)?,
                control_plane: None,
                node_name: None,
                host_groups: Vec::new(),
            }),
            HostConnection::HostedControlPlane { .. } => {
                let resolution = hosted_machine_resolution(config, machine_name)?;
                Ok(ResolvedMachineRuntime {
                    status: resolution.status,
                    control_plane: Some(resolution.control_plane),
                    node_name: resolution.node_name,
                    host_groups: resolution.host_groups,
                })
            }
        };
    }

    Ok(ResolvedMachineRuntime {
        status: firecracker_local_machine_status(runtime_root, machine_name)?,
        control_plane: None,
        node_name: None,
        host_groups: Vec::new(),
    })
}

fn resolve_service_runtime_context(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<ResolvedMachineRuntime> {
    let context = resolve_machine_runtime(config, runtime_root, machine_name)?;
    if context.status.state == MachineRuntimeState::Malformed {
        bail!(
            "service operations require well-formed runtime state for machine '{}': {}",
            machine_name,
            context.status.detail
        );
    }
    if !context.status.runtime_dir.exists() {
        bail!(
            "service operations require an existing Port runtime for machine '{}': {}",
            machine_name,
            context.status.detail
        );
    }
    Ok(context)
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{label} must not be empty");
    }
    if trimmed.contains('/') || trimmed.contains("..") {
        bail!("{label} must not contain path traversal or '/' segments");
    }
    Ok(())
}

fn validate_secret_bindings(bindings: &[ServiceSecretBinding]) -> Result<()> {
    let mut seen = BTreeMap::new();
    for binding in bindings {
        validate_identifier(&binding.env, "secret environment name")?;
        validate_identifier(&binding.secret, "secret binding name")?;
        if seen
            .insert(binding.env.clone(), binding.secret.clone())
            .is_some()
        {
            bail!(
                "secret environment name '{}' is bound more than once",
                binding.env
            );
        }
    }
    Ok(())
}

fn service_references_secret(runtime_dir: &Path, secret_name: &str) -> Result<Vec<String>> {
    let definitions = service_definition_dir(runtime_dir);
    if !definitions.exists() {
        return Ok(Vec::new());
    }

    let mut references = Vec::new();
    for entry in fs::read_dir(&definitions).with_context(|| {
        format!(
            "failed to read service directory '{}'",
            definitions.display()
        )
    })? {
        let entry =
            entry.with_context(|| format!("failed to inspect '{}'", definitions.display()))?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let record: ServiceDefinitionRecord = read_json_file(&entry.path())?;
        if record
            .secret_bindings
            .iter()
            .any(|binding| binding.secret == secret_name)
        {
            references.push(record.name);
        }
    }
    references.sort();
    Ok(references)
}

fn service_status_from_record(
    record: ServiceDefinitionRecord,
    manifest_path: PathBuf,
) -> ServiceDefinitionStatus {
    ServiceDefinitionStatus {
        machine_name: record.machine_name,
        name: record.name,
        kind: record.kind,
        desired_state: record.desired_state,
        command: record.command,
        secret_bindings: record.secret_bindings,
        control: record.control,
        control_plane: record.control_plane,
        node_name: record.node_name,
        host_groups: record.host_groups,
        manifest_path,
        detail: record.detail,
    }
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .with_context(|| format!("failed to encode '{}'", path.display()))?;
    fs::write(path, format!("{}\n", String::from_utf8_lossy(&bytes)))
        .with_context(|| format!("failed to write '{}'", path.display()))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("failed to decode '{}'", path.display()))
}

fn unix_timestamp_now() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .context("system clock is before the Unix epoch")
}

#[derive(Debug, Clone)]
struct HostedMachineResolution {
    control_plane: String,
    node_name: Option<String>,
    host_groups: Vec<String>,
    runtime_root: PathBuf,
    status: MachineStatus,
}

fn hosted_placeholder_runtime_root(control_plane: &str) -> PathBuf {
    PathBuf::from(".port/hosted").join(control_plane)
}

fn status_priority(state: MachineRuntimeState) -> u8 {
    match state {
        MachineRuntimeState::Running => 4,
        MachineRuntimeState::Malformed => 3,
        MachineRuntimeState::Stale => 2,
        MachineRuntimeState::Stopped => 1,
    }
}

fn hosted_machine_resolution(
    config: &PortConfig,
    machine_name: &str,
) -> Result<HostedMachineResolution> {
    let control = config.machine_control_contract(machine_name)?;
    let hosted_identity = config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let placeholder_root = hosted_placeholder_runtime_root(&hosted_identity.control_plane);

    let summary = match config.hosted_machine_summary_contract(machine_name) {
        Ok(Some(summary)) => summary,
        Ok(None) => {
            return Ok(HostedMachineResolution {
                control_plane: hosted_identity.control_plane.clone(),
                node_name: None,
                host_groups: Vec::new(),
                runtime_root: placeholder_root.clone(),
                status: synthetic_machine_status(
                    machine_name,
                    &RuntimePaths::for_machine(&placeholder_root, machine_name),
                    control,
                    MachineRuntimeState::Malformed,
                    format!(
                        "control plane '{}' did not resolve a hosted machine contract for '{}'",
                        hosted_identity.control_plane, machine_name
                    ),
                ),
            });
        }
        Err(error) => {
            return Ok(HostedMachineResolution {
                control_plane: hosted_identity.control_plane.clone(),
                node_name: None,
                host_groups: Vec::new(),
                runtime_root: placeholder_root.clone(),
                status: synthetic_machine_status(
                    machine_name,
                    &RuntimePaths::for_machine(&placeholder_root, machine_name),
                    control,
                    MachineRuntimeState::Malformed,
                    format!(
                        "control plane '{}' cannot resolve hosted runtime for machine '{}': {}",
                        hosted_identity.control_plane, machine_name, error
                    ),
                ),
            });
        }
    };

    if summary.candidate_nodes.is_empty() {
        return Ok(HostedMachineResolution {
            control_plane: summary.control_plane.clone(),
            node_name: None,
            host_groups: summary.host_groups.clone(),
            runtime_root: placeholder_root.clone(),
            status: synthetic_machine_status(
                machine_name,
                &RuntimePaths::for_machine(&placeholder_root, machine_name),
                control,
                MachineRuntimeState::Malformed,
                format!(
                    "control plane '{}' cannot place machine '{}': {}",
                    summary.control_plane, machine_name, summary.placement_detail
                ),
            ),
        });
    }

    let inventory = config.hosted_inventory_contract()?;
    let mut selected = None::<HostedMachineResolution>;
    for node_name in &summary.candidate_nodes {
        let Some(node) = inventory.nodes.get(node_name) else {
            continue;
        };

        let paths = RuntimePaths::for_machine(&node.runtime_root, machine_name);
        let mut status = if paths.runtime_dir.exists() {
            inspect_machine(&node.runtime_root, machine_name, control.clone())?
        } else {
            synthetic_machine_status(
                machine_name,
                &paths,
                control.clone(),
                MachineRuntimeState::Stopped,
                format!(
                    "control plane '{}' resolved node '{}' but the node-agent runtime root '{}' does not contain machine state",
                    summary.control_plane,
                    node_name,
                    node.runtime_root.display()
                ),
            )
        };
        status.detail = format!(
            "{} Routed through control plane '{}' and node '{}'.",
            status.detail, summary.control_plane, node_name
        );

        let candidate = HostedMachineResolution {
            control_plane: summary.control_plane.clone(),
            node_name: Some(node_name.clone()),
            host_groups: summary.host_groups.clone(),
            runtime_root: node.runtime_root.clone(),
            status,
        };

        if selected.as_ref().is_none_or(|current| {
            status_priority(candidate.status.state) > status_priority(current.status.state)
        }) {
            selected = Some(candidate);
        }
    }

    Ok(selected.unwrap_or(HostedMachineResolution {
        control_plane: summary.control_plane.clone(),
        node_name: None,
        host_groups: summary.host_groups.clone(),
        runtime_root: placeholder_root.clone(),
        status: synthetic_machine_status(
            machine_name,
            &RuntimePaths::for_machine(&placeholder_root, machine_name),
            control,
            MachineRuntimeState::Malformed,
            format!(
                "control plane '{}' resolved machine '{}' but no candidate node runtime bindings were available. {}",
                summary.control_plane, machine_name, summary.placement_detail
            ),
        ),
    }))
}

fn machine_is_hosted(config: &PortConfig, machine_name: &str) -> Result<bool> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{machine_name}'"))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    Ok(matches!(
        &host.connection,
        HostConnection::HostedControlPlane { .. }
    ))
}

fn hosted_client_for_machine(config: &PortConfig, machine_name: &str) -> Result<HostedClient> {
    HostedClient::from_machine_env(config, machine_name).with_context(|| {
        format!(
            "failed to resolve live hosted client transport for machine '{}'",
            machine_name
        )
    })
}

fn hosted_client_for_control_plane(
    config: &PortConfig,
    control_plane_name: &str,
) -> Result<HostedClient> {
    HostedClient::from_control_plane_env(config, control_plane_name).with_context(|| {
        format!(
            "failed to resolve live hosted client transport for control plane '{}'",
            control_plane_name
        )
    })
}

fn hosted_control_plane_names(config: &PortConfig) -> Vec<String> {
    let mut names = config
        .machines
        .values()
        .filter_map(|machine| {
            config
                .hosts
                .get(&machine.host)
                .and_then(|host| match &host.connection {
                    HostConnection::HostedControlPlane { control_plane } => {
                        Some(control_plane.clone())
                    }
                    HostConnection::Local => None,
                })
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn hosted_control_plane_list_machines(config: &PortConfig) -> Result<Vec<MachineStatus>> {
    let mut machines = Vec::new();
    for control_plane_name in hosted_control_plane_names(config) {
        let client = hosted_client_for_control_plane(config, &control_plane_name)?;
        let response: HostedSuccess<Vec<MachineStatus>> = client
            .execute_json(client.machines().list())
            .map_err(|error| {
                anyhow!(
                    "failed to list machines through hosted control plane '{}': {error}",
                    control_plane_name
                )
            })?;
        machines.extend(response.result);
    }
    machines.sort_by(|left, right| left.machine_name.cmp(&right.machine_name));
    Ok(machines)
}

fn hosted_control_plane_launch_machine(
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
    let hosted_identity = config
        .hosted_api_identity_contract(request.machine_name)?
        .ok_or_else(|| {
            anyhow!(
                "machine '{}' does not target a hosted control plane",
                request.machine_name
            )
        })?;
    if machine.protection_mode != ProtectionMode::Pvm {
        bail!(
            "{}",
            remote_launch_guidance(
                request.machine_name,
                &machine.host,
                host.provider,
                Some(&hosted_identity),
            )
        );
    }

    if let Some(summary) = config.hosted_machine_summary_contract(request.machine_name)? {
        if summary.candidate_nodes.is_empty() {
            bail!(
                "hosted machine '{}' is not placeable through control plane '{}': {}",
                request.machine_name,
                summary.control_plane,
                summary.placement_detail
            );
        }
    }

    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<LaunchMetadata> = client
        .execute_json(client.machines().launch(request.machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to launch machine '{}' through the live hosted control-plane route: {error}",
                request.machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_machine_status(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineStatus> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<MachineStatus> = client
        .execute_json(client.machines().status(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to load machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_machine_monitor(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<MachineMonitorReport> = client
        .execute_json(client.machines().monitor(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to monitor machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_machine_top(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineTopReport> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<MachineTopReport> = client
        .execute_json(client.machines().top(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to inspect top data for machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_stop_machine(
    config: &PortConfig,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let _ = timeout;
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<StopResult> = client
        .execute_json(client.machines().stop(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to stop machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_guest_endpoint(
    config: &PortConfig,
    request: &GuestRequest<'_>,
) -> Result<GuestEndpoint> {
    let resolution = hosted_machine_resolution(config, request.machine_name)?;
    let attach_detail = match config.hosted_guest_attach_contract(request.machine_name) {
        Ok(Some(attach)) => attach.detail,
        Ok(None) => {
            bail!(
                "machine '{}' does not target a hosted control plane guest route",
                request.machine_name
            );
        }
        Err(_) => String::from(
            "Hosted guest attach preserves the canonical guest protocol through control-plane authorization and node-agent brokerage.",
        ),
    };
    let Some(node_name) = resolution.node_name else {
        bail!(
            "control plane '{}' could not authorize guest attach for machine '{}': {}",
            resolution.control_plane,
            request.machine_name,
            resolution.status.detail
        );
    };

    let routed_request = GuestRequest {
        machine_name: request.machine_name,
        runtime_root: &resolution.runtime_root,
        operation: request.operation.clone(),
    };

    resolve_firecracker_guest_endpoint(config, &routed_request).with_context(|| {
        format!(
            "control plane '{}' authorized guest attach for machine '{}' and routed it to node '{}'. {}",
            resolution.control_plane, request.machine_name, node_name, attach_detail
        )
    })
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
        detail: match connection {
            HostConnection::Local => detail,
            HostConnection::HostedControlPlane { control_plane } => {
                format!(
                    "{detail} Hosted routing is modeled through control plane '{control_plane}'."
                )
            }
        },
    })
}

fn control_plane_check(
    control_plane_name: &str,
    control_plane: &port_model::HostedControlPlaneSpec,
) -> DoctorCheck {
    DoctorCheck {
        name: format!("control-plane:{control_plane_name}"),
        ok: true,
        required: false,
        detail: format!(
            "Hosted control plane '{}' targets '{}' with audience '{}' and expects a {} token from {} via the '{}' header.",
            control_plane_name,
            control_plane.endpoint,
            control_plane.audience,
            match control_plane.auth.scheme {
                port_model::HostedAuthScheme::Bearer => "bearer",
            },
            control_plane.auth.source.describe(),
            control_plane.auth.header,
        ),
    }
}

fn machine_contract_check(
    machine_name: &str,
    host: &port_model::HostSpec,
    machine: &port_model::MachineSpec,
    kernel: &port_model::ArtifactSpec,
    guest_image: &port_model::ArtifactSpec,
) -> DoctorCheck {
    let resolved_architecture = match resolve_machine_architecture(machine.architecture) {
        Ok(architecture) => architecture,
        Err(error) => {
            return DoctorCheck {
                name: format!("machine:{machine_name}"),
                ok: false,
                required: false,
                detail: error.to_string(),
            };
        }
    };

    let mut issues = Vec::new();
    match machine.substrate {
        ExecutionSubstrate::Firecracker => {
            if host.platform != HostPlatform::Linux {
                issues.push(String::from(
                    "Firecracker execution requires a Linux host platform.",
                ));
            }
            if machine.protection_mode == ProtectionMode::Pvm
                && resolved_architecture == MachineArchitecture::Aarch64
            {
                issues.push(String::from(
                    "Firecracker/PVM on arm64 remains a research lane; Port does not yet claim a supportable runtime path.",
                ));
            }
        }
        ExecutionSubstrate::CloudHypervisor => {
            if host.platform != HostPlatform::Linux {
                issues.push(String::from(
                    "Cloud Hypervisor execution currently expects a Linux host platform.",
                ));
            }
            if machine.protection_mode == ProtectionMode::Pvm {
                issues.push(String::from(
                    "Port does not currently define a Cloud Hypervisor PVM lane.",
                ));
            }
        }
        ExecutionSubstrate::Avf => {
            if host.platform != HostPlatform::Macos {
                issues.push(String::from(
                    "Apple Virtualization Framework requires a macOS host platform.",
                ));
            }
            if machine.protection_mode == ProtectionMode::Pvm {
                issues.push(String::from(
                    "Apple Virtualization Framework does not currently define a PVM lane.",
                ));
            }
        }
    }

    if !kernel.supports(
        resolved_architecture,
        machine.substrate,
        machine.protection_mode,
    ) {
        issues.push(format!(
            "Kernel artifact '{}' is not compatible with {:?}/{:?}/{:?}.",
            machine.kernel, machine.substrate, machine.protection_mode, resolved_architecture
        ));
    }
    if !guest_image.supports(
        resolved_architecture,
        machine.substrate,
        machine.protection_mode,
    ) {
        issues.push(format!(
            "Guest image artifact '{}' is not compatible with {:?}/{:?}/{:?}.",
            machine.guest_image, machine.substrate, machine.protection_mode, resolved_architecture
        ));
    }

    if issues.is_empty() {
        DoctorCheck {
            name: format!("machine:{machine_name}"),
            ok: true,
            required: false,
            detail: format!(
                "Machine models {:?}/{:?}/{:?} with compatible artifacts.",
                machine.substrate, machine.protection_mode, resolved_architecture
            ),
        }
    } else {
        DoctorCheck {
            name: format!("machine:{machine_name}"),
            ok: false,
            required: false,
            detail: issues.join(" "),
        }
    }
}

fn local_pvm_lane_checks(
    host_name: &str,
    host: &port_model::HostSpec,
    facts: &DoctorHostFacts,
) -> Vec<DoctorCheck> {
    if !matches!(host.connection, HostConnection::Local) {
        return Vec::new();
    }

    let mut checks = Vec::new();
    for lane in &host.firecracker.pvm_lanes {
        let architecture = architecture_dir(lane.architecture);
        match lane.decision {
            port_model::PvmLaneDecision::Planned => {
                let Some(host_kit) = lane.host_kit.as_ref() else {
                    checks.push(DoctorCheck {
                        name: format!("pvm:{host_name}:{architecture}:host-kit-contract"),
                        ok: false,
                        required: false,
                        detail: String::from(
                            "PVM lane is marked planned but does not define a host-kit contract.",
                        ),
                    });
                    continue;
                };
                if let Some(detail) = pvm_host_kit_contract_issue(lane.architecture, host_kit) {
                    checks.push(DoctorCheck {
                        name: format!("pvm:{host_name}:{architecture}:host-kit-contract"),
                        ok: false,
                        required: false,
                        detail,
                    });
                    continue;
                }
                checks.push(DoctorCheck {
                    name: format!("pvm:{host_name}:{architecture}:host-kit-contract"),
                    ok: true,
                    required: false,
                    detail: pvm_host_kit_contract_detail(host_kit),
                });

                let platform_ok =
                    host.platform == host_kit.host_platform && facts.host_os == "linux";
                checks.push(DoctorCheck {
                    name: format!("pvm:{host_name}:{architecture}:host-platform"),
                    ok: platform_ok,
                    required: false,
                    detail: if platform_ok {
                        format!(
                            "Host '{}' matches the Linux platform required for the Firecracker/PVM host kit.",
                            host_name
                        )
                    } else {
                        format!(
                            "Host '{}' must run on Linux for Firecracker/PVM; the standard Firecracker lane is not a PVM fallback.",
                            host_name
                        )
                    },
                });

                let expected_architecture = architecture_dir(host_kit.host_architecture);
                let architecture_ok = facts.host_architecture == expected_architecture;
                checks.push(DoctorCheck {
                    name: format!("pvm:{host_name}:{architecture}:host-architecture"),
                    ok: architecture_ok,
                    required: false,
                    detail: if architecture_ok {
                        format!(
                            "Host architecture '{}' matches the PVM host-kit requirement.",
                            expected_architecture
                        )
                    } else {
                        format!(
                            "Host architecture '{}' does not satisfy the Firecracker/PVM host-kit requirement '{}'; the standard Firecracker lane is not a PVM fallback.",
                            facts.host_architecture, expected_architecture
                        )
                    },
                });

                let missing_boot_args: Vec<&String> = host_kit
                    .host_boot_args
                    .iter()
                    .filter(|arg| {
                        facts
                            .proc_cmdline
                            .as_deref()
                            .map(|cmdline| {
                                !cmdline.split_whitespace().any(|item| item == arg.as_str())
                            })
                            .unwrap_or(true)
                    })
                    .collect();
                let boot_line_ok = missing_boot_args.is_empty();
                checks.push(DoctorCheck {
                    name: format!("pvm:{host_name}:{architecture}:boot-line"),
                    ok: boot_line_ok,
                    required: false,
                    detail: if boot_line_ok {
                        format!(
                            "Host boot line includes the required PVM argument(s): {}.",
                            host_kit.host_boot_args.join(", ")
                        )
                    } else {
                        format!(
                            "Host boot line is missing required PVM argument(s): {}. Reboot into the prepared host kit; the standard Firecracker lane is not a PVM fallback.",
                            missing_boot_args
                                .into_iter()
                                .map(std::string::String::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    },
                });

                let observed_binary = observed_pvm_firecracker_binary(host_kit, facts);
                let binary_ok = observed_binary.is_some();
                checks.push(DoctorCheck {
                    name: format!("pvm:{host_name}:{architecture}:firecracker-binary"),
                    ok: binary_ok,
                    required: false,
                    detail: match &observed_binary {
                        Some(path) => format!(
                            "Found the patched PVM Firecracker binary at '{}'.",
                            path.display()
                        ),
                        None => pvm_firecracker_missing_detail(host_kit),
                    },
                });
            }
            port_model::PvmLaneDecision::ResearchOnly => checks.push(DoctorCheck {
                name: format!("pvm:{host_name}:{architecture}"),
                ok: false,
                required: false,
                detail: format!(
                    "Firecracker/PVM on '{}' remains research-only. {}",
                    architecture,
                    lane.operator_prerequisites
                        .first()
                        .cloned()
                        .unwrap_or_else(|| String::from(
                            "Port does not yet claim a supportable runtime path for this lane."
                        ))
                ),
            }),
        }
    }

    checks
}

fn hosted_pvm_lane_checks(config: &PortConfig) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    for (node_name, node) in &config.nodes {
        for lane in &node.capabilities.pvm_lanes {
            if lane.state == port_model::PvmCapabilityState::ResearchOnly {
                continue;
            }

            let name = format!(
                "pvm:{node_name}:{}:host-kit-contract",
                architecture_dir(lane.architecture)
            );
            match lane.host_kit.as_ref() {
                Some(host_kit) => {
                    if let Some(detail) = pvm_host_kit_contract_issue(lane.architecture, host_kit) {
                        checks.push(DoctorCheck {
                            name,
                            ok: false,
                            required: false,
                            detail,
                        });
                    } else {
                        checks.push(DoctorCheck {
                            name,
                            ok: true,
                            required: false,
                            detail: format!(
                                "Hosted node '{}' advertises {}",
                                node_name,
                                pvm_host_kit_contract_detail(host_kit)
                            ),
                        });
                    }
                }
                None => checks.push(DoctorCheck {
                    name,
                    ok: false,
                    required: false,
                    detail: format!(
                        "Hosted node '{}' advertises a {:?} PVM lane without a host-kit contract.",
                        node_name, lane.state
                    ),
                }),
            }
        }
    }

    checks
}

fn pvm_host_kit_contract_issue(
    expected_architecture: MachineArchitecture,
    host_kit: &PvmHostKit,
) -> Option<String> {
    if host_kit.host_platform != HostPlatform::Linux {
        return Some(String::from(
            "host-kit contract must target host platform 'linux' for Firecracker/PVM.",
        ));
    }
    if host_kit.host_architecture != expected_architecture {
        return Some(format!(
            "host-kit contract must target host architecture '{}', not '{}'.",
            architecture_dir(expected_architecture),
            architecture_dir(host_kit.host_architecture)
        ));
    }
    if host_kit.firecracker_binary_name.trim().is_empty() {
        return Some(String::from(
            "host-kit contract must declare a non-empty firecracker binary name.",
        ));
    }
    if host_kit
        .firecracker_binary_env
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Some(String::from(
            "host-kit contract must declare a non-empty firecracker binary environment variable when firecracker_binary_env is set.",
        ));
    }
    if host_kit
        .host_boot_args
        .iter()
        .any(|argument| argument.trim().is_empty())
    {
        return Some(String::from(
            "host-kit contract must not contain empty host boot arguments.",
        ));
    }
    if host_kit.requires_custom_host_kernel && host_kit.host_boot_args.is_empty() {
        return Some(String::from(
            "host-kit contract must declare at least one host boot argument for the custom host kernel.",
        ));
    }

    None
}

fn pvm_host_kit_contract_detail(host_kit: &PvmHostKit) -> String {
    format!(
        "host-kit contract requires Linux/{}, boot args [{}], and the patched Firecracker binary {}.",
        architecture_dir(host_kit.host_architecture),
        host_kit.host_boot_args.join(", "),
        pvm_firecracker_lookup_detail(host_kit)
    )
}

fn pvm_firecracker_lookup_detail(host_kit: &PvmHostKit) -> String {
    match host_kit.firecracker_binary_env.as_deref() {
        Some(variable) => format!(
            "'{}' via ${variable} or PATH",
            host_kit.firecracker_binary_name
        ),
        None => format!("'{}' on PATH", host_kit.firecracker_binary_name),
    }
}

fn pvm_firecracker_missing_detail(host_kit: &PvmHostKit) -> String {
    match host_kit.firecracker_binary_env.as_deref() {
        Some(variable) => format!(
            "Missing the patched PVM Firecracker binary. Set {variable} or put '{}' on PATH; the standard firecracker binary is not compatible.",
            host_kit.firecracker_binary_name
        ),
        None => format!(
            "Missing the patched PVM Firecracker binary '{}'; the standard firecracker binary is not compatible.",
            host_kit.firecracker_binary_name
        ),
    }
}

fn observed_pvm_firecracker_binary(
    host_kit: &PvmHostKit,
    facts: &DoctorHostFacts,
) -> Option<PathBuf> {
    if host_kit.firecracker_binary_name == "firecracker-pvm"
        && host_kit.firecracker_binary_env.as_deref() == Some("PORT_PVM_FIRECRACKER_BINARY")
    {
        facts.pvm_firecracker_binary.clone()
    } else {
        find_pvm_firecracker_binary_for_host_kit(host_kit)
    }
}

fn resolve_machine_architecture(architecture: MachineArchitecture) -> Result<MachineArchitecture> {
    match architecture {
        MachineArchitecture::Native => match env::consts::ARCH {
            "x86_64" => Ok(MachineArchitecture::X86_64),
            "aarch64" => Ok(MachineArchitecture::Aarch64),
            other => bail!("host architecture '{other}' is not yet modeled by Port"),
        },
        concrete => Ok(concrete),
    }
}

fn remote_launch_guidance(
    machine_name: &str,
    host_name: &str,
    provider: HostProvider,
    hosted_identity: Option<&HostedApiIdentityContract>,
) -> String {
    let hosted_route = hosted_identity
        .map(|identity| {
            format!(
                " Hosted routing is modeled through control plane '{}' at '{}' with audience '{}' and token source '{}'.",
                identity.control_plane,
                identity.endpoint,
                identity.audience,
                identity.auth.source.describe(),
            )
        })
        .unwrap_or_default();

    let detail = match provider {
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
    };

    format!("{detail}{hosted_route}")
}

fn launch_preflight_checks(
    machine: &port_model::MachineSpec,
    kernel_path: &Path,
    guest_image_path: &Path,
) -> Vec<DoctorCheck> {
    let mut checks = vec![
        path_check(
            "kvm-device",
            Path::new("/dev/kvm"),
            true,
            "Found /dev/kvm for KVM acceleration.",
            "Missing /dev/kvm.",
        ),
        versioned_binary_check("iproute2", "ip", &["-V"], "iproute2", true),
        versioned_binary_check("iptables", "iptables", &["--version"], "iptables", true),
        path_check(
            format!("artifact:{}", machine.kernel),
            kernel_path,
            true,
            &format!("Artifact variant '{}' exists.", kernel_path.display()),
            &format!(
                "Artifact variant '{}' is missing. Build or pull the selected variant first.",
                kernel_path.display()
            ),
        ),
        path_check(
            format!("artifact:{}", machine.guest_image),
            guest_image_path,
            true,
            &format!("Artifact variant '{}' exists.", guest_image_path.display()),
            &format!(
                "Artifact variant '{}' is missing. Build or pull the selected variant first.",
                guest_image_path.display()
            ),
        ),
    ];

    if machine.protection_mode == ProtectionMode::Standard {
        checks.push(binary_check("firecracker-binary", "firecracker", true));
    }

    checks
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

fn select_firecracker_binary(
    protection_mode: ProtectionMode,
    standard_binary: Option<PathBuf>,
    pvm_binary: Option<PathBuf>,
    pvm_host_kit: Option<&PvmHostKit>,
) -> Result<PathBuf> {
    match protection_mode {
        ProtectionMode::Standard => standard_binary
            .context("firecracker binary was not found on PATH after preflight"),
        ProtectionMode::Pvm => pvm_binary.context(match pvm_host_kit {
            Some(host_kit) => format!(
                "pvm host-kit preflight passed but {}; the standard firecracker binary is not a compatible fallback",
                pvm_firecracker_missing_detail(host_kit)
            ),
            None => String::from(
                "pvm host-kit preflight passed but the patched PVM Firecracker binary is still missing; the standard firecracker binary is not a compatible fallback",
            ),
        }),
    }
}

fn find_pvm_firecracker_binary() -> Option<PathBuf> {
    if let Some(configured) = env::var_os("PORT_PVM_FIRECRACKER_BINARY") {
        let path = PathBuf::from(configured);
        if path.is_file() {
            return Some(path);
        }
    }

    find_binary("firecracker-pvm")
}

fn find_pvm_firecracker_binary_for_host_kit(host_kit: &PvmHostKit) -> Option<PathBuf> {
    if let Some(variable) = host_kit.firecracker_binary_env.as_deref() {
        if let Some(configured) = env::var_os(variable) {
            let path = PathBuf::from(configured);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    find_binary(&host_kit.firecracker_binary_name)
}

fn run_artifact_pipeline(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
    action: ArtifactAction,
) -> Result<ArtifactMetadata> {
    ensure_native_build_lane(request.architecture)?;
    let artifact = resolve_artifact_metadata(config, request)?;
    let kind = artifact.kind;
    let script = artifact_script(kind, action)?;

    let status = Command::new(&script)
        .arg(&artifact.path)
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

    Ok(artifact)
}

fn ensure_native_build_lane(architecture: MachineArchitecture) -> Result<()> {
    let native = resolve_machine_architecture(MachineArchitecture::Native)
        .context("failed to determine host architecture")?;
    let requested = resolve_machine_architecture(architecture)
        .with_context(|| format!("failed to resolve requested architecture '{architecture:?}'"))?;
    if requested == native {
        Ok(())
    } else {
        bail!(
            "artifact build and validate pipelines currently run only for the native host architecture {:?}; requested {:?}",
            native,
            requested
        )
    }
}

fn resolve_native_standard_variant(spec: &port_model::ArtifactSpec) -> Option<&ArtifactVariant> {
    let architecture = resolve_machine_architecture(MachineArchitecture::Native).ok()?;
    spec.variant(
        architecture,
        ExecutionSubstrate::Firecracker,
        ProtectionMode::Standard,
    )
}

fn resolve_artifact_metadata(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactMetadata> {
    let (kind, spec) = config
        .artifacts
        .lookup_named(request.name)
        .with_context(|| format!("unknown artifact '{}'", request.name))?;
    let architecture = resolve_machine_architecture(request.architecture).with_context(|| {
        format!(
            "failed to resolve requested architecture '{:?}'",
            request.architecture
        )
    })?;
    let variant = spec
        .variant(architecture, request.substrate, request.protection_mode)
        .with_context(|| {
            format!(
                "artifact '{}' has no variant for {:?}/{:?}/{:?}",
                request.name, architecture, request.substrate, request.protection_mode
            )
        })?;
    Ok(ArtifactMetadata {
        name: request.name.to_string(),
        kind,
        reference: spec.reference.clone(),
        selector: variant.selector,
        path: variant.path.clone(),
        cache_path: cache_path_for(spec, variant),
    })
}

fn cache_path_for(spec: &port_model::ArtifactSpec, variant: &ArtifactVariant) -> PathBuf {
    spec.distribution
        .cache_root
        .join(&spec.reference.registry)
        .join(&spec.reference.repository)
        .join(&spec.reference.version)
        .join(architecture_dir(variant.selector.architecture))
        .join(substrate_dir(variant.selector.substrate))
        .join(protection_mode_dir(variant.selector.protection_mode))
        .join(
            variant
                .path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("artifact")),
        )
}

fn push_store_path(config: &PortConfig, artifact: &ArtifactMetadata) -> Result<PathBuf> {
    let (_, spec) = config
        .artifacts
        .lookup_named(&artifact.name)
        .with_context(|| format!("unknown artifact '{}'", artifact.name))?;
    store_path(&spec.distribution.push, artifact)
        .context("push backend does not support Port artifact publishing yet")
}

fn pull_store_path(config: &PortConfig, artifact: &ArtifactMetadata) -> Result<PathBuf> {
    let (_, spec) = config
        .artifacts
        .lookup_named(&artifact.name)
        .with_context(|| format!("unknown artifact '{}'", artifact.name))?;
    store_path(&spec.distribution.pull, artifact)
        .context("pull backend does not support Port artifact fetching yet")
}

fn store_path(store: &ArtifactStore, artifact: &ArtifactMetadata) -> Result<PathBuf> {
    match store {
        ArtifactStore::FileSystem { root } => Ok(root
            .join(&artifact.reference.registry)
            .join(&artifact.reference.repository)
            .join(&artifact.reference.version)
            .join(architecture_dir(artifact.selector.architecture))
            .join(substrate_dir(artifact.selector.substrate))
            .join(protection_mode_dir(artifact.selector.protection_mode))
            .join(
                artifact
                    .path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("artifact")),
            )),
        ArtifactStore::OciRegistry { reference } => bail!(
            "OCI registry backend '{}' is reserved in the model but not implemented in the runtime yet",
            reference
        ),
        ArtifactStore::HostedApi { endpoint } => bail!(
            "Hosted API backend '{}' is reserved in the model but not implemented in the runtime yet",
            endpoint
        ),
    }
}

fn copy_file(source: &Path, destination: &Path) -> Result<u64> {
    if !source.is_file() {
        bail!("artifact source '{}' does not exist", source.display());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy artifact from '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })
}

fn architecture_dir(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => "native",
        MachineArchitecture::X86_64 => "x86_64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn substrate_dir(substrate: ExecutionSubstrate) -> &'static str {
    match substrate {
        ExecutionSubstrate::Firecracker => "firecracker",
        ExecutionSubstrate::CloudHypervisor => "cloud-hypervisor",
        ExecutionSubstrate::Avf => "avf",
    }
}

fn protection_mode_dir(mode: ProtectionMode) -> &'static str {
    match mode {
        ProtectionMode::Standard => "standard",
        ProtectionMode::Pvm => "pvm",
    }
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

pub fn execute_guest_operation(
    config: &PortConfig,
    request: GuestRequest<'_>,
) -> Result<OperationResult> {
    if matches!(
        &request.operation,
        GuestOperation::Copy(_) | GuestOperation::Forward(_)
    ) {
        bail!("copy and forward use dedicated runtime flows");
    }

    if machine_is_hosted(config, request.machine_name)? {
        return hosted_control_plane_guest_operation(config, request);
    }

    let driver = driver_for_machine(config, request.machine_name)?;
    let endpoint = driver.guest_endpoint(config, &request)?;
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

fn hosted_control_plane_guest_operation(
    config: &PortConfig,
    request: GuestRequest<'_>,
) -> Result<OperationResult> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<OperationResult> = match request.operation {
        GuestOperation::Exec(exec) => client.execute_json(
            client
                .guest()
                .exec(request.machine_name, exec)
                .context("failed to encode hosted guest exec request")?,
        ),
        GuestOperation::Pty(pty) => client.execute_json(
            client
                .guest()
                .pty(request.machine_name, pty)
                .context("failed to encode hosted guest pty request")?,
        ),
        GuestOperation::Logs(logs) => client.execute_json(
            client
                .guest()
                .logs(request.machine_name, logs)
                .context("failed to encode hosted guest logs request")?,
        ),
        GuestOperation::Copy(_) | GuestOperation::Forward(_) => {
            bail!("copy and forward use dedicated runtime flows");
        }
    }
    .map_err(|error| {
        anyhow!(
            "failed to execute guest operation for machine '{}' through the live hosted control-plane route: {error}",
            request.machine_name
        )
    })?;
    Ok(response.result)
}

pub fn copy_guest_file(
    config: &PortConfig,
    request: GuestCopyRequest<'_>,
) -> Result<port_agent_protocol::CopyResult> {
    if machine_is_hosted(config, request.machine_name)? {
        return hosted_control_plane_copy_guest_file(config, request);
    }

    let driver = driver_for_machine(config, request.machine_name)?;
    let endpoint = driver.guest_endpoint(
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
                bail!("expected {size_bytes} bytes from guest copy, received {bytes_copied}");
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

fn hosted_control_plane_copy_guest_file(
    config: &PortConfig,
    request: GuestCopyRequest<'_>,
) -> Result<port_agent_protocol::CopyResult> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<port_agent_protocol::CopyResult> = client
        .execute_json(
            client
                .guest()
                .copy(
                    request.machine_name,
                    port_agent_protocol::CopyRequest {
                        source: request.source.display().to_string(),
                        destination: request.destination.display().to_string(),
                        direction: request.direction,
                        size_bytes: None,
                    },
                )
                .context("failed to encode hosted guest copy request")?,
        )
        .map_err(|error| {
            anyhow!(
                "failed to copy files for machine '{}' through the live hosted control-plane route: {error}",
                request.machine_name
            )
        })?;
    Ok(response.result)
}

pub struct GuestForwardSession {
    listener: ForwardListener,
    endpoint: GuestEndpoint,
    target: String,
}

#[derive(Debug)]
enum ForwardListener {
    Tcp(TcpListener),
    Unix {
        listener: UnixListener,
        socket_path: PathBuf,
    },
}

trait ProxyStream: Read + Write + Send + 'static {
    fn try_clone_stream(&self) -> std::io::Result<Self>
    where
        Self: Sized;

    fn shutdown_write(&self) -> std::io::Result<()>;
}

impl ProxyStream for TcpStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

impl ProxyStream for UnixStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

impl GuestForwardSession {
    #[must_use]
    pub fn listen_addr(&self) -> String {
        match &self.listener {
            ForwardListener::Tcp(listener) => listener
                .local_addr()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|_| String::from("<unknown>")),
            ForwardListener::Unix { socket_path, .. } => {
                render_forward_endpoint(&ForwardEndpoint::Unix(socket_path.clone()))
            }
        }
    }

    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn serve(self) -> Result<()> {
        match self.listener {
            ForwardListener::Tcp(listener) => {
                for inbound in listener.incoming() {
                    let inbound = inbound.context("failed to accept forwarded host connection")?;
                    let endpoint = self.endpoint.clone();
                    let target = self.target.clone();
                    thread::spawn(move || {
                        if let Err(error) =
                            proxy_guest_forward_connection(endpoint, target, inbound)
                        {
                            eprintln!("port guest forward connection failed: {error}");
                        }
                    });
                }
            }
            ForwardListener::Unix {
                listener,
                socket_path,
            } => {
                for inbound in listener.incoming() {
                    let inbound =
                        inbound.context("failed to accept forwarded Unix-socket connection")?;
                    let endpoint = self.endpoint.clone();
                    let target = self.target.clone();
                    thread::spawn(move || {
                        if let Err(error) =
                            proxy_guest_forward_connection(endpoint, target, inbound)
                        {
                            eprintln!("port guest forward connection failed: {error}");
                        }
                    });
                }

                let _ = fs::remove_file(socket_path);
            }
        }

        Ok(())
    }
}

pub fn prepare_guest_forward(
    config: &PortConfig,
    request: GuestForwardRequest<'_>,
) -> Result<GuestForwardSession> {
    let driver = driver_for_machine(config, request.machine_name)?;
    let endpoint = driver.guest_endpoint(
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
    let listener = bind_forward_listener(request.listen)?;
    Ok(GuestForwardSession {
        listener,
        endpoint,
        target: request.target.to_string(),
    })
}

pub fn guest_forward_state_dir(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &Path,
) -> Result<PathBuf> {
    Ok(RuntimePaths::for_machine(
        &resolve_guest_runtime_root(config, machine_name, runtime_root)?,
        machine_name,
    )
    .runtime_dir
    .join("forwards"))
}

fn resolve_guest_runtime_root(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &Path,
) -> Result<PathBuf> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{machine_name}'"))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;

    match &host.connection {
        HostConnection::Local => Ok(runtime_root.to_path_buf()),
        HostConnection::HostedControlPlane { .. } => {
            let resolution = hosted_machine_resolution(config, machine_name)?;
            let Some(node_name) = resolution.node_name else {
                bail!(
                    "control plane '{}' could not resolve a detached forward state root for machine '{}': {}",
                    resolution.control_plane,
                    machine_name,
                    resolution.status.detail
                );
            };
            let _ = node_name;
            Ok(resolution.runtime_root)
        }
    }
}

fn bind_forward_listener(listen: &str) -> Result<ForwardListener> {
    match parse_forward_endpoint(listen).map_err(|error| anyhow!(error.to_string()))? {
        ForwardEndpoint::Tcp(address) => {
            let listener = TcpListener::bind(&address)
                .with_context(|| format!("failed to bind '{address}'"))?;
            Ok(ForwardListener::Tcp(listener))
        }
        ForwardEndpoint::Unix(socket_path) => {
            if socket_path.exists() {
                fs::remove_file(&socket_path).with_context(|| {
                    format!(
                        "failed to remove stale Unix forward socket '{}'",
                        socket_path.display()
                    )
                })?;
            }
            if let Some(parent) = socket_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            let listener = UnixListener::bind(&socket_path).with_context(|| {
                format!("failed to bind Unix listener '{}'", socket_path.display())
            })?;
            Ok(ForwardListener::Unix {
                listener,
                socket_path,
            })
        }
    }
}

fn proxy_guest_forward_connection<S: ProxyStream>(
    endpoint: GuestEndpoint,
    target: String,
    inbound: S,
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
        .try_clone_stream()
        .context("failed to clone inbound forward socket")?;
    let mut inbound_write = inbound;

    let first = thread::spawn(move || {
        let result = std::io::copy(&mut inbound_read, &mut guest_write);
        let _ = guest_write.shutdown(Shutdown::Write);
        result
    });
    let second = thread::spawn(move || {
        let result = std::io::copy(&mut guest_read, &mut inbound_write);
        let _ = inbound_write.shutdown_write();
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

fn resolve_firecracker_guest_endpoint(
    config: &PortConfig,
    request: &GuestRequest<'_>,
) -> Result<GuestEndpoint> {
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

impl MachineDriver for FirecrackerLocalDriver {
    fn kind(&self) -> MachineDriverKind {
        MachineDriverKind::FirecrackerLocal
    }

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata> {
        firecracker_local_launch_machine(config, request)
    }

    fn list_machines(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
    ) -> Result<Vec<MachineStatus>> {
        firecracker_local_list_machines(runtime_root)
    }

    fn machine_status(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus> {
        firecracker_local_machine_status(runtime_root, machine_name)
    }

    fn stop_machine(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult> {
        firecracker_local_stop_machine(runtime_root, machine_name, timeout)
    }

    fn machine_monitor(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport> {
        firecracker_local_machine_monitor(runtime_root, machine_name)
    }

    fn machine_top(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport> {
        firecracker_local_machine_top(runtime_root, machine_name)
    }

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint> {
        resolve_firecracker_guest_endpoint(config, request)
    }
}

impl MachineDriver for HostedControlPlaneDriver {
    fn kind(&self) -> MachineDriverKind {
        MachineDriverKind::HostedControlPlane
    }

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata> {
        hosted_control_plane_launch_machine(config, request)
    }

    fn list_machines(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
    ) -> Result<Vec<MachineStatus>> {
        hosted_control_plane_list_machines(config)
    }

    fn machine_status(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus> {
        hosted_control_plane_machine_status(config, machine_name)
    }

    fn stop_machine(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult> {
        hosted_control_plane_stop_machine(config, machine_name, timeout)
    }

    fn machine_monitor(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport> {
        hosted_control_plane_machine_monitor(config, machine_name)
    }

    fn machine_top(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport> {
        hosted_control_plane_machine_top(config, machine_name)
    }

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint> {
        hosted_control_plane_guest_endpoint(config, request)
    }
}

fn local_runtime_driver() -> FirecrackerLocalDriver {
    FirecrackerLocalDriver
}

fn hosted_control_plane_driver() -> HostedControlPlaneDriver {
    HostedControlPlaneDriver
}

fn driver_for_machine(config: &PortConfig, machine_name: &str) -> Result<Box<dyn MachineDriver>> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{machine_name}'"))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    match &host.connection {
        HostConnection::HostedControlPlane { .. } => Ok(Box::new(HostedControlPlaneDriver)),
        HostConnection::Local => match machine.substrate {
            ExecutionSubstrate::Firecracker => Ok(Box::new(FirecrackerLocalDriver)),
            ExecutionSubstrate::CloudHypervisor => bail!(
                "machine '{}' targets Cloud Hypervisor, but Port has not implemented a Cloud Hypervisor driver yet",
                machine_name
            ),
            ExecutionSubstrate::Avf => bail!(
                "machine '{}' targets Apple Virtualization Framework, but Port has not implemented an AVF driver yet",
                machine_name
            ),
        },
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
    stream
        .flush()
        .context("failed to flush Firecracker handshake")?;

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
    use std::net::{Shutdown, TcpListener as StdTcpListener, TcpStream};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use std::time::Duration;

    use tempfile::tempdir;

    use super::{
        ArtifactAction, ArtifactRequest, ControlPlaneServeRequest, DoctorCheck, DoctorHostFacts,
        GuestCopyRequest, GuestForwardRequest, GuestRequest, HostedNodeBinding, LaunchMetadata,
        LaunchRequest, MachineDriverKind, MachineRuntimeState, NodeAgentServeRequest, RuntimePaths,
        StopResult, artifact_script, build_firecracker_config, collect_doctor_report,
        collect_doctor_report_with_facts, copy_guest_file, driver_for_machine,
        ensure_native_build_lane, execute_guest_operation, launch_local_machine, list_machines,
        machine_monitor, machine_status, machine_top, path_check, prepare_guest_forward,
        prepare_runtime_state, read_pid_file, repo_root, resolve_artifact_metadata,
        resolve_machine_architecture, select_firecracker_binary, serve_control_plane,
        serve_node_agent, stop_machine,
    };
    use port_agent_protocol::{
        CopyDirection, ExecRequest, ExecResult, GuestOperation, OperationResult, RequestEnvelope,
        ResponseEnvelope, StreamKind, read_frame, write_frame,
    };
    use port_model::{
        ArtifactKind, ExecutionSubstrate, FirecrackerSupport, HostConnection, HostPlatform,
        HostProvider, HostSpec, MachineArchitecture, PortConfig, ProtectionMode,
    };

    fn sample_config_with_hosted_runtime_roots(root: &Path) -> PortConfig {
        let mut config = PortConfig::sample();
        config
            .nodes
            .get_mut("generic-linux-node")
            .expect("generic-linux-node should exist")
            .runtime_root = root.join("hosted/generic-linux-node");
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .runtime_root = root.join("hosted/aws-linux-node");
        config
            .nodes
            .get_mut("gcp-linux-node")
            .expect("gcp-linux-node should exist")
            .runtime_root = root.join("hosted/gcp-linux-node");
        config
    }

    fn hosted_server_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn reserve_addr() -> String {
        let listener = StdTcpListener::bind("127.0.0.1:0").expect("port should bind");
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

    fn write_fake_firecracker_binary(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        fs::write(&path, "#!/usr/bin/env bash\nsleep 30\n").expect("fake firecracker should write");
        let mut permissions = fs::metadata(&path)
            .expect("fake firecracker metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .expect("fake firecracker permissions should update");
        path
    }

    fn start_live_hosted_servers(config: &PortConfig, bind_node: bool) -> PortConfig {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let node_addr = reserve_addr();
        let control_plane_addr = reserve_addr();
        let mut client_config = config.clone();
        client_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{control_plane_addr}");

        if bind_node {
            let node_config = client_config.clone();
            let bind = node_addr.clone();
            thread::spawn(move || {
                serve_node_agent(
                    node_config,
                    NodeAgentServeRequest {
                        node_name: String::from("aws-linux-node"),
                        bind,
                        token: String::from("node-secret"),
                    },
                )
                .expect("node-agent should serve");
            });
            wait_for_tcp(&node_addr);
        }

        let control_config = client_config.clone();
        let bind = control_plane_addr.clone();
        let node_bindings = if bind_node {
            vec![HostedNodeBinding {
                node_name: String::from("aws-linux-node"),
                endpoint: format!("http://{node_addr}"),
                token: String::from("node-secret"),
            }]
        } else {
            Vec::new()
        };
        thread::spawn(move || {
            serve_control_plane(
                control_config,
                ControlPlaneServeRequest {
                    control_plane: String::from("demo"),
                    bind,
                    node_bindings,
                },
            )
            .expect("control plane should serve");
        });
        wait_for_tcp(&control_plane_addr);

        client_config
    }

    fn write_manifest(paths: &RuntimePaths, machine_name: &str, pid: u32) {
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let manifest = LaunchMetadata {
            machine_name: String::from(machine_name),
            pid,
            launched_at_unix_s: 1,
            runtime_dir: paths.runtime_dir.clone(),
            firecracker_binary: PathBuf::from("/usr/bin/firecracker"),
            config_path: paths.config_path.clone(),
            log_path: paths.firecracker_log.clone(),
            stdout_path: paths.stdout_log.clone(),
            stderr_path: paths.stderr_log.clone(),
            manifest_path: paths.manifest_path.clone(),
        };
        fs::write(
            &paths.manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");
    }

    fn write_detached_forward_manifest(
        paths: &RuntimePaths,
        name: &str,
        pid: u32,
        listen: &str,
        target: &str,
    ) {
        let forwards_dir = paths.runtime_dir.join("forwards");
        fs::create_dir_all(&forwards_dir).expect("forwards dir should exist");
        let manifest = serde_json::json!({
            "name": name,
            "machine": paths.runtime_dir.file_name().expect("machine dir should exist").to_string_lossy(),
            "pid": pid,
            "listen": listen,
            "target": target,
            "stdout_log": paths.runtime_dir.join(format!("{name}.forward.stdout.log")),
            "stderr_log": paths.runtime_dir.join(format!("{name}.forward.stderr.log")),
        });
        fs::write(
            forwards_dir.join(format!("{name}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&manifest).expect("manifest should serialize")
            ),
        )
        .expect("forward manifest should write");
    }

    #[test]
    fn driver_selection_routes_demo_machine_to_firecracker_local_driver() {
        let config = PortConfig::sample();
        let driver = driver_for_machine(&config, "demo").expect("driver should resolve");

        assert_eq!(driver.kind(), MachineDriverKind::FirecrackerLocal);
    }

    #[test]
    fn driver_selection_routes_hosted_machine_to_control_plane_driver() {
        let config = PortConfig::sample();
        let driver = driver_for_machine(&config, "cloud-aws").expect("driver should resolve");

        assert_eq!(driver.kind(), MachineDriverKind::HostedControlPlane);
    }

    #[test]
    fn driver_selection_rejects_avf_lane_without_driver() {
        let mut config = PortConfig::sample();
        config.hosts.insert(
            String::from("mac-local"),
            HostSpec {
                platform: HostPlatform::Macos,
                provider: HostProvider::Local,
                connection: HostConnection::Local,
                firecracker: FirecrackerSupport {
                    local_launch: false,
                    pvm_lanes: Vec::new(),
                    notes: Vec::new(),
                },
            },
        );
        let machine = config
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.host = String::from("mac-local");
        machine.substrate = ExecutionSubstrate::Avf;

        let error = driver_for_machine(&config, "demo")
            .err()
            .expect("AVF should not resolve");
        assert!(error.to_string().contains("AVF driver"));
    }

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
    fn resolve_artifact_metadata_distinguishes_standard_and_pvm_paths() {
        let config = PortConfig::sample();

        let standard = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: port_model::ProtectionMode::Standard,
            },
        )
        .expect("standard kernel metadata should resolve");
        let pvm = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: port_model::ProtectionMode::Pvm,
            },
        )
        .expect("pvm kernel metadata should resolve");

        assert_ne!(standard.path, pvm.path);
        assert_eq!(
            pvm.path,
            PathBuf::from("artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux")
        );
        assert_eq!(
            pvm.cache_path,
            PathBuf::from(".port/cache/demo-fs/port/demo-kernel/v1/x86_64/firecracker/pvm/vmlinux")
        );
    }

    #[test]
    fn resolve_artifact_metadata_accepts_the_native_alias() {
        let config = PortConfig::sample();

        let native = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::Native,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: port_model::ProtectionMode::Standard,
            },
        )
        .expect("native selector should resolve");

        assert_eq!(native.selector.architecture, MachineArchitecture::X86_64);
        assert_eq!(
            native.path,
            PathBuf::from("artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux")
        );
    }

    #[test]
    fn native_artifact_build_lane_accepts_the_native_alias() {
        ensure_native_build_lane(MachineArchitecture::Native)
            .expect("native alias should resolve to the host architecture");
        let concrete = resolve_machine_architecture(MachineArchitecture::Native)
            .expect("host architecture should resolve");
        ensure_native_build_lane(concrete)
            .expect("concrete native architecture should remain accepted");
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
    fn doctor_report_includes_machine_lane_checks() {
        let report = collect_doctor_report(Some(&PortConfig::sample()));

        let demo = report
            .checks
            .iter()
            .find(|check| check.name == "machine:demo")
            .expect("machine lane check should exist");

        assert!(demo.ok);
        assert!(demo.detail.contains("Machine models"));
        assert!(demo.detail.contains("Firecracker"));
    }

    #[test]
    fn doctor_report_includes_pvm_host_kit_checks_for_local_contract() {
        let report = collect_doctor_report_with_facts(
            Some(&PortConfig::sample()),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: String::from("x86_64"),
                proc_cmdline: Some(String::from("console=ttyS0 pti=off")),
                pvm_firecracker_binary: Some(PathBuf::from("/usr/bin/firecracker-pvm")),
            },
        );

        let platform = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:host-platform")
            .expect("pvm platform check should exist");
        let architecture = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:host-architecture")
            .expect("pvm architecture check should exist");
        let boot_line = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:boot-line")
            .expect("pvm boot-line check should exist");
        let binary = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:firecracker-binary")
            .expect("pvm binary check should exist");
        let arm64 = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:aarch64")
            .expect("arm64 research-only check should exist");

        assert!(platform.ok);
        assert!(architecture.ok);
        assert!(boot_line.ok);
        assert!(boot_line.detail.contains("pti=off"));
        assert!(binary.ok);
        assert!(binary.detail.contains("firecracker-pvm"));
        assert!(!arm64.ok);
        assert!(arm64.detail.contains("research-only"));
    }

    #[test]
    fn doctor_report_fails_fast_for_missing_pvm_boot_arg_and_binary() {
        let report = collect_doctor_report_with_facts(
            Some(&PortConfig::sample()),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: String::from("aarch64"),
                proc_cmdline: Some(String::from("console=ttyS0")),
                pvm_firecracker_binary: None,
            },
        );

        let architecture = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:host-architecture")
            .expect("pvm architecture check should exist");
        let boot_line = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:boot-line")
            .expect("pvm boot-line check should exist");
        let binary = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:local:x86_64:firecracker-binary")
            .expect("pvm binary check should exist");

        assert!(!architecture.ok);
        assert!(architecture.detail.contains("x86_64"));
        assert!(!boot_line.ok);
        assert!(boot_line.detail.contains("pti=off"));
        assert!(
            boot_line
                .detail
                .contains("standard Firecracker lane is not a PVM fallback")
        );
        assert!(!binary.ok);
        assert!(binary.detail.contains("firecracker-pvm"));
        assert!(
            binary
                .detail
                .contains("standard firecracker binary is not compatible")
        );
    }

    #[test]
    fn doctor_report_includes_hosted_pvm_host_kit_contract_checks() {
        let report = collect_doctor_report_with_facts(
            Some(&PortConfig::sample()),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: String::from("x86_64"),
                proc_cmdline: Some(String::from("console=ttyS0 pti=off")),
                pvm_firecracker_binary: Some(PathBuf::from("/usr/bin/firecracker-pvm")),
            },
        );

        let aws = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:aws-linux-node:x86_64:host-kit-contract")
            .expect("aws hosted host-kit contract check should exist");
        let generic = report
            .checks
            .iter()
            .find(|check| check.name == "pvm:generic-linux-node:x86_64:host-kit-contract")
            .expect("generic hosted host-kit contract check should exist");

        assert!(aws.ok);
        assert!(aws.detail.contains("firecracker-pvm"));
        assert!(aws.detail.contains("PORT_PVM_FIRECRACKER_BINARY"));
        assert!(!generic.ok);
        assert!(generic.detail.contains("host-kit contract"));
    }

    #[test]
    fn list_machines_reports_running_stale_and_malformed_runtime_entries() {
        let tempdir = tempdir().expect("tempdir should exist");
        let running_paths = RuntimePaths::for_machine(tempdir.path(), "running");
        let stale_paths = RuntimePaths::for_machine(tempdir.path(), "stale");
        let malformed_paths = RuntimePaths::for_machine(tempdir.path(), "broken");
        fs::create_dir_all(&running_paths.runtime_dir).expect("running dir should exist");
        fs::create_dir_all(&stale_paths.runtime_dir).expect("stale dir should exist");
        fs::create_dir_all(&malformed_paths.runtime_dir).expect("broken dir should exist");

        let mut command = Command::new("bash");
        command
            .args([
                "-lc",
                "exec -a firecracker /bin/sh -c 'sleep 30' --id running",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .expect("fake running firecracker process should start");
        thread::sleep(Duration::from_millis(100));

        let running_manifest = LaunchMetadata {
            machine_name: String::from("running"),
            pid: child.id(),
            launched_at_unix_s: 1,
            runtime_dir: running_paths.runtime_dir.clone(),
            firecracker_binary: PathBuf::from("/usr/bin/firecracker"),
            config_path: running_paths.config_path.clone(),
            log_path: running_paths.firecracker_log.clone(),
            stdout_path: running_paths.stdout_log.clone(),
            stderr_path: running_paths.stderr_log.clone(),
            manifest_path: running_paths.manifest_path.clone(),
        };
        fs::write(
            &running_paths.manifest_path,
            serde_json::to_vec_pretty(&running_manifest).expect("manifest should serialize"),
        )
        .expect("running manifest should write");
        fs::write(&running_paths.pid_path, format!("{}\n", child.id()))
            .expect("running pid should write");

        let stale_manifest = LaunchMetadata {
            machine_name: String::from("stale"),
            pid: 424242,
            launched_at_unix_s: 2,
            runtime_dir: stale_paths.runtime_dir.clone(),
            firecracker_binary: PathBuf::from("/usr/bin/firecracker"),
            config_path: stale_paths.config_path.clone(),
            log_path: stale_paths.firecracker_log.clone(),
            stdout_path: stale_paths.stdout_log.clone(),
            stderr_path: stale_paths.stderr_log.clone(),
            manifest_path: stale_paths.manifest_path.clone(),
        };
        fs::write(
            &stale_paths.manifest_path,
            serde_json::to_vec_pretty(&stale_manifest).expect("manifest should serialize"),
        )
        .expect("stale manifest should write");
        fs::write(&stale_paths.pid_path, "424242\n").expect("stale pid should write");
        fs::write(&malformed_paths.manifest_path, "{not-json\n")
            .expect("malformed manifest should write");

        let mut config = PortConfig::sample();
        config.machines.retain(|name, _| name == "demo");

        let machines =
            list_machines(&config, tempdir.path()).expect("machine listing should succeed");
        assert_eq!(machines.len(), 3);

        let running = machines
            .iter()
            .find(|machine| machine.machine_name == "running")
            .expect("running machine should exist");
        assert_eq!(running.state, MachineRuntimeState::Running);
        assert_eq!(running.pid, Some(child.id()));

        let stale = machines
            .iter()
            .find(|machine| machine.machine_name == "stale")
            .expect("stale machine should exist");
        assert_eq!(stale.state, MachineRuntimeState::Stale);
        assert_eq!(stale.pid, Some(424242));
        assert!(stale.detail.contains("no longer live"));

        let broken = machines
            .iter()
            .find(|machine| machine.machine_name == "broken")
            .expect("broken machine should exist");
        assert_eq!(broken.state, MachineRuntimeState::Malformed);
        assert!(broken.detail.contains("failed to parse"));

        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn machine_status_reports_runtime_paths_for_known_machine() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let manifest = LaunchMetadata {
            machine_name: String::from("demo"),
            pid: 99,
            launched_at_unix_s: 1,
            runtime_dir: paths.runtime_dir.clone(),
            firecracker_binary: PathBuf::from("/usr/bin/firecracker"),
            config_path: paths.config_path.clone(),
            log_path: paths.firecracker_log.clone(),
            stdout_path: paths.stdout_log.clone(),
            stderr_path: paths.stderr_log.clone(),
            manifest_path: paths.manifest_path.clone(),
        };
        fs::write(
            &paths.manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");

        let status = machine_status(&PortConfig::sample(), tempdir.path(), "demo")
            .expect("status should load");
        assert_eq!(status.machine_name, "demo");
        assert_eq!(status.state, MachineRuntimeState::Stopped);
        assert_eq!(
            status.control,
            port_model::MachineControlContract::local_runtime_root()
        );
        assert_eq!(status.runtime_dir, paths.runtime_dir);
        assert_eq!(status.config_path, paths.config_path);
        assert_eq!(status.manifest_path, paths.manifest_path);
        assert_eq!(status.pid_path, paths.pid_path);
        assert_eq!(status.firecracker_log, paths.firecracker_log);
        assert_eq!(status.stdout_log, paths.stdout_log);
        assert_eq!(status.stderr_log, paths.stderr_log);
    }

    #[test]
    fn stop_machine_terminates_live_port_owned_process() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.vsock_path, "").expect("vsock path should write");
        fs::write(&paths.guest_agent_socket, "").expect("guest socket should write");

        let mut command = Command::new("bash");
        command
            .args(["-lc", "exec -a firecracker /bin/sh -c 'sleep 30' --id demo"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .expect("fake firecracker process should start");
        thread::sleep(Duration::from_millis(100));

        let manifest = LaunchMetadata {
            machine_name: String::from("demo"),
            pid: child.id(),
            launched_at_unix_s: 1,
            runtime_dir: paths.runtime_dir.clone(),
            firecracker_binary: PathBuf::from("/usr/bin/firecracker"),
            config_path: paths.config_path.clone(),
            log_path: paths.firecracker_log.clone(),
            stdout_path: paths.stdout_log.clone(),
            stderr_path: paths.stderr_log.clone(),
            manifest_path: paths.manifest_path.clone(),
        };
        fs::write(
            &paths.manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");
        fs::write(&paths.pid_path, format!("{}\n", child.id())).expect("pid file should write");

        let result = stop_machine(
            &PortConfig::sample(),
            tempdir.path(),
            "demo",
            Duration::from_secs(2),
        )
        .expect("stop should succeed");
        assert_eq!(
            result,
            StopResult {
                machine_name: String::from("demo"),
                previous_state: MachineRuntimeState::Running,
                current_state: MachineRuntimeState::Stopped,
                pid: Some(child.id()),
                control: port_model::MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir.clone(),
                detail: String::from("sent SIGTERM to pid and cleaned stale runtime sockets"),
            }
        );
        assert_eq!(
            read_pid_file(&paths.pid_path).expect("pid file should be readable"),
            None
        );
        assert!(!paths.vsock_path.exists());
        assert!(!paths.guest_agent_socket.exists());

        let _ = child.wait();
    }

    #[test]
    fn hosted_machine_status_uses_control_plane_and_node_runtime_root() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        write_manifest(&paths, "cloud-aws", 424242);
        let config = start_live_hosted_servers(&config, true);

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted status should load");
        assert_eq!(status.machine_name, "cloud-aws");
        assert_eq!(status.state, MachineRuntimeState::Stopped);
        assert_eq!(
            status.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert_eq!(status.runtime_dir, paths.runtime_dir);
        assert!(status.detail.contains("control plane 'demo'"));
        assert!(status.detail.contains("node 'aws-linux-node'"));
    }

    #[test]
    fn list_machines_includes_hosted_control_plane_statuses() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let hosted_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let hosted_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws");
        write_manifest(&hosted_paths, "cloud-aws", 424242);
        let config = start_live_hosted_servers(&config, true);

        let machines = list_machines(&config, tempdir.path()).expect("machine list should load");
        let hosted = machines
            .iter()
            .find(|machine| machine.machine_name == "cloud-aws")
            .expect("hosted machine should appear in machine list");
        assert_eq!(
            hosted.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert_eq!(hosted.runtime_dir, hosted_paths.runtime_dir);
    }

    #[test]
    fn hosted_stop_machine_routes_through_node_runtime_root() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.vsock_path, "").expect("vsock path should write");
        fs::write(&paths.guest_agent_socket, "").expect("guest socket should write");

        let mut command = Command::new("bash");
        command
            .args([
                "-lc",
                "exec -a firecracker /bin/sh -c 'sleep 30' --id cloud-aws",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .expect("fake hosted firecracker process should start");
        thread::sleep(Duration::from_millis(100));

        write_manifest(&paths, "cloud-aws", child.id());
        fs::write(&paths.pid_path, format!("{}\n", child.id())).expect("pid file should write");
        let config = start_live_hosted_servers(&config, true);

        let result = stop_machine(&config, tempdir.path(), "cloud-aws", Duration::from_secs(2))
            .expect("hosted stop should succeed");
        assert_eq!(
            result.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert_eq!(result.previous_state, MachineRuntimeState::Running);
        assert_eq!(result.current_state, MachineRuntimeState::Stopped);
        assert!(result.detail.contains("control plane 'demo'"));
        assert!(result.detail.contains("node 'aws-linux-node'"));

        let _ = child.wait();
    }

    #[test]
    fn hosted_guest_exec_routes_through_node_runtime_root() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("should accept hosted guest transport");
            let reader_stream = stream.try_clone().expect("stream should clone");
            let mut reader = BufReader::new(reader_stream);

            let request: RequestEnvelope = read_frame(&mut reader).expect("request should decode");
            match request.operation {
                GuestOperation::Exec(request) => {
                    assert_eq!(
                        request.command,
                        vec![String::from("/bin/echo"), String::from("hosted-ok")]
                    );
                }
                other => panic!("unexpected hosted guest operation: {other:?}"),
            }

            write_frame(
                &mut stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Exec(ExecResult {
                        stdout: String::from("hosted-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("response should encode");
        });
        let config = start_live_hosted_servers(&config, true);

        let result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("hosted-ok")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("hosted guest exec should succeed");

        match result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "hosted-ok\n"),
            other => panic!("unexpected result: {other:?}"),
        }

        server.join().expect("server thread should complete");
    }

    #[test]
    fn hosted_guest_exec_explains_unresolved_node_routing() {
        let config = start_live_hosted_servers(&PortConfig::sample(), false);
        let error = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-azure",
                runtime_root: Path::new("runtime"),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/true")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect_err("unresolved hosted node routing should fail");

        let message = error.to_string();
        assert!(
            message.contains("control plane 'demo'") || message.contains("control-plane=demo"),
            "{message}"
        );
        assert!(message.contains("cloud-azure"));
        assert!(message.contains("no hosted node inventory record matches that host"));
    }

    #[test]
    fn hosted_pvm_status_surfaces_node_readiness_denial() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-generic");
        config
            .machines
            .get_mut("cloud-generic")
            .expect("cloud-generic should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let config = start_live_hosted_servers(&config, true);

        let status = machine_status(&config, tempdir.path(), "cloud-generic")
            .expect("hosted pvm status should load");

        assert_eq!(status.state, MachineRuntimeState::Malformed);
        assert!(status.detail.contains("generic-linux-node"));
        assert!(status.detail.contains("planned"));
        assert!(status.detail.contains("PVM"));
    }

    #[test]
    fn hosted_pvm_launch_rejects_unplaceable_nodes_before_remote_guidance() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-generic")
            .expect("cloud-generic should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-generic",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("hosted pvm launch should fail fast");

        let message = error.to_string();
        assert!(message.contains("generic-linux-node"));
        assert!(message.contains("planned"));
        assert!(message.contains("PVM"));
    }

    #[test]
    fn hosted_pvm_launch_routes_through_live_control_plane_and_prepared_node() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = ProtectionMode::Pvm;

        let kernel_path = tempdir.path().join("pvm-vmlinux");
        let guest_path = tempdir.path().join("pvm-rootfs.ext4");
        fs::write(&kernel_path, b"fake-kernel").expect("kernel variant should write");
        fs::write(&guest_path, b"fake-rootfs").expect("guest variant should write");

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
            .path = kernel_path.clone();
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
            .path = guest_path.clone();

        let host_kit = config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws node should exist")
            .capabilities
            .pvm_lanes[0]
            .host_kit
            .as_mut()
            .expect("aws node should declare a host-kit");
        host_kit.requires_custom_host_kernel = false;
        host_kit.host_boot_args.clear();
        host_kit.firecracker_binary_env = Some(String::from("PORT_TEST_HOSTED_PVM_FIRECRACKER"));
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "firecracker-pvm");
        unsafe {
            std::env::set_var("PORT_TEST_HOSTED_PVM_FIRECRACKER", &fake_binary);
        }

        let config = start_live_hosted_servers(&config, true);
        let metadata = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect("hosted pvm launch should route through live control plane");

        assert_eq!(metadata.machine_name, "cloud-aws");
        assert_eq!(metadata.firecracker_binary, fake_binary);
        assert!(metadata.manifest_path.exists());

        let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
    }

    #[test]
    fn hosted_machine_monitor_reports_node_runtime_and_detached_forward_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        write_manifest(&paths, "cloud-aws", 424242);

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

        write_detached_forward_manifest(
            &paths,
            "web",
            child.id(),
            "127.0.0.1:8081",
            "127.0.0.1:80",
        );
        let config = start_live_hosted_servers(&config, true);

        let report =
            machine_monitor(&config, tempdir.path(), "cloud-aws").expect("monitor should load");
        assert_eq!(report.machine_name, "cloud-aws");
        assert_eq!(report.control_plane.as_deref(), Some("demo"));
        assert_eq!(report.node_name.as_deref(), Some("aws-linux-node"));
        assert!(report.host_groups.contains(&String::from("aws-builders")));
        assert_eq!(report.detached_forwards.len(), 1);
        let forward = &report.detached_forwards[0];
        assert_eq!(forward.name, "web");
        assert_eq!(forward.state, MachineRuntimeState::Running);
        assert_eq!(forward.pid, Some(child.id()));
        assert_eq!(forward.listen, "127.0.0.1:8081");
        assert_eq!(forward.target, "127.0.0.1:80");

        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn hosted_machine_top_reports_hypervisor_and_detached_forward_processes() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");

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
        write_manifest(&paths, "cloud-aws", firecracker.id());
        fs::write(&paths.pid_path, format!("{}\n", firecracker.id())).expect("pid should write");

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
        write_detached_forward_manifest(
            &paths,
            "web",
            child.id(),
            "127.0.0.1:8081",
            "127.0.0.1:80",
        );
        let config = start_live_hosted_servers(&config, true);

        let report = machine_top(&config, tempdir.path(), "cloud-aws").expect("top should load");
        let hypervisor = report
            .entries
            .iter()
            .find(|entry| entry.name == "firecracker")
            .expect("hypervisor entry should exist");
        assert_eq!(hypervisor.state, MachineRuntimeState::Running);
        assert_eq!(hypervisor.pid, Some(firecracker.id()));
        assert!(
            hypervisor
                .command
                .as_deref()
                .expect("command should exist")
                .contains("firecracker")
        );

        let forward = report
            .entries
            .iter()
            .find(|entry| entry.name == "web")
            .expect("forward entry should exist");
        assert_eq!(forward.state, MachineRuntimeState::Running);
        assert_eq!(forward.pid, Some(child.id()));
        assert!(
            forward
                .command
                .as_deref()
                .expect("command should exist")
                .contains("port-forward")
        );

        let _ = child.kill();
        let _ = child.wait();
        let _ = firecracker.kill();
        let _ = firecracker.wait();
    }

    #[test]
    fn machine_status_reports_missing_and_malformed_runtime_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let error = machine_status(&PortConfig::sample(), tempdir.path(), "missing")
            .expect_err("missing machine should fail");
        assert!(
            error
                .to_string()
                .contains("runtime state for machine 'missing' does not exist")
        );

        let broken_paths = RuntimePaths::for_machine(tempdir.path(), "broken");
        fs::create_dir_all(&broken_paths.runtime_dir).expect("broken runtime dir should exist");
        fs::write(&broken_paths.manifest_path, "{not-json\n")
            .expect("malformed manifest should write");

        let broken = machine_status(&PortConfig::sample(), tempdir.path(), "broken")
            .expect("broken status should load");
        assert_eq!(broken.state, MachineRuntimeState::Malformed);
        assert_eq!(
            broken.control,
            port_model::MachineControlContract::local_runtime_root()
        );
        assert!(broken.detail.contains("failed to parse"));
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
    fn launch_rejects_pvm_host_kit_when_runtime_is_not_prepared() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("demo")
            .expect("demo should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let tempdir = tempdir().expect("tempdir should exist");

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("launch should reject an unprepared PVM host kit");

        let message = error.to_string();
        assert!(message.contains("pvm host-kit preflight failed"));
        assert!(message.contains("pti=off"));
        assert!(message.contains("firecracker-pvm"));
    }

    #[test]
    fn launch_rejects_malformed_pvm_host_kit_contract_with_explicit_detail() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("demo")
            .expect("demo should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        config
            .hosts
            .get_mut("local")
            .expect("local host should exist")
            .firecracker
            .pvm_lanes[0]
            .host_kit
            .as_mut()
            .expect("x86 host kit should exist")
            .firecracker_binary_name
            .clear();
        let tempdir = tempdir().expect("tempdir should exist");

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("launch should reject a malformed PVM host-kit contract");

        let message = error.to_string();
        assert!(message.contains("host-kit contract"));
        assert!(message.contains("firecracker binary"));
    }

    #[test]
    fn firecracker_binary_selection_uses_the_pvm_lane_without_standard_fallback() {
        let standard = PathBuf::from("/usr/bin/firecracker");
        let pvm = PathBuf::from("/usr/bin/firecracker-pvm");
        let sample = PortConfig::sample();
        let host_kit = sample.hosts["local"].firecracker.pvm_lanes[0]
            .host_kit
            .as_ref();

        assert_eq!(
            select_firecracker_binary(
                port_model::ProtectionMode::Standard,
                Some(standard.clone()),
                Some(pvm.clone()),
                None,
            )
            .expect("standard lane should use the standard binary"),
            standard
        );
        assert_eq!(
            select_firecracker_binary(
                port_model::ProtectionMode::Pvm,
                Some(standard),
                Some(pvm.clone()),
                host_kit,
            )
            .expect("pvm lane should use the patched binary"),
            pvm
        );

        let error =
            select_firecracker_binary(port_model::ProtectionMode::Pvm, None, None, host_kit)
                .expect_err("pvm lane should require the patched binary");
        let message = error.to_string();
        assert!(message.contains("firecracker-pvm"));
        assert!(message.contains("not a compatible fallback"));
    }

    #[test]
    fn guest_operations_explain_missing_live_vm_transport_socket() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        fs::write(&paths.manifest_path, "{}\n").expect("manifest marker should write");

        let error = execute_guest_operation(
            &PortConfig::sample(),
            GuestRequest {
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
            },
        )
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
            stream
                .write_all(b"OK\n")
                .expect("should acknowledge handshake");
            stream.flush().expect("should flush handshake response");

            let request: RequestEnvelope = read_frame(&mut reader).expect("request should decode");
            match request.operation {
                GuestOperation::Exec(request) => {
                    assert_eq!(
                        request.command,
                        vec![String::from("/bin/echo"), String::from("live-ok")]
                    );
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
        let serve_thread =
            thread::spawn(move || session.serve().expect("forward serve should run"));

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
        forwarded
            .shutdown(Shutdown::Write)
            .expect("forward shutdown");
        let mut echoed = Vec::new();
        forwarded
            .read_to_end(&mut echoed)
            .expect("forward read should complete");
        assert_eq!(echoed, b"forward-ok");

        let _ = serve_thread.thread().id();
        server
            .join()
            .expect("forward server thread should complete");
    }
}
