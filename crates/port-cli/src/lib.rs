use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use port_agent_protocol::{
    CopyDirection, ExecRequest, ForwardRequest, GuestOperation, LogsRequest, OperationResult,
    PtyRequest,
};
use port_model::{
    ExecutionSubstrate, HostConnection, HostedSchedulerPolicy, MachineArchitecture,
    MachineControlContract, MachineRuntimeClassSpec, MachineVolumeBackend,
    MachineVolumePersistence, MachineVolumeSpec, PortConfig, ProtectionMode, PvmHostKitPackage,
};
use port_runtime::{
    ArtifactRequest, ClusterDownRequest, ClusterStageRequest, ClusterStatusRequest,
    ClusterUpRequest, ControlPlaneServeRequest, DoctorReport, GuestCopyRequest,
    GuestForwardRequest, GuestRequest, HostedNodeBinding, HostedPvmNodePrepareRequest,
    LaunchRequest, NodeAgentServeRequest, ServiceHealthPolicy, ServiceHealthcheck, ServicePolicy,
    ServiceRestartPolicy,
};
use serde::{Deserialize, Serialize};

mod upgrade;

const AFTER_HELP: &str = "\
Quick start:
  `port` uses the built-in sample model when `--config` is omitted.
  Use `--config examples/port.toml` for the checked-in repo workflow.

Examples:
  port --config examples/port.toml artifacts list
  port doctor
  port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
  port --config examples/port.toml cluster show --cluster demo
  port --config examples/port.toml cluster up --cluster demo --runtime-root /tmp/port-runtime
  port --config examples/port.toml cluster kubeconfig --cluster demo --runtime-root /tmp/port-runtime --format json
  port --config examples/port.toml machine list
  port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'cat /proc/version'";

#[derive(Debug, Parser)]
#[command(
    name = "port",
    version,
    about = "CLI-first Firecracker orchestration for local and cloud Linux hosts",
    long_about = "Port manages microVM-backed workloads through one canonical CLI and shared machine model. Firecracker with standard protection on Linux is the default local lane; `cluster`, `artifacts`, `machine`, `guest`, and `service` reuse the same model while hosted control-plane, Cloud Hypervisor, and Apple Virtualization Framework lanes stay explicit.",
    after_help = AFTER_HELP
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load a Port model from a TOML file instead of using the built-in sample"
    )]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Inspect platform support and host requirements")]
    Doctor {
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Install the latest Port release or a specific git revision")]
    Upgrade(UpgradeCommand),
    #[command(subcommand, about = "Build and validate kernel or guest artifacts")]
    Artifacts(ArtifactCommand),
    #[command(
        subcommand,
        about = "Operate named local and hosted K3s cluster contracts"
    )]
    Cluster(ClusterCommand),
    #[command(subcommand, about = "Launch and inspect Port-managed machines")]
    Machine(MachineCommand),
    #[command(subcommand, about = "Reach guest agent capabilities")]
    Guest(GuestCommand),
    #[command(
        subcommand,
        about = "Manage machine-bound secrets, services, and sandboxes"
    )]
    Service(ServiceCommand),
    #[command(subcommand, about = "Serve hosted control-plane endpoints")]
    ControlPlane(ControlPlaneCommand),
    #[command(subcommand, about = "Serve hosted node-agent endpoints")]
    NodeAgent(NodeAgentCommand),
    #[command(subcommand, hide = true)]
    Internal(InternalCommand),
}

#[derive(Debug, Clone, Args)]
pub struct UpgradeCommand {
    #[arg(
        long,
        value_name = "TAG",
        conflicts_with = "sha",
        help = "Build and install a specific git tag from source"
    )]
    pub tag: Option<String>,

    #[arg(
        long,
        value_name = "SHA",
        conflicts_with = "tag",
        help = "Build and install a specific git commit from source"
    )]
    pub sha: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CopyDirectionArg {
    HostToGuest,
    GuestToHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ForwardLifecycleArg {
    Foreground,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceKindArg {
    Service,
    Sandbox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceRestartPolicyArg {
    Never,
    OnFailure,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceHealthPolicyArg {
    None,
    Command,
}

#[derive(Debug, Clone)]
pub struct HostedNodeBindingArg(pub HostedNodeBinding);

impl std::str::FromStr for HostedNodeBindingArg {
    type Err = String;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let (node_name, endpoint_and_token) = input
            .split_once('=')
            .ok_or_else(|| String::from("expected <node>=<endpoint>,<token>"))?;
        let (endpoint, token) = endpoint_and_token
            .rsplit_once(',')
            .ok_or_else(|| String::from("expected <node>=<endpoint>,<token>"))?;
        if node_name.trim().is_empty() || endpoint.trim().is_empty() || token.trim().is_empty() {
            return Err(String::from(
                "node name, endpoint, and token must all be non-empty",
            ));
        }
        Ok(Self(HostedNodeBinding {
            node_name: node_name.trim().to_string(),
            endpoint: endpoint.trim().to_string(),
            token: token.trim().to_string(),
        }))
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => f.write_str("text"),
            Self::Json => f.write_str("json"),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ArtifactCommand {
    #[command(about = "List configured artifacts and the variants available on local disk")]
    List {
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Build a named artifact from the model")]
    Build {
        #[arg(long)]
        artifact: String,
        #[command(flatten)]
        selection: ArtifactSelectionArgs,
    },
    #[command(about = "Validate a named artifact from the model")]
    Validate {
        #[arg(long)]
        artifact: String,
        #[command(flatten)]
        selection: ArtifactSelectionArgs,
    },
    #[command(about = "Publish a selected artifact variant to its configured backend")]
    Push {
        #[arg(long)]
        artifact: String,
        #[command(flatten)]
        selection: ArtifactSelectionArgs,
    },
    #[command(about = "Fetch a selected artifact variant from its configured backend")]
    Pull {
        #[arg(long)]
        artifact: String,
        #[command(flatten)]
        selection: ArtifactSelectionArgs,
    },
}

#[derive(Debug, Subcommand)]
pub enum ClusterCommand {
    #[command(about = "List named local and hosted K3s cluster contracts from the model")]
    List {
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Inspect one named local or hosted K3s cluster contract from the model")]
    Show {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Stage the offline bootstrap kit for one named local cluster")]
    Stage {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Launch and bootstrap one named cluster")]
    Up {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        boot_wait_secs: u64,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Report Port-owned readiness for one named cluster")]
    Status {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Return a usable kubeconfig for one named cluster")]
    Kubeconfig {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(about = "Stop one named cluster and clean up local forwards when applicable")]
    Down {
        #[arg(long)]
        cluster: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        stop_wait_secs: u64,
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Args)]
pub struct ArtifactSelectionArgs {
    #[arg(long, value_enum, default_value_t = ArchitectureArg::Native)]
    architecture: ArchitectureArg,
    #[arg(long, value_enum, default_value_t = SubstrateArg::Firecracker)]
    substrate: SubstrateArg,
    #[arg(long = "protection-mode", value_enum, default_value_t = ProtectionModeArg::Standard)]
    protection_mode: ProtectionModeArg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ArchitectureArg {
    Native,
    X86_64,
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SubstrateArg {
    Firecracker,
    CloudHypervisor,
    Avf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProtectionModeArg {
    Standard,
    Pvm,
}

impl From<ArchitectureArg> for MachineArchitecture {
    fn from(value: ArchitectureArg) -> Self {
        match value {
            ArchitectureArg::Native => Self::Native,
            ArchitectureArg::X86_64 => Self::X86_64,
            ArchitectureArg::Aarch64 => Self::Aarch64,
        }
    }
}

impl From<SubstrateArg> for ExecutionSubstrate {
    fn from(value: SubstrateArg) -> Self {
        match value {
            SubstrateArg::Firecracker => Self::Firecracker,
            SubstrateArg::CloudHypervisor => Self::CloudHypervisor,
            SubstrateArg::Avf => Self::Avf,
        }
    }
}

impl From<ProtectionModeArg> for ProtectionMode {
    fn from(value: ProtectionModeArg) -> Self {
        match value {
            ProtectionModeArg::Standard => Self::Standard,
            ProtectionModeArg::Pvm => Self::Pvm,
        }
    }
}

impl From<ServiceKindArg> for port_runtime::ServiceKind {
    fn from(value: ServiceKindArg) -> Self {
        match value {
            ServiceKindArg::Service => Self::Service,
            ServiceKindArg::Sandbox => Self::Sandbox,
        }
    }
}

impl From<ServiceRestartPolicyArg> for ServiceRestartPolicy {
    fn from(value: ServiceRestartPolicyArg) -> Self {
        match value {
            ServiceRestartPolicyArg::Never => Self::Never,
            ServiceRestartPolicyArg::OnFailure => Self::OnFailure,
            ServiceRestartPolicyArg::Always => Self::Always,
        }
    }
}

impl From<ServiceHealthPolicyArg> for ServiceHealthPolicy {
    fn from(value: ServiceHealthPolicyArg) -> Self {
        match value {
            ServiceHealthPolicyArg::None => Self::None,
            ServiceHealthPolicyArg::Command => Self::Command,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum MachineCommand {
    #[command(about = "Launch a named machine from the model")]
    Launch {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        boot_wait_secs: u64,
    },
    #[command(about = "List Port-managed machines under a runtime root")]
    List {
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(about = "Inspect the runtime status of a named machine")]
    Status {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(
        about = "Inspect runtime ownership, logs, and detached forward state for a named machine"
    )]
    Monitor {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(about = "Inspect hypervisor and detached-forward processes for a named machine")]
    Top {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(about = "Stop a Port-managed machine under a runtime root")]
    Stop {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        wait_secs: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommand {
    #[command(subcommand, about = "Manage machine-bound secret references")]
    Secret(ServiceSecretCommand),
    #[command(about = "Apply a service or sandbox definition through the resolved runtime owner")]
    Apply {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long, value_enum, default_value_t = ServiceKindArg::Service)]
        kind: ServiceKindArg,
        #[arg(long)]
        host_group: Option<String>,
        #[arg(long, value_enum, default_value_t = ServiceRestartPolicyArg::Never)]
        restart: ServiceRestartPolicyArg,
        #[arg(long, value_enum, default_value_t = ServiceHealthPolicyArg::None)]
        health: ServiceHealthPolicyArg,
        #[arg(long = "health-command")]
        health_command: Vec<String>,
        #[arg(long = "secret")]
        secret: Vec<String>,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    #[command(about = "List service and sandbox definitions plus runtime state for a machine")]
    List {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(about = "Inspect one service or sandbox definition and runtime state")]
    Status {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        name: String,
    },
    #[command(about = "Stop one service or sandbox through the resolved runtime owner")]
    Stop {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceSecretCommand {
    #[command(about = "Store a secret reference for one machine runtime")]
    Put {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long)]
        value: String,
    },
    #[command(about = "List stored secret references for one machine runtime")]
    List {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
    },
    #[command(about = "Remove a stored secret reference that is no longer in use")]
    Remove {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum GuestCommand {
    #[command(about = "Run a non-interactive command in the guest")]
    Exec {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    #[command(about = "Copy files between host and guest")]
    Copy {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, value_enum)]
        direction: CopyDirectionArg,
        #[arg(long)]
        source: String,
        #[arg(long)]
        destination: String,
    },
    #[command(about = "Open an interactive PTY-backed session in the guest")]
    Pty {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    #[command(about = "Stream guest logs exposed by the agent")]
    Logs {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long, default_value = "/var/log/port-agent.log")]
        path: String,
        #[arg(long)]
        tail_lines: Option<u32>,
        #[arg(long)]
        follow: bool,
    },
    #[command(about = "Forward TCP or Unix-socket listeners into the guest through the agent")]
    Forward {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        listen: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, value_enum, default_value_t = ForwardLifecycleArg::Foreground)]
        lifecycle: ForwardLifecycleArg,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        stop: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ControlPlaneCommand {
    #[command(
        about = "Serve hosted machine and guest routes over authenticated HTTP and reload durable fleet state"
    )]
    Serve {
        #[arg(long)]
        control_plane: String,
        #[arg(long, default_value = "127.0.0.1:7040")]
        bind: String,
        #[arg(long = "node-binding")]
        node_bindings: Vec<HostedNodeBindingArg>,
    },
    #[command(
        about = "Prepare one hosted node for Firecracker/PVM by attaching a canonical host-kit package through the control plane"
    )]
    PreparePvmNode {
        #[arg(long)]
        control_plane: String,
        #[arg(long)]
        node: String,
        #[arg(long, value_enum, default_value_t = ArchitectureArg::X86_64)]
        architecture: ArchitectureArg,
        #[arg(long, default_value = "operator-prepare")]
        provenance: String,
        #[arg(long)]
        package_name: String,
        #[arg(long)]
        package_version: String,
        #[arg(long)]
        host_kernel_release: String,
        #[arg(long)]
        firecracker_build: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum NodeAgentCommand {
    #[command(
        about = "Serve one hosted node's runtime-root-backed machine and guest routes while refreshing durable registration"
    )]
    Serve {
        #[arg(long)]
        node: String,
        #[arg(long, default_value = "127.0.0.1:9234")]
        bind: String,
        #[arg(long)]
        token: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum InternalCommand {
    #[command(hide = true)]
    ForwardDaemon {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        runtime_root: PathBuf,
        #[arg(long)]
        listen: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        manifest_path: PathBuf,
        #[arg(long)]
        name: String,
    },
    #[command(hide = true)]
    SshMachineLaunch {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        boot_wait_secs: u64,
    },
    #[command(hide = true)]
    SshMachineStatus {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        runtime_root: PathBuf,
    },
    #[command(hide = true)]
    SshMachineStop {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        runtime_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        wait_secs: u64,
    },
}

#[derive(Debug, Serialize)]
struct RenderedDoctorReport {
    host_os: String,
    local_firecracker_supported: bool,
    notes: Vec<String>,
    checks: Vec<RenderedDoctorCheck>,
}

#[derive(Debug, Serialize)]
struct RenderedDoctorCheck {
    name: String,
    ok: bool,
    required: bool,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DetachedForwardManifest {
    name: String,
    machine: String,
    pid: u32,
    listen: String,
    target: String,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
}

#[derive(Debug, Serialize)]
struct RenderedClusterRecord {
    name: String,
    flavor: String,
    provider: String,
    count: u16,
    machine: String,
    version: String,
    args: Vec<String>,
    stage_root: String,
    install_script: String,
    binary: String,
    guest_profile: String,
    required_commands: Vec<String>,
    health_command: Vec<String>,
    kubeconfig_path: String,
    api_forward_target: String,
    boundary: String,
}

#[derive(Debug, Serialize)]
struct RenderedHostedK3sClusterRecord {
    name: String,
    flavor: String,
    provider: String,
    control_plane: String,
    host_group: String,
    control_plane_scheduler: String,
    control_plane_machines: Vec<String>,
    worker_machines: Vec<String>,
    api_endpoint: String,
    version: String,
    server_args: Vec<String>,
    worker_args: Vec<String>,
    boundary: String,
}

#[derive(Debug, Serialize)]
struct RenderedClusterKubeconfig {
    cluster_name: String,
    machine_name: String,
    kubeconfig_path: String,
    kubeconfig_surface: String,
    forward_name: String,
    forward_action: String,
    forward_listen: String,
    forward_target: String,
    boundary: String,
    kubeconfig: String,
}

#[derive(Debug)]
struct EnsuredDetachedForward {
    manifest: DetachedForwardManifest,
    action: &'static str,
}

enum ResolvedClusterKind<'a> {
    Local(&'a port_model::ClusterSpec),
    Hosted(&'a port_model::K3sClusterSpec),
}

fn resolve_cluster_kind<'a>(
    config: &'a PortConfig,
    cluster_name: &str,
) -> Result<ResolvedClusterKind<'a>> {
    match (
        config.clusters.get(cluster_name),
        config.k3s_clusters.get(cluster_name),
    ) {
        (Some(local), None) => Ok(ResolvedClusterKind::Local(local)),
        (None, Some(hosted)) => Ok(ResolvedClusterKind::Hosted(hosted)),
        (Some(_), Some(_)) => bail!(
            "cluster '{}' is ambiguous: it exists in both local `[clusters]` and hosted `[k3s_clusters]` contracts",
            cluster_name
        ),
        (None, None) => bail!("cluster '{}' not found in config", cluster_name),
    }
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Doctor { format } => doctor(format, cli.config.as_deref()),
        Command::Upgrade(command) => upgrade::run(command),
        Command::Artifacts(command) => {
            let config = load_config(cli.config)?;
            run_artifacts(command, &config)
        }
        Command::Cluster(command) => {
            let config_path = cli.config.clone();
            let config = load_config(config_path.clone())?;
            run_cluster(command, config_path.as_deref(), &config)
        }
        Command::Machine(command) => run_machine(command, cli.config),
        Command::Guest(command) => {
            let config_path = cli.config.clone();
            let config = load_config(config_path.clone())?;
            run_guest(command, config_path.as_deref(), &config)
        }
        Command::Service(command) => {
            let config = load_config(cli.config)?;
            run_service(command, &config)
        }
        Command::ControlPlane(command) => {
            let config = load_config(cli.config)?;
            run_control_plane(command, config)
        }
        Command::NodeAgent(command) => {
            let config = load_config(cli.config)?;
            run_node_agent(command, config)
        }
        Command::Internal(command) => match command {
            InternalCommand::ForwardDaemon {
                machine,
                runtime_root,
                listen,
                target,
                manifest_path,
                name,
            } => {
                let config = load_config(cli.config)?;
                run_forward_daemon(
                    &config,
                    &machine,
                    &runtime_root,
                    &listen,
                    &target,
                    &manifest_path,
                    &name,
                )
            }
            InternalCommand::SshMachineLaunch {
                machine,
                runtime_root,
                boot_wait_secs,
            } => {
                let config = load_config_from_stdin()?;
                println!(
                    "{}",
                    serde_json::to_string(&port_runtime::ssh_internal_launch_machine(
                        &config,
                        &LaunchRequest {
                            machine_name: &machine,
                            runtime_root: &runtime_root,
                            boot_wait: Duration::from_secs(boot_wait_secs),
                        },
                    )?)
                    .context("failed to encode ssh launch metadata")?
                );
                Ok(())
            }
            InternalCommand::SshMachineStatus {
                machine,
                runtime_root,
            } => {
                let config = load_config_from_stdin()?;
                println!(
                    "{}",
                    serde_json::to_string(&port_runtime::ssh_internal_machine_status(
                        &config,
                        &runtime_root,
                        &machine,
                    )?)
                    .context("failed to encode ssh machine status")?
                );
                Ok(())
            }
            InternalCommand::SshMachineStop {
                machine,
                runtime_root,
                wait_secs,
            } => {
                let config = load_config_from_stdin()?;
                println!(
                    "{}",
                    serde_json::to_string(&port_runtime::ssh_internal_stop_machine(
                        &config,
                        &runtime_root,
                        &machine,
                        Duration::from_secs(wait_secs),
                    )?)
                    .context("failed to encode ssh stop result")?
                );
                Ok(())
            }
        },
    }
}

fn run_cluster(
    command: ClusterCommand,
    config_path: Option<&Path>,
    config: &PortConfig,
) -> Result<()> {
    match command {
        ClusterCommand::List { format } => {
            let local_clusters = config
                .clusters
                .iter()
                .map(|(name, cluster)| render_cluster_record(name, cluster))
                .collect::<Vec<_>>();
            let hosted_clusters = config
                .k3s_clusters
                .iter()
                .map(|(name, cluster)| render_hosted_k3s_cluster_record(name, cluster))
                .collect::<Vec<_>>();
            match format {
                OutputFormat::Text => {
                    if local_clusters.is_empty() && hosted_clusters.is_empty() {
                        println!("no clusters defined");
                    } else {
                        for cluster in local_clusters {
                            println!(
                                "{}\tflavor={}\tprovider={}\tcount={}\tmachine={}\tversion={}",
                                cluster.name,
                                cluster.flavor,
                                cluster.provider,
                                cluster.count,
                                cluster.machine,
                                cluster.version
                            );
                        }
                        for cluster in hosted_clusters {
                            println!(
                                "{}\tflavor={}\tprovider={}\tcontrol-planes={}\tworkers={}\tapi-endpoint={}\tscheduler={}\tversion={}",
                                cluster.name,
                                cluster.flavor,
                                cluster.provider,
                                cluster.control_plane_machines.len(),
                                cluster.worker_machines.len(),
                                cluster.api_endpoint,
                                cluster.control_plane_scheduler,
                                cluster.version
                            );
                        }
                    }
                    Ok(())
                }
                OutputFormat::Json => {
                    let mut clusters = Vec::new();
                    for cluster in local_clusters {
                        clusters.push(serde_json::json!({
                            "kind": "local",
                            "name": cluster.name,
                            "flavor": cluster.flavor,
                            "provider": cluster.provider,
                            "count": cluster.count,
                            "machine": cluster.machine,
                            "version": cluster.version,
                            "args": cluster.args,
                            "stage_root": cluster.stage_root,
                            "install_script": cluster.install_script,
                            "binary": cluster.binary,
                            "guest_profile": cluster.guest_profile,
                            "required_commands": cluster.required_commands,
                            "health_command": cluster.health_command,
                            "kubeconfig_path": cluster.kubeconfig_path,
                            "api_forward_target": cluster.api_forward_target,
                            "boundary": cluster.boundary,
                        }));
                    }
                    for cluster in hosted_clusters {
                        clusters.push(serde_json::json!({
                            "kind": "hosted-k3s",
                            "name": cluster.name,
                            "flavor": cluster.flavor,
                            "provider": cluster.provider,
                            "control_plane": cluster.control_plane,
                            "host_group": cluster.host_group,
                            "control_plane_scheduler": cluster.control_plane_scheduler,
                            "control_plane_machines": cluster.control_plane_machines,
                            "worker_machines": cluster.worker_machines,
                            "api_endpoint": cluster.api_endpoint,
                            "version": cluster.version,
                            "server_args": cluster.server_args,
                            "worker_args": cluster.worker_args,
                            "boundary": cluster.boundary,
                        }));
                    }
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&clusters)
                            .context("failed to encode cluster list")?
                    );
                    Ok(())
                }
            }
        }
        ClusterCommand::Show { cluster, format } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Local(cluster_record) => {
                let rendered = render_cluster_record(&cluster, cluster_record);
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", rendered.name);
                        println!("flavor: {}", rendered.flavor);
                        println!("provider: {}", rendered.provider);
                        println!("count: {}", rendered.count);
                        println!("machine: {}", rendered.machine);
                        println!("version: {}", rendered.version);
                        println!(
                            "args: {}",
                            if rendered.args.is_empty() {
                                String::from("none")
                            } else {
                                rendered.args.join(" ")
                            }
                        );
                        println!("stage root: {}", rendered.stage_root);
                        println!("install script: {}", rendered.install_script);
                        println!("binary: {}", rendered.binary);
                        println!("guest profile: {}", rendered.guest_profile);
                        println!(
                            "required commands: {}",
                            rendered.required_commands.join(" ")
                        );
                        println!(
                            "health command: {}",
                            render_shell_command(&rendered.health_command)
                        );
                        println!("kubeconfig path: {}", rendered.kubeconfig_path);
                        println!("api forward target: {}", rendered.api_forward_target);
                        println!("boundary: {}", rendered.boundary);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&rendered)
                                .context("failed to encode cluster record")?
                        );
                        Ok(())
                    }
                }
            }
            ResolvedClusterKind::Hosted(cluster_record) => {
                let rendered = render_hosted_k3s_cluster_record(&cluster, cluster_record);
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", rendered.name);
                        println!("flavor: {}", rendered.flavor);
                        println!("provider: {}", rendered.provider);
                        println!("control plane: {}", rendered.control_plane);
                        println!("host group: {}", rendered.host_group);
                        println!(
                            "control-plane scheduler: {}",
                            rendered.control_plane_scheduler
                        );
                        println!(
                            "control-plane machines: {}",
                            rendered.control_plane_machines.join(" ")
                        );
                        println!(
                            "worker machines: {}",
                            if rendered.worker_machines.is_empty() {
                                String::from("none")
                            } else {
                                rendered.worker_machines.join(" ")
                            }
                        );
                        println!("api endpoint: {}", rendered.api_endpoint);
                        println!("version: {}", rendered.version);
                        println!(
                            "server args: {}",
                            if rendered.server_args.is_empty() {
                                String::from("none")
                            } else {
                                rendered.server_args.join(" ")
                            }
                        );
                        println!(
                            "worker args: {}",
                            if rendered.worker_args.is_empty() {
                                String::from("none")
                            } else {
                                rendered.worker_args.join(" ")
                            }
                        );
                        println!("boundary: {}", rendered.boundary);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&rendered)
                                .context("failed to encode hosted cluster record")?
                        );
                        Ok(())
                    }
                }
            }
        },
        ClusterCommand::Stage {
            cluster,
            runtime_root,
            format,
        } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Hosted(_) => bail!(
                "cluster '{}' is a hosted K3s microVM contract; `port cluster stage` only applies to the local offline bootstrap slice",
                cluster
            ),
            ResolvedClusterKind::Local(_) => {
                let result = port_runtime::stage_local_cluster_bootstrap(
                    config,
                    ClusterStageRequest {
                        cluster_name: &cluster,
                        runtime_root: &runtime_root,
                    },
                )?;
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("machine: {}", result.machine_name);
                        println!("stage root: {}", result.stage_root.display());
                        println!("guest profile: {}", result.guest_profile);
                        println!("required commands: {}", result.required_commands.join(" "));
                        for staged in &result.staged_files {
                            println!(
                                "staged file: {} -> {} ({} bytes)",
                                staged.source.display(),
                                staged.destination.display(),
                                staged.bytes_copied
                            );
                        }
                        println!(
                            "preflight command: {}",
                            render_shell_command(&result.preflight_command)
                        );
                        println!("preflight output:");
                        print!("{}", result.preflight_stdout);
                        if !result.preflight_stdout.ends_with('\n') {
                            println!();
                        }
                        println!(
                            "install command: {}",
                            render_shell_command(&result.install_command)
                        );
                        println!("install output:");
                        print!("{}", result.install_stdout);
                        if !result.install_stdout.ends_with('\n') {
                            println!();
                        }
                        println!("installed binary: {}", result.installed_binary.display());
                        println!("installed kubectl: {}", result.installed_kubectl.display());
                        println!("boundary: {}", result.boundary);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode cluster stage result")?
                        );
                        Ok(())
                    }
                }
            }
        },
        ClusterCommand::Up {
            cluster,
            runtime_root,
            boot_wait_secs,
            format,
        } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Local(_) => {
                let result = port_runtime::up_local_cluster(
                    config,
                    ClusterUpRequest {
                        cluster_name: &cluster,
                        runtime_root: &runtime_root,
                        boot_wait: Duration::from_secs(boot_wait_secs),
                    },
                )?;

                if let Some(cluster_record) = config.clusters.get(&cluster) {
                    for fwd in &cluster_record.lifecycle.forwards {
                        let fwd_name = service_forward_name(&cluster, &fwd.name);
                        let _ = ensure_detached_forward(
                            config_path,
                            config,
                            &result.machine_name,
                            &runtime_root,
                            &fwd.target,
                            &fwd_name,
                        );
                    }
                }

                port_runtime::chown_runtime_to_sudo_caller(
                    &runtime_root.join(&result.machine_name),
                )
                .context("failed to transfer runtime ownership to invoking user")?;

                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("machine: {}", result.machine_name);
                        println!("launch action: {}", result.launch_action);
                        if let Some(launch) = &result.launch {
                            println!("machine pid: {}", launch.pid);
                            println!("runtime dir: {}", launch.runtime_dir.display());
                        }
                        println!("stage root: {}", result.stage.stage_root.display());
                        println!("guest profile: {}", result.stage.guest_profile);
                        print_cluster_status_report(&result.status);
                        println!("boundary: {}", result.boundary);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode cluster up result")?
                        );
                        Ok(())
                    }
                }
            }
            ResolvedClusterKind::Hosted(_) => {
                let result =
                    port_runtime::bootstrap_hosted_k3s_cluster(config, &runtime_root, &cluster)?;
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("control plane: {}", result.control_plane);
                        println!("host group: {}", result.host_group);
                        println!("api endpoint: {}", result.api_endpoint);
                        println!(
                            "stable-endpoint posture: {}",
                            result.stable_endpoint_posture
                        );
                        println!("stable-endpoint detail: {}", result.stable_endpoint_detail);
                        println!(
                            "control-plane machines: {}",
                            result.server_machines.join(" ")
                        );
                        println!(
                            "worker machines: {}",
                            if result.worker_machines.is_empty() {
                                String::from("none")
                            } else {
                                result.worker_machines.join(" ")
                            }
                        );
                        for launch in &result.server_launches {
                            println!(
                                "control-plane launch: {} pid={} runtime-dir={}",
                                launch.machine_name,
                                launch.pid,
                                launch.runtime_dir.display()
                            );
                        }
                        for launch in &result.worker_launches {
                            println!(
                                "worker launch: {} pid={} runtime-dir={}",
                                launch.machine_name,
                                launch.pid,
                                launch.runtime_dir.display()
                            );
                        }
                        print_boundary_notes(&result.boundary_notes);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode hosted cluster up result")?
                        );
                        Ok(())
                    }
                }
            }
        },
        ClusterCommand::Status {
            cluster,
            runtime_root,
            format,
        } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Local(_) => {
                let result = port_runtime::local_cluster_status(
                    config,
                    ClusterStatusRequest {
                        cluster_name: &cluster,
                        runtime_root: &runtime_root,
                    },
                )?;
                match format {
                    OutputFormat::Text => {
                        print_cluster_status_report(&result);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode cluster status result")?
                        );
                        Ok(())
                    }
                }
            }
            ResolvedClusterKind::Hosted(_) => {
                let result =
                    port_runtime::hosted_k3s_cluster_access(config, &runtime_root, &cluster)?;
                match format {
                    OutputFormat::Text => {
                        print_hosted_k3s_cluster_access_report(&result);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode hosted cluster status result")?
                        );
                        Ok(())
                    }
                }
            }
        },
        ClusterCommand::Kubeconfig {
            cluster,
            runtime_root,
            format,
        } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Local(_) => {
                let result = port_runtime::local_cluster_kubeconfig(
                    config,
                    ClusterStatusRequest {
                        cluster_name: &cluster,
                        runtime_root: &runtime_root,
                    },
                )?;
                let forward_name = cluster_forward_name(&cluster);
                let forward = ensure_detached_forward(
                    config_path,
                    config,
                    &result.machine_name,
                    &runtime_root,
                    &result.api_forward_target,
                    &forward_name,
                )?;
                let rewritten =
                    rewrite_kubeconfig_server(&result.kubeconfig, &forward.manifest.listen)?;
                let rendered = RenderedClusterKubeconfig {
                    cluster_name: result.cluster_name,
                    machine_name: result.machine_name,
                    kubeconfig_path: result.kubeconfig_path.display().to_string(),
                    kubeconfig_surface: result.kubeconfig_surface,
                    forward_name: forward.manifest.name,
                    forward_action: forward.action.to_string(),
                    forward_listen: forward.manifest.listen,
                    forward_target: forward.manifest.target,
                    boundary: result.boundary,
                    kubeconfig: rewritten,
                };
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", rendered.cluster_name);
                        println!("machine: {}", rendered.machine_name);
                        println!("kubeconfig path: {}", rendered.kubeconfig_path);
                        println!("kubeconfig surface: {}", rendered.kubeconfig_surface);
                        println!("forward name: {}", rendered.forward_name);
                        println!("forward action: {}", rendered.forward_action);
                        println!("forward listen: {}", rendered.forward_listen);
                        println!("forward target: {}", rendered.forward_target);
                        println!("boundary: {}", rendered.boundary);
                        println!("kubeconfig:");
                        print!("{}", rendered.kubeconfig);
                        if !rendered.kubeconfig.ends_with('\n') {
                            println!();
                        }
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&rendered)
                                .context("failed to encode cluster kubeconfig result")?
                        );
                        Ok(())
                    }
                }
            }
            ResolvedClusterKind::Hosted(_) => {
                let result =
                    port_runtime::hosted_k3s_cluster_access(config, &runtime_root, &cluster)?;
                let rewritten =
                    rewrite_kubeconfig_server(&result.kubeconfig, &result.api_endpoint)?;
                let rendered = serde_json::json!({
                    "cluster_name": result.cluster_name,
                    "control_plane": result.control_plane,
                    "host_group": result.host_group,
                    "server_machines": result.server_machines,
                    "worker_machines": result.worker_machines,
                    "api_endpoint": result.api_endpoint,
                    "stable_endpoint_posture": result.stable_endpoint_posture,
                    "stable_endpoint_detail": result.stable_endpoint_detail,
                    "kubeconfig_surface": result.kubeconfig_surface,
                    "visibility_surface": result.visibility_surface,
                    "boundary_notes": result.boundary_notes,
                    "kubeconfig": rewritten,
                });
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("control plane: {}", result.control_plane);
                        println!("host group: {}", result.host_group);
                        println!("api endpoint: {}", result.api_endpoint);
                        println!(
                            "stable-endpoint posture: {}",
                            result.stable_endpoint_posture
                        );
                        println!("stable-endpoint detail: {}", result.stable_endpoint_detail);
                        println!(
                            "control-plane machines: {}",
                            result.server_machines.join(" ")
                        );
                        println!(
                            "worker machines: {}",
                            if result.worker_machines.is_empty() {
                                String::from("none")
                            } else {
                                result.worker_machines.join(" ")
                            }
                        );
                        println!("kubeconfig surface: {}", result.kubeconfig_surface);
                        print_boundary_notes(&result.boundary_notes);
                        println!("kubeconfig:");
                        print!("{}", rewritten);
                        if !rewritten.ends_with('\n') {
                            println!();
                        }
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&rendered)
                                .context("failed to encode hosted cluster kubeconfig result")?
                        );
                        Ok(())
                    }
                }
            }
        },
        ClusterCommand::Down {
            cluster,
            runtime_root,
            stop_wait_secs,
            format,
        } => match resolve_cluster_kind(config, &cluster)? {
            ResolvedClusterKind::Local(cluster_record) => {
                for fwd in &cluster_record.lifecycle.forwards {
                    let fwd_name = service_forward_name(&cluster, &fwd.name);
                    let _ = stop_detached_forward_if_present(
                        config,
                        &cluster_record.machine,
                        &runtime_root,
                        &fwd_name,
                    );
                }
                let forward_name = cluster_forward_name(&cluster);
                let forward_cleanup = stop_detached_forward_if_present(
                    config,
                    &cluster_record.machine,
                    &runtime_root,
                    &forward_name,
                )?;
                let result = port_runtime::down_local_cluster(
                    config,
                    ClusterDownRequest {
                        cluster_name: &cluster,
                        runtime_root: &runtime_root,
                        stop_wait: Duration::from_secs(stop_wait_secs),
                    },
                )?;
                let forward_cleanup = forward_cleanup.map_or_else(
                    || format!("{forward_name} not-present"),
                    |manifest| format!("{} stopped", manifest.name),
                );
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("machine: {}", result.machine_name);
                        println!("forward cleanup: {}", forward_cleanup);
                        println!("previous state: {}", result.stop.previous_state);
                        println!("current state: {}", result.stop.current_state);
                        println!(
                            "machine pid: {}",
                            result
                                .stop
                                .pid
                                .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
                        );
                        println!("runtime dir: {}", result.stop.runtime_dir.display());
                        println!("detail: {}", result.stop.detail);
                        println!("boundary: {}", result.boundary);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "cluster_name": result.cluster_name,
                                "machine_name": result.machine_name,
                                "forward_cleanup": forward_cleanup,
                                "stop": result.stop,
                                "boundary": result.boundary,
                            }))
                            .context("failed to encode cluster down result")?
                        );
                        Ok(())
                    }
                }
            }
            ResolvedClusterKind::Hosted(_) => {
                let result = port_runtime::down_hosted_k3s_cluster(
                    config,
                    &runtime_root,
                    &cluster,
                    Duration::from_secs(stop_wait_secs),
                )?;
                match format {
                    OutputFormat::Text => {
                        println!("cluster: {}", result.cluster_name);
                        println!("control plane: {}", result.control_plane);
                        println!("host group: {}", result.host_group);
                        println!("api endpoint: {}", result.api_endpoint);
                        println!(
                            "control-plane machines: {}",
                            result.server_machines.join(" ")
                        );
                        println!(
                            "worker machines: {}",
                            if result.worker_machines.is_empty() {
                                String::from("none")
                            } else {
                                result.worker_machines.join(" ")
                            }
                        );
                        for stop in &result.worker_stops {
                            println!(
                                "worker stop: {} previous={} current={} pid={}",
                                stop.machine_name,
                                stop.previous_state,
                                stop.current_state,
                                stop.pid
                                    .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
                            );
                        }
                        for stop in &result.server_stops {
                            println!(
                                "control-plane stop: {} previous={} current={} pid={}",
                                stop.machine_name,
                                stop.previous_state,
                                stop.current_state,
                                stop.pid
                                    .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
                            );
                        }
                        print_boundary_notes(&result.boundary_notes);
                        Ok(())
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result)
                                .context("failed to encode hosted cluster down result")?
                        );
                        Ok(())
                    }
                }
            }
        },
    }
}

fn render_cluster_record(name: &str, cluster: &port_model::ClusterSpec) -> RenderedClusterRecord {
    RenderedClusterRecord {
        name: name.to_string(),
        flavor: cluster.flavor.to_string(),
        provider: cluster.provider.to_string(),
        count: cluster.count,
        machine: cluster.machine.clone(),
        version: cluster.version.clone(),
        args: cluster.args.clone(),
        stage_root: cluster.bootstrap.stage_root.display().to_string(),
        install_script: cluster.bootstrap.install_script.display().to_string(),
        binary: cluster.bootstrap.binary.display().to_string(),
        guest_profile: cluster.bootstrap.guest_profile.name.clone(),
        required_commands: cluster.bootstrap.guest_profile.required_commands.clone(),
        health_command: cluster.lifecycle.health_command.clone(),
        kubeconfig_path: cluster.lifecycle.kubeconfig_path.display().to_string(),
        api_forward_target: cluster.lifecycle.api_forward_target.clone(),
        boundary: String::from(
            "single-node local K3s only in this slice; hosted microVM-backed K3s is a separate contract and local multi-node expansion remains follow-on work",
        ),
    }
}

fn render_hosted_k3s_cluster_record(
    name: &str,
    cluster: &port_model::K3sClusterSpec,
) -> RenderedHostedK3sClusterRecord {
    RenderedHostedK3sClusterRecord {
        name: name.to_string(),
        flavor: String::from("k3s"),
        provider: String::from("hosted"),
        control_plane: cluster.control_plane.clone(),
        host_group: cluster.host_group.clone(),
        control_plane_scheduler: render_scheduler_policy(cluster.control_plane_scheduler),
        control_plane_machines: cluster.server_machines.clone(),
        worker_machines: cluster.worker_machines.clone(),
        api_endpoint: cluster.api_endpoint.clone(),
        version: cluster.version.clone(),
        server_args: cluster.server_args.clone(),
        worker_args: cluster.worker_args.clone(),
        boundary: String::from(
            "hosted Firecracker microVM K3s; real HA depends on at least three control-plane microVMs, a stable HTTPS api endpoint, and distinct execution hosts behind that endpoint",
        ),
    }
}

fn print_cluster_status_report(report: &port_runtime::ClusterStatusReport) {
    println!("cluster: {}", report.cluster_name);
    println!("machine: {}", report.machine_name);
    println!("runtime dir: {}", report.runtime_dir.display());
    println!("machine state: {}", report.machine_state);
    println!(
        "pid: {}",
        report
            .pid
            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
    );
    println!("readiness: {}", report.readiness);
    println!(
        "health command: {}",
        render_shell_command(&report.health_command)
    );
    if report.health_output.is_empty() {
        println!("health output: (none)");
    } else {
        println!("health output:");
        print!("{}", report.health_output);
        if !report.health_output.ends_with('\n') {
            println!();
        }
    }
    println!("kubeconfig path: {}", report.kubeconfig_path.display());
    println!("kubeconfig available: {}", report.kubeconfig_available);
    println!("api forward target: {}", report.api_forward_target);
    println!("kubeconfig surface: {}", report.kubeconfig_surface);
    println!("boundary: {}", report.boundary);
    println!("detail: {}", report.detail);
}

fn print_boundary_notes(notes: &[String]) {
    for note in notes {
        println!("boundary: {}", note);
    }
}

fn print_hosted_k3s_cluster_access_report(report: &port_runtime::HostedK3sClusterAccessReport) {
    println!("cluster: {}", report.cluster_name);
    println!("control plane: {}", report.control_plane);
    println!("host group: {}", report.host_group);
    println!(
        "control-plane machines: {}",
        report.server_machines.join(" ")
    );
    println!(
        "worker machines: {}",
        if report.worker_machines.is_empty() {
            String::from("none")
        } else {
            report.worker_machines.join(" ")
        }
    );
    println!("api endpoint: {}", report.api_endpoint);
    println!(
        "stable-endpoint posture: {}",
        report.stable_endpoint_posture
    );
    println!("stable-endpoint detail: {}", report.stable_endpoint_detail);
    println!("real-ha status: {}", report.ha_status);
    println!("real-ha detail: {}", report.ha_status_detail);
    for placement in &report.control_plane_placements {
        println!(
            "control-plane placement: {} -> {}",
            placement.machine_name,
            placement.node_name.as_deref().unwrap_or("(unresolved)")
        );
    }
    println!("kubeconfig surface: {}", report.kubeconfig_surface);
    println!("visibility surface: {}", report.visibility_surface);
    if report.visibility_output.is_empty() {
        println!("visibility output: (none)");
    } else {
        println!("visibility output:");
        print!("{}", report.visibility_output);
        if !report.visibility_output.ends_with('\n') {
            println!();
        }
    }
    for machine in &report.machine_access {
        let machine_name = machine.route.machine_name.as_deref().unwrap_or("(unknown)");
        let node_name = machine.route.node_name.as_deref().unwrap_or("(unresolved)");
        println!(
            "route: role={} machine={} node={}",
            machine.role, machine_name, node_name
        );
        println!("route detail: {}", machine.detail);
    }
    print_boundary_notes(&report.boundary_notes);
}

fn render_shell_command(command: &[String]) -> String {
    command
        .iter()
        .map(|part| {
            if part
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "/._-:=+".contains(character))
            {
                part.clone()
            } else {
                format!("'{}'", part.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_control_plane(command: ControlPlaneCommand, config: PortConfig) -> Result<()> {
    match command {
        ControlPlaneCommand::Serve {
            control_plane,
            bind,
            node_bindings,
        } => port_runtime::serve_control_plane(
            config,
            ControlPlaneServeRequest {
                control_plane,
                bind,
                node_bindings: node_bindings.into_iter().map(|binding| binding.0).collect(),
            },
        ),
        ControlPlaneCommand::PreparePvmNode {
            control_plane,
            node,
            architecture,
            provenance,
            package_name,
            package_version,
            host_kernel_release,
            firecracker_build,
        } => {
            let record = port_runtime::prepare_hosted_pvm_node(
                &config,
                HostedPvmNodePrepareRequest {
                    control_plane,
                    node_name: node.clone(),
                    architecture: architecture.into(),
                    provenance,
                    package: PvmHostKitPackage {
                        name: package_name,
                        version: package_version,
                        host_kernel_release,
                        firecracker_build,
                    },
                },
            )?;
            let prepared = record
                .pvm_host_kit_packages
                .iter()
                .map(|attachment| {
                    format!(
                        "{:?}/{}@{}",
                        attachment.architecture,
                        attachment.package.name,
                        attachment.package.version
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!("prepared hosted pvm node: {node} ({prepared})");
            Ok(())
        }
    }
}

fn run_node_agent(command: NodeAgentCommand, config: PortConfig) -> Result<()> {
    match command {
        NodeAgentCommand::Serve { node, bind, token } => port_runtime::serve_node_agent(
            config,
            NodeAgentServeRequest {
                node_name: node,
                bind,
                token,
            },
        ),
    }
}

fn doctor(format: OutputFormat, config_path: Option<&std::path::Path>) -> Result<()> {
    let config = load_config_if_present(config_path)?;
    let report = port_runtime::collect_doctor_report(config.as_ref());
    let rendered = RenderedDoctorReport::from(report);

    match format {
        OutputFormat::Text => {
            println!("host_os: {}", rendered.host_os);
            println!(
                "local_firecracker_supported: {}",
                rendered.local_firecracker_supported
            );
            for check in rendered.checks {
                let status = if check.ok { "ok" } else { "fail" };
                println!(
                    "check[{status}]: {} (required={}) - {}",
                    check.name, check.required, check.detail
                );
            }
            for note in rendered.notes {
                println!("note: {note}");
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&rendered)
                    .context("failed to encode doctor report")?
            );
        }
    }

    Ok(())
}

fn run_artifacts(command: ArtifactCommand, config: &PortConfig) -> Result<()> {
    match command {
        ArtifactCommand::List { format } => {
            let inventory = port_runtime::list_artifacts(config);
            if format == OutputFormat::Text {
                if inventory.is_empty() {
                    println!("no artifacts defined");
                } else {
                    for (index, artifact) in inventory.iter().enumerate() {
                        if index > 0 {
                            println!();
                        }
                        println!("artifact: {}", artifact.name);
                        println!("kind: {}", render_artifact_kind(artifact.kind));
                        println!("reference: {}", artifact.reference);
                        println!("build: {}", artifact.build_command);
                        println!("validate: {}", artifact.validate_command);
                        for variant in &artifact.variants {
                            println!(
                                "variant: {}\tavailability={}\tlocal={}\tcache={}\tpath={}\tcache_path={}",
                                render_selector(variant.selector),
                                variant.availability,
                                variant.local_present,
                                variant.cache_present,
                                variant.path.display(),
                                variant.cache_path.display()
                            );
                        }
                    }
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&inventory)
                        .context("failed to encode artifact inventory")?
                );
            }
        }
        ArtifactCommand::Build {
            artifact,
            selection,
        } => {
            let metadata = port_runtime::build_artifact(
                config,
                ArtifactRequest {
                    name: &artifact,
                    architecture: selection.architecture.into(),
                    substrate: selection.substrate.into(),
                    protection_mode: selection.protection_mode.into(),
                },
            )?;
            println!(
                "built {} artifact '{}' as {} for {} at {}",
                render_artifact_kind(metadata.kind),
                metadata.name,
                metadata.reference,
                render_selector(metadata.selector),
                metadata.path.display()
            );
        }
        ArtifactCommand::Validate {
            artifact,
            selection,
        } => {
            let metadata = port_runtime::validate_artifact(
                config,
                ArtifactRequest {
                    name: &artifact,
                    architecture: selection.architecture.into(),
                    substrate: selection.substrate.into(),
                    protection_mode: selection.protection_mode.into(),
                },
            )?;
            println!(
                "validated {} artifact '{}' as {} for {} at {}",
                render_artifact_kind(metadata.kind),
                metadata.name,
                metadata.reference,
                render_selector(metadata.selector),
                metadata.path.display()
            );
        }
        ArtifactCommand::Push {
            artifact,
            selection,
        } => {
            let transfer = port_runtime::push_artifact(
                config,
                ArtifactRequest {
                    name: &artifact,
                    architecture: selection.architecture.into(),
                    substrate: selection.substrate.into(),
                    protection_mode: selection.protection_mode.into(),
                },
            )?;
            println!(
                "pushed {} artifact '{}' as {} for {}",
                render_artifact_kind(transfer.artifact.kind),
                transfer.artifact.name,
                transfer.artifact.reference,
                render_selector(transfer.artifact.selector)
            );
            println!("backend: {}", transfer.backend_detail);
            println!("local path: {}", transfer.artifact.path.display());
            println!("store path: {}", transfer.store_path.display());
            println!("cache path: {}", transfer.artifact.cache_path.display());
            println!("bytes: {}", transfer.bytes_copied);
        }
        ArtifactCommand::Pull {
            artifact,
            selection,
        } => {
            let transfer = port_runtime::pull_artifact(
                config,
                ArtifactRequest {
                    name: &artifact,
                    architecture: selection.architecture.into(),
                    substrate: selection.substrate.into(),
                    protection_mode: selection.protection_mode.into(),
                },
            )?;
            println!(
                "pulled {} artifact '{}' as {} for {}",
                render_artifact_kind(transfer.artifact.kind),
                transfer.artifact.name,
                transfer.artifact.reference,
                render_selector(transfer.artifact.selector)
            );
            println!("backend: {}", transfer.backend_detail);
            println!("store path: {}", transfer.store_path.display());
            println!("cache path: {}", transfer.artifact.cache_path.display());
            println!("local path: {}", transfer.artifact.path.display());
            println!("bytes: {}", transfer.bytes_copied);
        }
    }

    Ok(())
}

fn render_artifact_kind(kind: port_model::ArtifactKind) -> &'static str {
    match kind {
        port_model::ArtifactKind::Kernel => "kernel",
        port_model::ArtifactKind::GuestImage => "guest-image",
    }
}

fn render_selector(selector: port_model::ArtifactSelector) -> String {
    format!(
        "{}/{}/{}",
        match selector.architecture {
            MachineArchitecture::Native => "native",
            MachineArchitecture::X86_64 => "x86_64",
            MachineArchitecture::Aarch64 => "aarch64",
        },
        match selector.substrate {
            ExecutionSubstrate::Firecracker => "firecracker",
            ExecutionSubstrate::CloudHypervisor => "cloud-hypervisor",
            ExecutionSubstrate::Avf => "avf",
        },
        match selector.protection_mode {
            ProtectionMode::Standard => "standard",
            ProtectionMode::Pvm => "pvm",
        }
    )
}

fn run_machine(command: MachineCommand, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config(config_path)?;
    match command {
        MachineCommand::Launch {
            machine,
            runtime_root,
            boot_wait_secs,
        } => {
            let ssh_context = ssh_machine_route_context(&config, &machine)?;
            let control = machine_control_contract(&config, &machine)?;
            let metadata = port_runtime::launch_local_machine(
                &config,
                &LaunchRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    boot_wait: Duration::from_secs(boot_wait_secs),
                },
            )?;
            println!("launched machine: {}", metadata.machine_name);
            println!("pid: {}", metadata.pid);
            println!("runtime dir: {}", metadata.runtime_dir.display());
            println!(
                "hypervisor binary: {}",
                metadata.firecracker_binary.display()
            );
            println!("config path: {}", metadata.config_path.display());
            println!("hypervisor log: {}", metadata.log_path.display());
            println!("console stdout: {}", metadata.stdout_path.display());
            println!("console stderr: {}", metadata.stderr_path.display());
            println!("manifest: {}", metadata.manifest_path.display());
            output_runtime_class(&metadata.runtime_class);
            if !metadata.attached_volumes.is_empty() {
                println!("inventory owner: {}", control.inventory_owner);
                println!("lifecycle owner: {}", control.lifecycle_owner);
                print!("{}", format_attached_volumes(&metadata.attached_volumes));
            }
            if let Some(context) = ssh_context.as_ref() {
                print_ssh_machine_route_context(context, "launch route");
            }
        }
        MachineCommand::List { runtime_root } => {
            let machines = port_runtime::list_machines(&config, &runtime_root)?;
            if machines.is_empty() {
                println!(
                    "no Port-managed machines found under runtime root '{}'",
                    runtime_root.display()
                );
            } else {
                for machine in machines {
                    println!("machine: {}", machine.machine_name);
                    println!("state: {}", machine.state);
                    println!(
                        "pid: {}",
                        machine
                            .pid
                            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
                    );
                    println!("inventory scope: {}", machine.control.inventory_scope);
                    println!("inventory owner: {}", machine.control.inventory_owner);
                    println!("lifecycle owner: {}", machine.control.lifecycle_owner);
                    println!("status source: {}", machine.control.status_source);
                    println!("status route: {}", machine.control.status_route);
                    output_runtime_class(&machine.runtime_class);
                    println!("runtime dir: {}", machine.runtime_dir.display());
                    print_hosted_fleet_nodes(&machine.hosted_fleet_nodes);
                    println!("detail: {}", machine.detail);
                    println!();
                }
            }
        }
        MachineCommand::Status {
            machine,
            runtime_root,
        } => {
            let ssh_context = ssh_machine_route_context(&config, &machine)?;
            let status = port_runtime::machine_status(&config, &runtime_root, &machine)?;
            if let Some(context) = ssh_context.as_ref() {
                print_ssh_machine_route_context(context, "status route");
            }
            print!("{}", format_machine_status(&status));
        }
        MachineCommand::Monitor {
            machine,
            runtime_root,
        } => {
            let report = port_runtime::machine_monitor(&config, &runtime_root, &machine)?;
            print_machine_monitor(&report);
        }
        MachineCommand::Top {
            machine,
            runtime_root,
        } => {
            let report = port_runtime::machine_top(&config, &runtime_root, &machine)?;
            print_machine_top(&report);
        }
        MachineCommand::Stop {
            machine,
            runtime_root,
            wait_secs,
        } => {
            let ssh_context = ssh_machine_route_context(&config, &machine)?;
            let result = port_runtime::stop_machine(
                &config,
                &runtime_root,
                &machine,
                Duration::from_secs(wait_secs),
            )?;
            println!("machine: {}", result.machine_name);
            if let Some(context) = ssh_context.as_ref() {
                print_ssh_machine_route_context(context, "stop route");
            }
            println!("previous state: {}", result.previous_state);
            println!("current state: {}", result.current_state);
            println!(
                "pid: {}",
                result
                    .pid
                    .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
            );
            println!("inventory scope: {}", result.control.inventory_scope);
            if !result.attached_volumes.is_empty() {
                println!("inventory owner: {}", result.control.inventory_owner);
            }
            println!("lifecycle owner: {}", result.control.lifecycle_owner);
            println!("status source: {}", result.control.status_source);
            println!("stop route: {}", result.control.stop_route);
            println!("runtime dir: {}", result.runtime_dir.display());
            output_runtime_class(&result.runtime_class);
            if !result.attached_volumes.is_empty() {
                print!("{}", format_attached_volumes(&result.attached_volumes));
            }
            println!("detail: {}", result.detail);
        }
    }

    Ok(())
}

fn machine_control_contract(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineControlContract> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{}'", machine_name))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    Ok(MachineControlContract::for_connection(&host.connection))
}

fn print_hosted_fleet_nodes(nodes: &[port_runtime::HostedFleetNodeStatus]) {
    let rendered = format_hosted_fleet_nodes(nodes);
    if rendered.is_empty() {
        return;
    }
    print!("{rendered}");
}

fn output_runtime_class(runtime_class: &Option<MachineRuntimeClassSpec>) {
    let rendered = format_runtime_class(runtime_class);
    if rendered.is_empty() {
        return;
    }
    print!("{rendered}");
}

fn format_machine_status(status: &port_runtime::MachineStatus) -> String {
    let mut output = String::new();
    writeln!(&mut output, "machine: {}", status.machine_name).expect("write should succeed");
    writeln!(&mut output, "state: {}", status.state).expect("write should succeed");
    writeln!(
        &mut output,
        "pid: {}",
        status
            .pid
            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
    )
    .expect("write should succeed");
    writeln!(
        &mut output,
        "inventory scope: {}",
        status.control.inventory_scope
    )
    .expect("write should succeed");
    writeln!(
        &mut output,
        "inventory owner: {}",
        status.control.inventory_owner
    )
    .expect("write should succeed");
    writeln!(
        &mut output,
        "lifecycle owner: {}",
        status.control.lifecycle_owner
    )
    .expect("write should succeed");
    writeln!(&mut output, "guest broker: {}", status.control.guest_broker)
        .expect("write should succeed");
    writeln!(
        &mut output,
        "status source: {}",
        status.control.status_source
    )
    .expect("write should succeed");
    writeln!(&mut output, "launch route: {}", status.control.launch_route)
        .expect("write should succeed");
    writeln!(
        &mut output,
        "inventory route: {}",
        status.control.inventory_route
    )
    .expect("write should succeed");
    writeln!(&mut output, "status route: {}", status.control.status_route)
        .expect("write should succeed");
    writeln!(&mut output, "stop route: {}", status.control.stop_route)
        .expect("write should succeed");
    output.push_str(&format_runtime_class(&status.runtime_class));
    output.push_str(&format_attached_volumes(&status.attached_volumes));
    writeln!(&mut output, "guest route: {}", status.control.guest_route)
        .expect("write should succeed");
    writeln!(&mut output, "runtime dir: {}", status.runtime_dir.display())
        .expect("write should succeed");
    writeln!(&mut output, "config path: {}", status.config_path.display())
        .expect("write should succeed");
    writeln!(&mut output, "manifest: {}", status.manifest_path.display())
        .expect("write should succeed");
    writeln!(&mut output, "pid file: {}", status.pid_path.display()).expect("write should succeed");
    writeln!(
        &mut output,
        "hypervisor log: {}",
        status.firecracker_log.display()
    )
    .expect("write should succeed");
    writeln!(
        &mut output,
        "console stdout: {}",
        status.stdout_log.display()
    )
    .expect("write should succeed");
    writeln!(
        &mut output,
        "console stderr: {}",
        status.stderr_log.display()
    )
    .expect("write should succeed");
    output.push_str(&format_hosted_fleet_nodes(&status.hosted_fleet_nodes));
    writeln!(&mut output, "detail: {}", status.detail).expect("write should succeed");
    output
}

fn format_runtime_class(runtime_class: &Option<MachineRuntimeClassSpec>) -> String {
    let Some(runtime_class) = runtime_class else {
        return String::new();
    };

    let mut output = String::new();
    writeln!(&mut output, "runtime class: {}", runtime_class.kind).expect("write should succeed");
    writeln!(&mut output, "trust posture: {}", runtime_class.trust).expect("write should succeed");
    writeln!(
        &mut output,
        "state isolation: {}",
        runtime_class.state_isolation
    )
    .expect("write should succeed");
    if let Some(workspace) = &runtime_class.workspace {
        writeln!(&mut output, "workspace: {}", workspace.workspace).expect("write should succeed");
        writeln!(&mut output, "workspace lane: {}", workspace.lane).expect("write should succeed");
    }
    for input in &runtime_class.declared_inputs {
        writeln!(&mut output, "declared input: {}", input).expect("write should succeed");
    }
    for root in &runtime_class.writable_roots {
        writeln!(&mut output, "writable root: {}", root).expect("write should succeed");
    }
    output
}

fn format_attached_volumes(volumes: &[MachineVolumeSpec]) -> String {
    let mut output = String::new();
    for volume in volumes {
        writeln!(&mut output, "attached volume: {}", volume.name).expect("write should succeed");
        writeln!(
            &mut output,
            "backend: {}",
            match volume.backend {
                MachineVolumeBackend::HostFile => "host-file",
            }
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "persistence: {}",
            match volume.persistence {
                MachineVolumePersistence::Persistent => "persistent",
            }
        )
        .expect("write should succeed");
        writeln!(&mut output, "host path: {}", volume.path.display())
            .expect("write should succeed");
    }
    output
}

fn format_hosted_fleet_nodes(nodes: &[port_runtime::HostedFleetNodeStatus]) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    writeln!(&mut output, "fleet nodes:").expect("write should succeed");
    for node in nodes {
        writeln!(&mut output, "  node: {}", node.node_name).expect("write should succeed");
        writeln!(&mut output, "  configured: {}", node.configured).expect("write should succeed");
        writeln!(&mut output, "  imported: {}", node.imported).expect("write should succeed");
        writeln!(&mut output, "  registered: {}", node.registered).expect("write should succeed");
        writeln!(&mut output, "  selected: {}", node.selected).expect("write should succeed");
        writeln!(&mut output, "  freshness: {}", node.freshness).expect("write should succeed");
        writeln!(
            &mut output,
            "  routing eligibility: {}",
            node.routing_eligibility
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "  import provenance: {}",
            node.import_provenance.as_deref().unwrap_or("(none)")
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "  imported at: {}",
            node.imported_at_unix_s
                .map_or_else(|| String::from("(none)"), |value| value.to_string())
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "  refreshed at: {}",
            node.refreshed_at_unix_s
                .map_or_else(|| String::from("(none)"), |value| value.to_string())
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "  ttl seconds: {}",
            node.ttl_seconds
                .map_or_else(|| String::from("(none)"), |value| value.to_string())
        )
        .expect("write should succeed");
        writeln!(
            &mut output,
            "  fresh until: {}",
            node.fresh_until_unix_s
                .map_or_else(|| String::from("(none)"), |value| value.to_string())
        )
        .expect("write should succeed");
        writeln!(&mut output, "  node detail: {}", node.detail).expect("write should succeed");
    }
    output
}

fn run_service(command: ServiceCommand, config: &PortConfig) -> Result<()> {
    match command {
        ServiceCommand::Secret(command) => match command {
            ServiceSecretCommand::Put {
                machine,
                runtime_root,
                name,
                value,
            } => {
                ensure_machine_exists(config, &machine)?;
                let secret = port_runtime::put_machine_secret(
                    config,
                    port_runtime::SecretPutRequest {
                        machine_name: &machine,
                        runtime_root: &runtime_root,
                        name: &name,
                        value: &value,
                    },
                )?;
                print_machine_secret(&secret);
            }
            ServiceSecretCommand::List {
                machine,
                runtime_root,
            } => {
                ensure_machine_exists(config, &machine)?;
                let secrets = port_runtime::list_machine_secrets(config, &runtime_root, &machine)?;
                if secrets.is_empty() {
                    println!("no secrets stored for machine '{}'", machine);
                } else {
                    for secret in secrets {
                        print_machine_secret(&secret);
                        println!();
                    }
                }
            }
            ServiceSecretCommand::Remove {
                machine,
                runtime_root,
                name,
            } => {
                ensure_machine_exists(config, &machine)?;
                let secret =
                    port_runtime::delete_machine_secret(config, &runtime_root, &machine, &name)?;
                print_machine_secret(&secret);
            }
        },
        ServiceCommand::Apply {
            machine,
            runtime_root,
            name,
            kind,
            host_group,
            restart,
            health,
            health_command,
            secret,
            command,
        } => {
            ensure_machine_exists(config, &machine)?;
            let definition = port_runtime::apply_machine_service(
                config,
                port_runtime::ServiceApplyRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    name: &name,
                    kind: kind.into(),
                    host_group: host_group.as_deref(),
                    command,
                    secret_bindings: parse_secret_bindings(secret)?,
                    policy: ServicePolicy {
                        restart: restart.into(),
                        healthcheck: ServiceHealthcheck {
                            policy: health.into(),
                            command: health_command,
                        },
                    },
                },
            )?;
            print_service_definition(&definition);
        }
        ServiceCommand::List {
            machine,
            runtime_root,
        } => {
            ensure_machine_exists(config, &machine)?;
            let services = port_runtime::list_machine_services(config, &runtime_root, &machine)?;
            if services.is_empty() {
                println!("no services or sandboxes stored for machine '{}'", machine);
            } else {
                for service in services {
                    print_service_definition(&service);
                    println!();
                }
            }
        }
        ServiceCommand::Status {
            machine,
            runtime_root,
            name,
        } => {
            ensure_machine_exists(config, &machine)?;
            let service =
                port_runtime::machine_service_status(config, &runtime_root, &machine, &name)?;
            print_service_definition(&service);
        }
        ServiceCommand::Stop {
            machine,
            runtime_root,
            name,
        } => {
            ensure_machine_exists(config, &machine)?;
            let service =
                port_runtime::stop_machine_service(config, &runtime_root, &machine, &name)?;
            print_service_definition(&service);
        }
    }

    Ok(())
}

fn print_machine_monitor(report: &port_runtime::MachineMonitorReport) {
    println!("machine: {}", report.machine_name);
    println!("state: {}", report.state);
    println!(
        "pid: {}",
        report
            .pid
            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
    );
    println!("inventory scope: {}", report.control.inventory_scope);
    println!("inventory owner: {}", report.control.inventory_owner);
    println!("lifecycle owner: {}", report.control.lifecycle_owner);
    println!("status source: {}", report.control.status_source);
    println!("monitor route: {}", report.control.monitor_route);
    println!("top route: {}", report.control.top_route);
    println!(
        "control plane: {}",
        report.control_plane.as_deref().unwrap_or("(local)")
    );
    println!("node: {}", report.node_name.as_deref().unwrap_or("(local)"));
    println!(
        "host groups: {}",
        if report.host_groups.is_empty() {
            String::from("(none)")
        } else {
            report.host_groups.join(", ")
        }
    );
    println!("runtime dir: {}", report.runtime_dir.display());
    println!("config path: {}", report.config_path.display());
    println!("manifest: {}", report.manifest_path.display());
    println!("pid file: {}", report.pid_path.display());
    println!("hypervisor log: {}", report.firecracker_log.display());
    println!("console stdout: {}", report.stdout_log.display());
    println!("console stderr: {}", report.stderr_log.display());
    println!("detached forwards: {}", report.detached_forwards.len());
    for forward in &report.detached_forwards {
        println!();
        println!("forward: {}", forward.name);
        println!("state: {}", forward.state);
        println!(
            "pid: {}",
            forward
                .pid
                .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
        );
        println!("listen: {}", forward.listen);
        println!("target: {}", forward.target);
        println!("manifest: {}", forward.manifest_path.display());
        println!("stdout log: {}", forward.stdout_log.display());
        println!("stderr log: {}", forward.stderr_log.display());
        println!("detail: {}", forward.detail);
    }
    println!("detail: {}", report.detail);
}

fn print_machine_top(report: &port_runtime::MachineTopReport) {
    println!("machine: {}", report.machine_name);
    println!("state: {}", report.state);
    println!(
        "pid: {}",
        report
            .pid
            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
    );
    println!("inventory scope: {}", report.control.inventory_scope);
    println!("lifecycle owner: {}", report.control.lifecycle_owner);
    println!("status source: {}", report.control.status_source);
    println!("monitor route: {}", report.control.monitor_route);
    println!("top route: {}", report.control.top_route);
    println!(
        "control plane: {}",
        report.control_plane.as_deref().unwrap_or("(local)")
    );
    println!("node: {}", report.node_name.as_deref().unwrap_or("(local)"));
    println!(
        "host groups: {}",
        if report.host_groups.is_empty() {
            String::from("(none)")
        } else {
            report.host_groups.join(", ")
        }
    );
    println!("runtime dir: {}", report.runtime_dir.display());
    println!("detail: {}", report.detail);
    if report.entries.is_empty() {
        println!("entries: 0");
        return;
    }
    println!("entries: {}", report.entries.len());
    for entry in &report.entries {
        println!();
        println!("entry kind: {}", entry.kind);
        println!("name: {}", entry.name);
        println!("state: {}", entry.state);
        println!(
            "pid: {}",
            entry
                .pid
                .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
        );
        println!("command: {}", entry.command.as_deref().unwrap_or("(none)"));
        println!("source: {}", entry.source.display());
        println!("detail: {}", entry.detail);
    }
}

fn print_machine_secret(secret: &port_runtime::MachineSecretSummary) {
    println!("machine: {}", secret.machine_name);
    println!("secret: {}", secret.name);
    println!("backend: {}", secret.backend);
    println!("materialization: {}", secret.materialization);
    println!("inventory scope: {}", secret.control.inventory_scope);
    println!("lifecycle owner: {}", secret.control.lifecycle_owner);
    println!("guest broker: {}", secret.control.guest_broker);
    println!("service route: {}", secret.control.service_route);
    println!(
        "control plane: {}",
        secret.control_plane.as_deref().unwrap_or("(local)")
    );
    println!("node: {}", secret.node_name.as_deref().unwrap_or("(local)"));
    println!(
        "host groups: {}",
        if secret.host_groups.is_empty() {
            String::from("(none)")
        } else {
            secret.host_groups.join(", ")
        }
    );
    println!("path: {}", secret.path.display());
    println!("backend path: {}", secret.backend_path.display());
    println!("detail: {}", secret.detail);
}

fn print_service_definition(service: &port_runtime::ServiceDefinitionStatus) {
    println!("machine: {}", service.machine_name);
    println!("name: {}", service.name);
    println!("kind: {}", service.kind);
    println!("desired state: {}", service.desired_state);
    println!("runtime state: {}", service.runtime.state);
    println!("inventory scope: {}", service.control.inventory_scope);
    println!("lifecycle owner: {}", service.control.lifecycle_owner);
    println!("guest broker: {}", service.control.guest_broker);
    println!("service route: {}", service.control.service_route);
    println!(
        "control plane: {}",
        service.control_plane.as_deref().unwrap_or("(local)")
    );
    println!(
        "node: {}",
        service.node_name.as_deref().unwrap_or("(local)")
    );
    println!(
        "host groups: {}",
        if service.host_groups.is_empty() {
            String::from("(none)")
        } else {
            service.host_groups.join(", ")
        }
    );
    println!(
        "target host group: {}",
        service.target_host_group.as_deref().unwrap_or("(none)")
    );
    println!(
        "scheduler: {}",
        service
            .scheduler
            .map_or_else(|| String::from("(none)"), render_scheduler_policy)
    );
    println!("restart policy: {}", service.policy.restart);
    println!("health policy: {}", service.policy.healthcheck.policy);
    println!(
        "health command: {}",
        if service.policy.healthcheck.command.is_empty() {
            String::from("(none)")
        } else {
            service.policy.healthcheck.command.join(" ")
        }
    );
    println!("manifest: {}", service.manifest_path.display());
    println!("runtime record: {}", service.runtime.record_path.display());
    println!(
        "runtime pid: {}",
        service
            .runtime
            .pid
            .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
    );
    println!(
        "runtime exit code: {}",
        service
            .runtime
            .exit_code
            .map_or_else(|| String::from("(none)"), |code| code.to_string())
    );
    println!("restart count: {}", service.runtime.restart_count);
    println!(
        "last exit code: {}",
        service
            .runtime
            .last_exit_code
            .map_or_else(|| String::from("(none)"), |code| code.to_string())
    );
    println!(
        "last exit detail: {}",
        service
            .runtime
            .last_exit_detail
            .as_deref()
            .unwrap_or("(none)")
    );
    println!("health state: {}", service.runtime.health_state);
    println!(
        "health detail: {}",
        service.runtime.health_detail.as_deref().unwrap_or("(none)")
    );
    println!(
        "stdout log: {}",
        service
            .runtime
            .stdout_path
            .as_ref()
            .map_or_else(|| String::from("(none)"), |path| path.display().to_string())
    );
    println!(
        "stderr log: {}",
        service
            .runtime
            .stderr_path
            .as_ref()
            .map_or_else(|| String::from("(none)"), |path| path.display().to_string())
    );
    println!(
        "command: {}",
        if service.command.is_empty() {
            String::from("(none)")
        } else {
            service.command.join(" ")
        }
    );
    if service.secret_bindings.is_empty() {
        println!("secret bindings: (none)");
    } else {
        println!(
            "secret bindings: {}",
            service
                .secret_bindings
                .iter()
                .map(|binding| format!("{}={}", binding.env, binding.secret))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if service.secret_sources.is_empty() {
        println!("secret sources: (none)");
    } else {
        println!(
            "secret sources: {}",
            service
                .secret_sources
                .iter()
                .map(|source| format!(
                    "{}<={} via {}/{} @ {}",
                    source.env,
                    source.secret,
                    source.backend,
                    source.materialization,
                    source.path.display()
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("detail: {}", service.detail);
}

fn parse_secret_bindings(values: Vec<String>) -> Result<Vec<port_runtime::ServiceSecretBinding>> {
    let mut bindings = Vec::new();
    for value in values {
        let (env, secret) = value
            .split_once('=')
            .context("service --secret entries must use ENV=SECRET_NAME")?;
        if env.trim().is_empty() || secret.trim().is_empty() {
            bail!("service --secret entries must use ENV=SECRET_NAME");
        }
        bindings.push(port_runtime::ServiceSecretBinding {
            env: env.trim().to_string(),
            secret: secret.trim().to_string(),
        });
    }
    Ok(bindings)
}

fn render_scheduler_policy(policy: HostedSchedulerPolicy) -> String {
    match policy {
        HostedSchedulerPolicy::DeterministicFirstFit => String::from("deterministic-first-fit"),
        HostedSchedulerPolicy::Spread => String::from("spread"),
    }
}

fn run_guest(command: GuestCommand, config_path: Option<&Path>, config: &PortConfig) -> Result<()> {
    match command {
        GuestCommand::Exec {
            machine,
            runtime_root,
            command,
        } => {
            ensure_machine_exists(config, &machine)?;
            match port_runtime::execute_guest_operation(
                config,
                GuestRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    operation: GuestOperation::Exec(ExecRequest {
                        command,
                        cwd: None,
                        env: Default::default(),
                    }),
                },
            )? {
                OperationResult::Exec(result) => {
                    print!("{}", result.stdout);
                    eprint!("{}", result.stderr);
                }
                other => bail!("unexpected guest exec result: {other:?}"),
            }
        }
        GuestCommand::Copy {
            machine,
            runtime_root,
            direction,
            source,
            destination,
        } => {
            ensure_machine_exists(config, &machine)?;
            let result = port_runtime::copy_guest_file(
                config,
                GuestCopyRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    source: source.as_ref(),
                    destination: destination.as_ref(),
                    direction: direction.into(),
                },
            )?;
            println!(
                "copied {} bytes via {:?} to {}",
                result.bytes_copied, result.direction, result.path
            );
        }
        GuestCommand::Pty {
            machine,
            runtime_root,
            command,
        } => {
            ensure_machine_exists(config, &machine)?;
            let request = GuestRequest {
                machine_name: &machine,
                runtime_root: &runtime_root,
                operation: GuestOperation::Pty(PtyRequest {
                    command,
                    cols: 80,
                    rows: 24,
                }),
            };
            if machine_uses_hosted_control_plane(config, &machine)? {
                match port_runtime::execute_guest_operation(config, request)? {
                    OperationResult::Pty(result) => {
                        print!("{}", result.transcript);
                    }
                    other => bail!("unexpected guest pty result: {other:?}"),
                }
            } else {
                let stdin = std::io::stdin();
                let mut stdout = std::io::stdout();
                let mut stderr = std::io::stderr();
                let _ = port_runtime::stream_guest_pty(
                    config,
                    request,
                    stdin,
                    &mut stdout,
                    &mut stderr,
                )?;
            }
        }
        GuestCommand::Logs {
            machine,
            runtime_root,
            path,
            tail_lines,
            follow,
        } => {
            ensure_machine_exists(config, &machine)?;
            let request = GuestRequest {
                machine_name: &machine,
                runtime_root: &runtime_root,
                operation: GuestOperation::Logs(LogsRequest {
                    path,
                    follow,
                    tail_lines,
                }),
            };
            if follow && !machine_uses_hosted_control_plane(config, &machine)? {
                let mut stdout = std::io::stdout();
                let _ = port_runtime::stream_guest_logs(config, request, &mut stdout)?;
            } else {
                match port_runtime::execute_guest_operation(config, request)? {
                    OperationResult::Logs(result) => {
                        print!("{}", result.contents);
                    }
                    other => bail!("unexpected guest logs result: {other:?}"),
                }
            }
        }
        GuestCommand::Forward {
            machine,
            runtime_root,
            listen,
            target,
            lifecycle,
            name,
            list,
            stop,
        } => {
            ensure_machine_exists(config, &machine)?;
            let hosted = machine_uses_hosted_control_plane(config, &machine)?;
            if list {
                if stop {
                    bail!("--list and --stop are mutually exclusive");
                }
                if listen.is_some() || target.is_some() {
                    bail!("--list does not accept --listen or --target");
                }
                if hosted {
                    let forwards = port_runtime::list_hosted_detached_forwards(config, &machine)?;
                    if forwards.is_empty() {
                        println!("no detached forwards found for machine '{}'", machine);
                        return Ok(());
                    }
                    for forward in forwards {
                        let state = format!("{:?}", forward.state).to_ascii_lowercase();
                        println!("forward: {}", forward.name);
                        println!("state: {}", state);
                        if let Some(pid) = forward.pid {
                            println!("pid: {}", pid);
                        }
                        println!("listen: {}", forward.listen);
                        println!("target: {}", forward.target);
                        println!("stdout: {}", forward.stdout_log.display());
                        println!("stderr: {}", forward.stderr_log.display());
                        println!();
                    }
                    return Ok(());
                }
                return list_detached_forwards(config, &machine, &runtime_root);
            }
            if stop {
                if listen.is_some() || target.is_some() {
                    bail!("--stop does not accept --listen or --target");
                }
                let name = name
                    .as_deref()
                    .context("--stop requires --name to select a detached forward")?;
                if hosted {
                    let result =
                        port_runtime::stop_hosted_detached_forward(config, &machine, name)?;
                    println!("forward name: {}", result.name);
                    println!("forward lifecycle: detached");
                    println!("forward state: stopped");
                    if let Some(pid) = result.pid {
                        println!("forward pid: {}", pid);
                    }
                    return Ok(());
                }
                return stop_detached_forward(config, &machine, &runtime_root, name);
            }

            let listen = listen.context("forward serve requires --listen")?;
            let target = target.context("forward serve requires --target")?;
            if hosted {
                if lifecycle == ForwardLifecycleArg::Detached {
                    let result = port_runtime::start_hosted_detached_forward(
                        config,
                        &machine,
                        &listen,
                        &target,
                        name.as_deref(),
                    )?;
                    println!("forward name: {}", result.name);
                    println!("forward listening: {}", result.listen);
                    println!("forward target: {}", result.target);
                    println!("forward lifecycle: detached");
                    if let Some(pid) = result.pid {
                        println!("forward pid: {}", pid);
                    }
                    println!("forward stdout: {}", result.stdout_log.display());
                    println!("forward stderr: {}", result.stderr_log.display());
                    return Ok(());
                }
                if name.is_some() {
                    bail!("--name requires `--lifecycle detached`");
                }
                let result = port_runtime::execute_guest_operation(
                    config,
                    GuestRequest {
                        machine_name: &machine,
                        runtime_root: &runtime_root,
                        operation: GuestOperation::Forward(ForwardRequest {
                            listen: listen.clone(),
                            target: target.clone(),
                        }),
                    },
                )?;
                let OperationResult::Forward(result) = result else {
                    bail!("unexpected guest forward result: {result:?}");
                };
                println!("forward listening: {}", result.listen);
                println!("forward target: {}", result.target);
                println!("forward lifecycle: hosted-control-plane");
                return Ok(());
            }
            match lifecycle {
                ForwardLifecycleArg::Foreground => {
                    let session = port_runtime::prepare_guest_forward(
                        config,
                        GuestForwardRequest {
                            machine_name: &machine,
                            runtime_root: &runtime_root,
                            listen: &listen,
                            target: &target,
                        },
                    )?;
                    println!("forward listening: {}", session.listen_addr());
                    println!("forward target: {}", session.target());
                    println!("forward lifecycle: foreground; press Ctrl-C to stop");
                    session.serve()?;
                }
                ForwardLifecycleArg::Detached => {
                    let manifest = start_detached_forward(
                        config_path,
                        config,
                        &machine,
                        &runtime_root,
                        &listen,
                        &target,
                        name.as_deref(),
                    )?;
                    println!("forward name: {}", manifest.name);
                    println!("forward listening: {}", manifest.listen);
                    println!("forward target: {}", manifest.target);
                    println!("forward lifecycle: detached");
                    println!("forward pid: {}", manifest.pid);
                    println!("forward stdout: {}", manifest.stdout_log.display());
                    println!("forward stderr: {}", manifest.stderr_log.display());
                }
            }
        }
    }

    Ok(())
}

fn ensure_machine_exists(config: &PortConfig, machine: &str) -> Result<()> {
    if config.machines.contains_key(machine) {
        Ok(())
    } else {
        bail!("unknown machine '{machine}'")
    }
}

#[derive(Debug, Clone)]
struct SshMachineRouteContext {
    host_name: String,
    provider: port_model::HostProvider,
    destination: String,
    user: String,
    port: u16,
    control: port_model::MachineControlContract,
}

fn ssh_machine_route_context(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Option<SshMachineRouteContext>> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{machine_name}'"))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    let HostConnection::Ssh {
        destination,
        user,
        port,
    } = &host.connection
    else {
        return Ok(None);
    };
    Ok(Some(SshMachineRouteContext {
        host_name: machine.host.clone(),
        provider: host.provider,
        destination: destination.clone(),
        user: user.clone(),
        port: *port,
        control: config
            .machine_control_contract(machine_name)
            .map_err(anyhow::Error::from)
            .context("failed to resolve machine control contract")?,
    }))
}

fn render_host_provider(provider: port_model::HostProvider) -> &'static str {
    match provider {
        port_model::HostProvider::Local => "local",
        port_model::HostProvider::GenericLinux => "generic-linux",
        port_model::HostProvider::Aws => "aws",
        port_model::HostProvider::Gcp => "gcp",
        port_model::HostProvider::Azure => "azure",
    }
}

fn print_ssh_machine_route_context(context: &SshMachineRouteContext, route_label: &str) {
    println!("host: {}", context.host_name);
    println!("provider: {}", render_host_provider(context.provider));
    println!(
        "ssh target: {}@{}:{}",
        context.user, context.destination, context.port
    );
    println!("{}: {}", route_label, context.control.launch_route);
    println!("inventory owner: {}", context.control.inventory_owner);
    println!("lifecycle owner: {}", context.control.lifecycle_owner);
}

fn machine_uses_hosted_control_plane(config: &PortConfig, machine: &str) -> Result<bool> {
    let machine = config
        .machines
        .get(machine)
        .with_context(|| format!("unknown machine '{machine}'"))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    Ok(matches!(
        host.connection,
        HostConnection::HostedControlPlane { .. }
    ))
}

fn start_detached_forward(
    config_path: Option<&Path>,
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    listen: &str,
    target: &str,
    name: Option<&str>,
) -> Result<DetachedForwardManifest> {
    let state_dir = port_runtime::guest_forward_state_dir(config, machine, runtime_root)?;
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("failed to create '{}'", state_dir.display()))?;

    let name = name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("forward-{}", unix_timestamp()));
    let manifest_path = state_dir.join(format!("{name}.json"));
    let stdout_log = state_dir.join(format!("{name}.stdout.log"));
    let stderr_log = state_dir.join(format!("{name}.stderr.log"));

    let current_exe =
        std::env::current_exe().context("failed to resolve the current port executable")?;
    let mut command = ProcessCommand::new(current_exe);
    if let Some(config_path) = config_path {
        command.arg("--config").arg(config_path);
    }
    command
        .arg("internal")
        .arg("forward-daemon")
        .arg("--machine")
        .arg(machine)
        .arg("--runtime-root")
        .arg(runtime_root)
        .arg("--listen")
        .arg(listen)
        .arg("--target")
        .arg(target)
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--name")
        .arg(&name)
        .stdin(Stdio::null())
        .stdout(
            fs::File::create(&stdout_log)
                .with_context(|| format!("failed to create '{}'", stdout_log.display()))?,
        )
        .stderr(
            fs::File::create(&stderr_log)
                .with_context(|| format!("failed to create '{}'", stderr_log.display()))?,
        );
    configure_detached_session(&mut command);

    let child = command
        .spawn()
        .context("failed to start detached forward daemon")?;
    let pid = child.id();

    let manifest = wait_for_detached_forward_manifest(
        &manifest_path,
        DetachedForwardManifest {
            name,
            machine: machine.to_string(),
            pid,
            listen: listen.to_string(),
            target: target.to_string(),
            stdout_log,
            stderr_log,
        },
    )?;

    Ok(manifest)
}

fn cluster_forward_name(cluster_name: &str) -> String {
    format!("cluster-{cluster_name}-api")
}

fn service_forward_name(cluster_name: &str, forward_name: &str) -> String {
    format!("cluster-{cluster_name}-{forward_name}")
}

fn detached_forward_manifest_path(
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    name: &str,
) -> Result<PathBuf> {
    Ok(
        port_runtime::guest_forward_state_dir(config, machine, runtime_root)?
            .join(format!("{name}.json")),
    )
}

fn load_detached_forward_manifest(
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    name: &str,
) -> Result<Option<DetachedForwardManifest>> {
    let manifest_path = detached_forward_manifest_path(config, machine, runtime_root, name)?;
    if !manifest_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
    let manifest: DetachedForwardManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse '{}'", manifest_path.display()))?;
    Ok(Some(manifest))
}

fn ensure_detached_forward(
    config_path: Option<&Path>,
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    target: &str,
    name: &str,
) -> Result<EnsuredDetachedForward> {
    let existing = load_detached_forward_manifest(config, machine, runtime_root, name)?;
    if let Some(manifest) = existing.as_ref() {
        if pid_is_live(manifest.pid) && manifest.target == target {
            return Ok(EnsuredDetachedForward {
                manifest: manifest.clone(),
                action: "reused",
            });
        }
    }

    let action = if existing.is_some() {
        stop_detached_forward_if_present(config, machine, runtime_root, name)?;
        "restarted"
    } else {
        "started"
    };
    let manifest = start_detached_forward(
        config_path,
        config,
        machine,
        runtime_root,
        "127.0.0.1:0",
        target,
        Some(name),
    )?;
    Ok(EnsuredDetachedForward { manifest, action })
}

fn wait_for_detached_forward_manifest(
    manifest_path: &Path,
    fallback: DetachedForwardManifest,
) -> Result<DetachedForwardManifest> {
    for _ in 0..100 {
        if manifest_path.exists() {
            let bytes = fs::read(manifest_path)
                .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
            let manifest: DetachedForwardManifest =
                serde_json::from_slice(&bytes).with_context(|| {
                    format!(
                        "failed to parse detached forward manifest '{}'",
                        manifest_path.display()
                    )
                })?;
            return Ok(manifest);
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    Ok(fallback)
}

fn remove_detached_forward_resources(
    manifest_path: &Path,
    manifest: &DetachedForwardManifest,
) -> Result<()> {
    if pid_is_live(manifest.pid) {
        kill_pid(manifest.pid)?;
    }
    if let Some(socket_path) = manifest.listen.strip_prefix("unix:") {
        let socket_path = Path::new(socket_path);
        if socket_path.exists() {
            fs::remove_file(socket_path).with_context(|| {
                format!(
                    "failed to remove detached forward socket '{}'",
                    socket_path.display()
                )
            })?;
        }
    }
    if manifest_path.exists() {
        fs::remove_file(manifest_path)
            .with_context(|| format!("failed to remove '{}'", manifest_path.display()))?;
    }
    Ok(())
}

fn list_detached_forwards(config: &PortConfig, machine: &str, runtime_root: &Path) -> Result<()> {
    let state_dir = port_runtime::guest_forward_state_dir(config, machine, runtime_root)?;
    if !state_dir.exists() {
        println!("no detached forwards found for machine '{}'", machine);
        return Ok(());
    }

    let mut manifests = Vec::new();
    for entry in fs::read_dir(&state_dir)
        .with_context(|| format!("failed to read '{}'", state_dir.display()))?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(entry.path())?;
        let manifest: DetachedForwardManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse '{}'", entry.path().display()))?;
        manifests.push(manifest);
    }

    manifests.sort_by(|left, right| left.name.cmp(&right.name));
    if manifests.is_empty() {
        println!("no detached forwards found for machine '{}'", machine);
        return Ok(());
    }

    for manifest in manifests {
        let state = if pid_is_live(manifest.pid) {
            "running"
        } else {
            "stale"
        };
        println!("forward: {}", manifest.name);
        println!("state: {}", state);
        println!("pid: {}", manifest.pid);
        println!("listen: {}", manifest.listen);
        println!("target: {}", manifest.target);
        println!("stdout: {}", manifest.stdout_log.display());
        println!("stderr: {}", manifest.stderr_log.display());
        println!();
    }

    Ok(())
}

fn stop_detached_forward(
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    name: &str,
) -> Result<()> {
    let manifest = stop_detached_forward_if_present(config, machine, runtime_root, name)?
        .with_context(|| {
            format!("detached forward '{name}' was not found for machine '{machine}'")
        })?;

    println!("forward name: {}", manifest.name);
    println!("forward lifecycle: detached");
    println!("forward state: stopped");
    println!("forward pid: {}", manifest.pid);
    Ok(())
}

fn stop_detached_forward_if_present(
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    name: &str,
) -> Result<Option<DetachedForwardManifest>> {
    let manifest_path = detached_forward_manifest_path(config, machine, runtime_root, name)?;
    let Some(manifest) = load_detached_forward_manifest(config, machine, runtime_root, name)?
    else {
        return Ok(None);
    };
    remove_detached_forward_resources(&manifest_path, &manifest)?;
    Ok(Some(manifest))
}

fn run_forward_daemon(
    config: &PortConfig,
    machine: &str,
    runtime_root: &Path,
    listen: &str,
    target: &str,
    manifest_path: &Path,
    name: &str,
) -> Result<()> {
    ensure_machine_exists(config, machine)?;
    let session = port_runtime::prepare_guest_forward(
        config,
        GuestForwardRequest {
            machine_name: machine,
            runtime_root,
            listen,
            target,
        },
    )?;

    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    let stdout_log = manifest_path.with_extension("stdout.log");
    let stderr_log = manifest_path.with_extension("stderr.log");
    let manifest = DetachedForwardManifest {
        name: name.to_string(),
        machine: machine.to_string(),
        pid: std::process::id(),
        listen: session.listen_addr(),
        target: session.target().to_string(),
        stdout_log,
        stderr_log,
    };
    fs::write(
        manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("manifest should encode"),
    )
    .with_context(|| format!("failed to write '{}'", manifest_path.display()))?;

    session.serve()
}

fn pid_is_live(pid: u32) -> bool {
    ProcessCommand::new("kill")
        .args(["-0", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn configure_detached_session(command: &mut ProcessCommand) {
    // Keep detached forwards alive after the invoking CLI process exits.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

fn kill_pid(pid: u32) -> Result<()> {
    let status = ProcessCommand::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .with_context(|| format!("failed to signal pid {pid}"))?;
    if !status.success() {
        bail!("failed to stop detached forward pid {pid}");
    }
    Ok(())
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn rewrite_kubeconfig_server(kubeconfig: &str, server: &str) -> Result<String> {
    let mut rewritten = Vec::new();
    let mut replaced = false;
    for line in kubeconfig.lines() {
        if !replaced && line.trim_start().starts_with("server: ") {
            let indent_len = line.len() - line.trim_start().len();
            let indent = &line[..indent_len];
            let existing = line
                .trim_start()
                .strip_prefix("server: ")
                .unwrap_or_default()
                .trim();
            let rewritten_server = if server.contains("://") {
                server.to_string()
            } else if let Some((scheme, _)) = existing.split_once("://") {
                format!("{scheme}://{server}")
            } else {
                format!("https://{server}")
            };
            rewritten.push(format!("{indent}server: {rewritten_server}"));
            replaced = true;
        } else {
            rewritten.push(line.to_string());
        }
    }
    if !replaced {
        bail!("kubeconfig does not contain a server field to rewrite");
    }
    let mut output = rewritten.join("\n");
    if kubeconfig.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn load_config(path: Option<PathBuf>) -> Result<PortConfig> {
    let config = match path {
        Some(path) => PortConfig::from_path(&path)
            .with_context(|| format!("failed to load Port config from '{}'", path.display())),
        None => Ok(PortConfig::sample()),
    }?;
    validate_config(config)
}

fn load_config_if_present(path: Option<&std::path::Path>) -> Result<Option<PortConfig>> {
    path.map(PortConfig::from_path)
        .transpose()
        .map_err(anyhow::Error::from)?
        .map(validate_config)
        .transpose()
}

fn load_config_from_stdin() -> Result<PortConfig> {
    let mut encoded = String::new();
    std::io::stdin()
        .read_to_string(&mut encoded)
        .context("failed to read Port config from stdin")?;
    let config = serde_json::from_str::<PortConfig>(&encoded)
        .context("failed to decode Port config JSON from stdin")?;
    validate_config(config)
}

fn validate_config(config: PortConfig) -> Result<PortConfig> {
    config
        .validate()
        .with_context(|| "invalid Port config".to_string())?;
    Ok(config)
}

pub fn render_help() -> String {
    let mut command = Cli::command();
    command.render_long_help().to_string()
}

pub fn render_subcommand_help(name: &str) -> Option<String> {
    let mut command = Cli::command();
    let subcommand = command.find_subcommand_mut(name)?;
    Some(subcommand.render_long_help().to_string())
}

pub fn render_nested_subcommand_help(path: &[&str]) -> Option<String> {
    let mut command = Cli::command();
    let (last, parents) = path.split_last()?;
    let mut current = &mut command;
    for segment in parents {
        current = current.find_subcommand_mut(segment)?;
    }
    let nested = current.find_subcommand_mut(last)?;
    Some(nested.render_long_help().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use super::{
        ArchitectureArg, ArtifactCommand, Cli, ClusterCommand, Command, ControlPlaneCommand,
        CopyDirectionArg, GuestCommand, HostedNodeBindingArg, MachineCommand, NodeAgentCommand,
        ProtectionModeArg, ServiceCommand, ServiceHealthPolicyArg, ServiceKindArg,
        ServiceRestartPolicyArg, ServiceSecretCommand, SubstrateArg, format_hosted_fleet_nodes,
        format_machine_status, render_help, render_nested_subcommand_help, render_subcommand_help,
    };

    fn sample_hosted_status(
        hosted_fleet_nodes: Vec<port_runtime::HostedFleetNodeStatus>,
        detail: &str,
    ) -> port_runtime::MachineStatus {
        port_runtime::MachineStatus {
            machine_name: String::from("cloud-aws"),
            state: port_runtime::MachineRuntimeState::Malformed,
            pid: Some(424242),
            control: port_model::MachineControlContract::hosted_control_plane(),
            runtime_dir: PathBuf::from("/tmp/runtime/cloud-aws"),
            config_path: PathBuf::from("/tmp/runtime/cloud-aws/firecracker-config.json"),
            manifest_path: PathBuf::from("/tmp/runtime/cloud-aws/manifest.json"),
            pid_path: PathBuf::from("/tmp/runtime/cloud-aws/firecracker.pid"),
            firecracker_log: PathBuf::from("/tmp/runtime/cloud-aws/firecracker.log"),
            stdout_log: PathBuf::from("/tmp/runtime/cloud-aws/console.stdout.log"),
            stderr_log: PathBuf::from("/tmp/runtime/cloud-aws/console.stderr.log"),
            runtime_class: Some(port_model::MachineRuntimeClassSpec {
                kind: port_model::MachineRuntimeClassKind::WorkspaceScratchBuilder,
                trust: port_model::MachineRuntimeTrustPosture::WorkspaceUntrusted,
                state_isolation: port_model::MachineRuntimeStateIsolation::WorkspaceWritable,
                writable_roots: vec![
                    port_model::MachineRuntimeWritableRoot::NixStore,
                    port_model::MachineRuntimeWritableRoot::SourceRoot,
                    port_model::MachineRuntimeWritableRoot::TempRoot,
                ],
                declared_inputs: Vec::new(),
                workspace: Some(port_model::MachineRuntimeWorkspaceBinding {
                    workspace: String::from("acme"),
                    lane: String::from("scratch"),
                }),
            }),
            attached_volumes: Vec::new(),
            hosted_fleet_nodes,
            detail: detail.to_string(),
        }
    }

    #[test]
    fn help_includes_primary_surfaces() {
        let help = render_help();
        let guest_help = render_subcommand_help("guest").expect("guest help should exist");

        for keyword in [
            "doctor",
            "artifacts",
            "cluster",
            "machine",
            "guest",
            "service",
            "control-plane",
            "node-agent",
            "--config <PATH>",
            "examples/port.toml",
            "Quick start:",
            "Examples:",
            "port --config examples/port.toml artifacts list",
            "port doctor",
            "port --config examples/port.toml artifacts build --artifact demo-kernel",
            "port --config examples/port.toml cluster show --cluster demo",
            "port --config examples/port.toml cluster up --cluster demo",
            "port --config examples/port.toml cluster kubeconfig --cluster demo",
            "port --config examples/port.toml machine list",
            "port --config examples/port.toml guest exec --machine demo",
        ] {
            assert!(help.contains(keyword), "missing help keyword: {keyword}");
        }

        let cluster_help = render_subcommand_help("cluster").expect("cluster help should exist");
        let machine_help = render_subcommand_help("machine").expect("machine help should exist");

        for keyword in ["exec", "copy", "pty", "logs", "forward"] {
            assert!(
                guest_help.contains(keyword),
                "missing guest help keyword: {keyword}"
            );
        }

        let artifact_help = render_nested_subcommand_help(&["artifacts", "push"])
            .expect("artifact help should exist");

        for keyword in ["list", "show"] {
            assert!(
                cluster_help.contains(keyword),
                "missing cluster help keyword: {keyword}"
            );
        }

        for keyword in ["launch", "list", "status", "stop", "monitor", "top"] {
            assert!(
                machine_help.contains(keyword),
                "missing machine help keyword: {keyword}"
            );
        }

        let service_help = render_subcommand_help("service").expect("service help should exist");
        let service_apply_help = render_nested_subcommand_help(&["service", "apply"])
            .expect("service apply help should exist");
        for keyword in ["secret", "apply", "list", "status", "stop", "sandbox"] {
            assert!(
                service_help.contains(keyword),
                "missing service help keyword: {keyword}"
            );
        }

        for keyword in [
            "--restart",
            "--health",
            "--health-command",
            "on-failure",
            "command",
        ] {
            assert!(
                service_apply_help.contains(keyword),
                "missing service apply help keyword: {keyword}"
            );
        }

        for keyword in [
            "architecture",
            "substrate",
            "protection-mode",
            "Publish a selected artifact variant",
        ] {
            assert!(
                artifact_help.contains(keyword),
                "missing artifact help keyword: {keyword}"
            );
        }
    }

    #[test]
    fn help_includes_machine_commands_examples() {
        help_includes_primary_surfaces();
    }

    #[test]
    fn hosted_fleet_render_includes_node_state_breakdown() {
        let rendered = format_hosted_fleet_nodes(&[
            port_runtime::HostedFleetNodeStatus {
                node_name: String::from("aws-linux-node"),
                configured: true,
                imported: false,
                registered: true,
                selected: true,
                freshness: port_runtime::HostedFleetFreshnessState::Live,
                routing_eligibility: port_runtime::HostedFleetRoutingEligibility::Eligible,
                import_provenance: None,
                imported_at_unix_s: None,
                refreshed_at_unix_s: Some(1773044061),
                ttl_seconds: Some(15),
                fresh_until_unix_s: Some(1773044076),
                detail: String::from("Selected by the current control-plane route."),
            },
            port_runtime::HostedFleetNodeStatus {
                node_name: String::from("aws-linux-node-c"),
                configured: true,
                imported: true,
                registered: false,
                selected: false,
                freshness: port_runtime::HostedFleetFreshnessState::MissingRegistration,
                routing_eligibility:
                    port_runtime::HostedFleetRoutingEligibility::MissingRegistration,
                import_provenance: Some(String::from("imported/aws-linux-node-c.json")),
                imported_at_unix_s: Some(1_700_000_123),
                refreshed_at_unix_s: None,
                ttl_seconds: None,
                fresh_until_unix_s: None,
                detail: String::from("Imported inventory from aws-linux-node-c.json."),
            },
        ]);

        for expected in [
            "fleet nodes:",
            "node: aws-linux-node",
            "selected: true",
            "freshness: live",
            "routing eligibility: eligible",
            "node: aws-linux-node-c",
            "imported: true",
            "registered: false",
            "import provenance: imported/aws-linux-node-c.json",
            "freshness: missing-registration",
            "routing eligibility: missing-registration",
        ] {
            assert!(
                rendered.contains(expected),
                "missing '{expected}' in:\n{rendered}"
            );
        }
    }

    #[test]
    fn hosted_fleet_render_distinguishes_live_stale_and_missing_nodes() {
        let rendered = format_machine_status(&sample_hosted_status(
            vec![
                port_runtime::HostedFleetNodeStatus {
                    node_name: String::from("aws-linux-node"),
                    configured: true,
                    imported: false,
                    registered: true,
                    selected: true,
                    freshness: port_runtime::HostedFleetFreshnessState::Live,
                    routing_eligibility: port_runtime::HostedFleetRoutingEligibility::Eligible,
                    import_provenance: None,
                    imported_at_unix_s: None,
                    refreshed_at_unix_s: Some(1773044061),
                    ttl_seconds: Some(15),
                    fresh_until_unix_s: Some(1773044076),
                    detail: String::from("Live node."),
                },
                port_runtime::HostedFleetNodeStatus {
                    node_name: String::from("aws-linux-node-b"),
                    configured: true,
                    imported: false,
                    registered: true,
                    selected: false,
                    freshness: port_runtime::HostedFleetFreshnessState::Stale,
                    routing_eligibility:
                        port_runtime::HostedFleetRoutingEligibility::StaleRegistration,
                    import_provenance: None,
                    imported_at_unix_s: None,
                    refreshed_at_unix_s: Some(1),
                    ttl_seconds: Some(1),
                    fresh_until_unix_s: Some(2),
                    detail: String::from("Registration is stale."),
                },
                port_runtime::HostedFleetNodeStatus {
                    node_name: String::from("aws-linux-node-c"),
                    configured: true,
                    imported: true,
                    registered: false,
                    selected: false,
                    freshness: port_runtime::HostedFleetFreshnessState::MissingRegistration,
                    routing_eligibility:
                        port_runtime::HostedFleetRoutingEligibility::MissingRegistration,
                    import_provenance: Some(String::from("imported/aws-linux-node-c.json")),
                    imported_at_unix_s: Some(1_700_000_123),
                    refreshed_at_unix_s: None,
                    ttl_seconds: None,
                    fresh_until_unix_s: None,
                    detail: String::from("No registered node-agent endpoint."),
                },
            ],
            "control plane 'demo' could not inspect hosted fleet state for machine 'cloud-aws'",
        ));

        for expected in [
            "freshness: live",
            "freshness: stale",
            "routing eligibility: stale-registration",
            "freshness: missing-registration",
            "routing eligibility: missing-registration",
            "detail: control plane 'demo' could not inspect hosted fleet state for machine 'cloud-aws'",
        ] {
            assert!(
                rendered.contains(expected),
                "missing '{expected}' in:\n{rendered}"
            );
        }
    }

    #[test]
    fn machine_status_render_includes_runtime_class_contract() {
        let rendered = format_machine_status(&sample_hosted_status(
            Vec::new(),
            "runtime class contract should render",
        ));

        for expected in [
            "runtime class: workspace-scratch-builder",
            "trust posture: workspace-untrusted",
            "state isolation: workspace-writable",
            "workspace: acme",
            "workspace lane: scratch",
            "writable root: nix-store",
            "writable root: source-root",
            "writable root: temp-root",
        ] {
            assert!(
                rendered.contains(expected),
                "missing '{expected}' in:\n{rendered}"
            );
        }
    }

    #[test]
    fn machine_status_render_includes_promotion_runner_contract() {
        let mut status = sample_hosted_status(Vec::new(), "promotion contract should render");
        status.runtime_class = Some(port_model::MachineRuntimeClassSpec {
            kind: port_model::MachineRuntimeClassKind::BlessedClosurePromotionRunner,
            trust: port_model::MachineRuntimeTrustPosture::PromotionTrusted,
            state_isolation: port_model::MachineRuntimeStateIsolation::CleanRoom,
            writable_roots: vec![port_model::MachineRuntimeWritableRoot::EvidenceRoot],
            declared_inputs: vec![
                port_model::MachineRuntimeDeclaredInput::SourceBundle,
                port_model::MachineRuntimeDeclaredInput::RequestedOutputs,
                port_model::MachineRuntimeDeclaredInput::PolicySnapshot,
            ],
            workspace: None,
        });

        let rendered = format_machine_status(&status);

        for expected in [
            "runtime class: blessed-closure-promotion-runner",
            "trust posture: promotion-trusted",
            "state isolation: clean-room",
            "declared input: source-bundle",
            "declared input: requested-outputs",
            "declared input: policy-snapshot",
            "writable root: evidence-root",
        ] {
            assert!(
                rendered.contains(expected),
                "missing '{expected}' in:\n{rendered}"
            );
        }
    }

    #[test]
    fn parses_machine_launch_arguments() {
        let cli = Cli::parse_from([
            "port",
            "--config",
            "examples/port.toml",
            "machine",
            "launch",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
        ]);

        match cli.command {
            Command::Machine(MachineCommand::Launch {
                machine,
                runtime_root,
                boot_wait_secs,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(boot_wait_secs, 3);
                assert_eq!(
                    cli.config.as_deref(),
                    Some(std::path::Path::new("examples/port.toml"))
                );
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_cluster_contract_arguments() {
        let list = Cli::parse_from(["port", "cluster", "list", "--format", "json"]);
        match list.command {
            Command::Cluster(ClusterCommand::List { format }) => {
                assert_eq!(format, super::OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let show = Cli::parse_from([
            "port",
            "--config",
            "examples/port.toml",
            "cluster",
            "show",
            "--cluster",
            "demo",
        ]);
        match show.command {
            Command::Cluster(ClusterCommand::Show { cluster, format }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(format, super::OutputFormat::Text);
                assert_eq!(
                    show.config.as_deref(),
                    Some(std::path::Path::new("examples/port.toml"))
                );
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let stage = Cli::parse_from([
            "port",
            "cluster",
            "stage",
            "--cluster",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--format",
            "json",
        ]);
        match stage.command {
            Command::Cluster(ClusterCommand::Stage {
                cluster,
                runtime_root,
                format,
            }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(format, super::OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let up = Cli::parse_from([
            "port",
            "cluster",
            "up",
            "--cluster",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--boot-wait-secs",
            "9",
            "--format",
            "json",
        ]);
        match up.command {
            Command::Cluster(ClusterCommand::Up {
                cluster,
                runtime_root,
                boot_wait_secs,
                format,
            }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(boot_wait_secs, 9);
                assert_eq!(format, super::OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let status = Cli::parse_from([
            "port",
            "cluster",
            "status",
            "--cluster",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
        ]);
        match status.command {
            Command::Cluster(ClusterCommand::Status {
                cluster,
                runtime_root,
                format,
            }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(format, super::OutputFormat::Text);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let kubeconfig = Cli::parse_from([
            "port",
            "cluster",
            "kubeconfig",
            "--cluster",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--format",
            "json",
        ]);
        match kubeconfig.command {
            Command::Cluster(ClusterCommand::Kubeconfig {
                cluster,
                runtime_root,
                format,
            }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(format, super::OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let down = Cli::parse_from([
            "port",
            "cluster",
            "down",
            "--cluster",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--stop-wait-secs",
            "5",
        ]);
        match down.command {
            Command::Cluster(ClusterCommand::Down {
                cluster,
                runtime_root,
                stop_wait_secs,
                format,
            }) => {
                assert_eq!(cluster, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(stop_wait_secs, 5);
                assert_eq!(format, super::OutputFormat::Text);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_guest_exec_trailing_command() {
        let cli = Cli::parse_from([
            "port",
            "guest",
            "exec",
            "--machine",
            "demo",
            "--",
            "/bin/sh",
            "-lc",
            "uname -a",
        ]);

        match cli.command {
            Command::Guest(GuestCommand::Exec {
                machine,
                runtime_root,
                command,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("runtime"));
                assert_eq!(command, ["/bin/sh", "-lc", "uname -a"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_guest_copy_direction_and_runtime_root() {
        let cli = Cli::parse_from([
            "port",
            "guest",
            "copy",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--direction",
            "guest-to-host",
            "--source",
            "/tmp/in-guest.txt",
            "--destination",
            "./copied.txt",
        ]);

        match cli.command {
            Command::Guest(GuestCommand::Copy {
                machine,
                runtime_root,
                direction,
                source,
                destination,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(direction, CopyDirectionArg::GuestToHost);
                assert_eq!(source, "/tmp/in-guest.txt");
                assert_eq!(destination, "./copied.txt");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_artifact_push_with_variant_selection() {
        let cli = Cli::parse_from([
            "port",
            "artifacts",
            "push",
            "--artifact",
            "demo-kernel",
            "--architecture",
            "x86-64",
            "--substrate",
            "firecracker",
            "--protection-mode",
            "standard",
        ]);

        match cli.command {
            Command::Artifacts(ArtifactCommand::Push {
                artifact,
                selection,
            }) => {
                assert_eq!(artifact, "demo-kernel");
                assert_eq!(selection.architecture, ArchitectureArg::X86_64);
                assert_eq!(selection.substrate, SubstrateArg::Firecracker);
                assert_eq!(selection.protection_mode, ProtectionModeArg::Standard);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_artifact_list_with_json_output() {
        let cli = Cli::parse_from(["port", "artifacts", "list", "--format", "json"]);

        match cli.command {
            Command::Artifacts(ArtifactCommand::List { format }) => {
                assert_eq!(format, super::OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_machine_lifecycle_arguments() {
        let list = Cli::parse_from(["port", "machine", "list", "--runtime-root", "/tmp/runtime"]);
        match list.command {
            Command::Machine(MachineCommand::List { runtime_root }) => {
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let status = Cli::parse_from([
            "port",
            "machine",
            "status",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
        ]);

        match status.command {
            Command::Machine(MachineCommand::Status {
                machine,
                runtime_root,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let stop = Cli::parse_from([
            "port",
            "machine",
            "stop",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
            "--wait-secs",
            "9",
        ]);

        match stop.command {
            Command::Machine(MachineCommand::Stop {
                machine,
                runtime_root,
                wait_secs,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
                assert_eq!(wait_secs, 9);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let monitor = Cli::parse_from([
            "port",
            "machine",
            "monitor",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
        ]);
        match monitor.command {
            Command::Machine(MachineCommand::Monitor {
                machine,
                runtime_root,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let top = Cli::parse_from([
            "port",
            "machine",
            "top",
            "--machine",
            "demo",
            "--runtime-root",
            "/tmp/runtime",
        ]);
        match top.command {
            Command::Machine(MachineCommand::Top {
                machine,
                runtime_root,
            }) => {
                assert_eq!(machine, "demo");
                assert_eq!(runtime_root, std::path::Path::new("/tmp/runtime"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_service_and_secret_arguments() {
        let apply = Cli::parse_from([
            "port",
            "service",
            "apply",
            "--machine",
            "cloud-aws",
            "--host-group",
            "aws-builders",
            "--name",
            "api",
            "--kind",
            "service",
            "--restart",
            "on-failure",
            "--health",
            "command",
            "--health-command",
            "/bin/true",
            "--secret",
            "API_TOKEN=demo-token",
            "--",
            "/app/api",
            "--listen",
            ":8080",
        ]);

        match apply.command {
            Command::Service(ServiceCommand::Apply {
                machine,
                runtime_root,
                name,
                kind,
                host_group,
                restart,
                health,
                health_command,
                secret,
                command,
            }) => {
                assert_eq!(machine, "cloud-aws");
                assert_eq!(runtime_root, std::path::Path::new("runtime"));
                assert_eq!(name, "api");
                assert_eq!(kind, ServiceKindArg::Service);
                assert_eq!(host_group.as_deref(), Some("aws-builders"));
                assert_eq!(restart, ServiceRestartPolicyArg::OnFailure);
                assert_eq!(health, ServiceHealthPolicyArg::Command);
                assert_eq!(health_command, vec![String::from("/bin/true")]);
                assert_eq!(secret, vec![String::from("API_TOKEN=demo-token")]);
                assert_eq!(command, vec!["/app/api", "--listen", ":8080"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let secret_put = Cli::parse_from([
            "port",
            "service",
            "secret",
            "put",
            "--machine",
            "cloud-aws",
            "--name",
            "demo-token",
            "--value",
            "s3cr3t",
        ]);

        match secret_put.command {
            Command::Service(ServiceCommand::Secret(ServiceSecretCommand::Put {
                machine,
                runtime_root,
                name,
                value,
            })) => {
                assert_eq!(machine, "cloud-aws");
                assert_eq!(runtime_root, std::path::Path::new("runtime"));
                assert_eq!(name, "demo-token");
                assert_eq!(value, "s3cr3t");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_control_plane_serve_arguments() {
        let cli = Cli::parse_from([
            "port",
            "--config",
            "examples/port.toml",
            "control-plane",
            "serve",
            "--control-plane",
            "demo",
            "--bind",
            "127.0.0.1:7040",
            "--node-binding",
            "aws-linux-node=http://127.0.0.1:9234,node-secret",
        ]);

        match cli.command {
            Command::ControlPlane(ControlPlaneCommand::Serve {
                control_plane,
                bind,
                node_bindings,
            }) => {
                assert_eq!(control_plane, "demo");
                assert_eq!(bind, "127.0.0.1:7040");
                assert_eq!(node_bindings.len(), 1);
                let HostedNodeBindingArg(binding) = &node_bindings[0];
                assert_eq!(binding.node_name, "aws-linux-node");
                assert_eq!(binding.endpoint, "http://127.0.0.1:9234");
                assert_eq!(binding.token, "node-secret");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_control_plane_node_binding() {
        let error = Cli::try_parse_from([
            "port",
            "control-plane",
            "serve",
            "--control-plane",
            "demo",
            "--node-binding",
            "aws-linux-node=http://127.0.0.1:9234",
        ])
        .expect_err("missing token should fail");

        let rendered = error.to_string();
        assert!(rendered.contains("<node>=<endpoint>,<token>"));
    }

    #[test]
    fn parses_node_agent_serve_arguments() {
        let cli = Cli::parse_from([
            "port",
            "node-agent",
            "serve",
            "--node",
            "aws-linux-node",
            "--bind",
            "127.0.0.1:9234",
            "--token",
            "node-secret",
        ]);

        match cli.command {
            Command::NodeAgent(NodeAgentCommand::Serve { node, bind, token }) => {
                assert_eq!(node, "aws-linux-node");
                assert_eq!(bind, "127.0.0.1:9234");
                assert_eq!(token, "node-secret");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}

impl From<DoctorReport> for RenderedDoctorReport {
    fn from(value: DoctorReport) -> Self {
        Self {
            host_os: value.host_os,
            local_firecracker_supported: value.local_firecracker_supported,
            notes: value.notes,
            checks: value
                .checks
                .into_iter()
                .map(|check| RenderedDoctorCheck {
                    name: check.name,
                    ok: check.ok,
                    required: check.required,
                    detail: check.detail,
                })
                .collect(),
        }
    }
}

impl From<CopyDirectionArg> for CopyDirection {
    fn from(value: CopyDirectionArg) -> Self {
        match value {
            CopyDirectionArg::HostToGuest => Self::HostToGuest,
            CopyDirectionArg::GuestToHost => Self::GuestToHost,
        }
    }
}
