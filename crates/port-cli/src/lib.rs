use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use port_agent_protocol::{
    CopyDirection, ExecRequest, GuestOperation, LogsRequest, OperationResult, PtyRequest,
};
use port_model::{ExecutionSubstrate, MachineArchitecture, PortConfig, ProtectionMode};
use port_runtime::{
    ArtifactRequest, DoctorReport, GuestCopyRequest, GuestForwardRequest, GuestRequest,
    LaunchRequest,
};
use serde::Serialize;

const AFTER_HELP: &str = "\
Example assumptions:
  Run these sample-config commands from the repository root.
  Local artifact and launch workflows require the needed host tools to be available in the execution environment.
  Treat `port doctor` as the gate for whether local `port machine launch` can succeed.

Runnable local workflow:
  port doctor
  port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
  port --config examples/port.toml artifacts build --artifact demo-guest --architecture native
  port --config examples/port.toml artifacts push --artifact demo-kernel --architecture native
  port --config examples/port.toml artifacts pull --artifact demo-kernel --architecture native
  port --config examples/port.toml doctor
  port --config examples/port.toml machine launch --machine demo
  port --config examples/port.toml machine list
  port --config examples/port.toml machine status --machine demo
  port --config examples/port.toml machine stop --machine demo

Guest workflow examples:
  port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'uname -a'
  port --config examples/port.toml guest copy --machine demo --direction host-to-guest --source ./host.txt --destination /workspace/host.txt
  port --config examples/port.toml guest logs --machine demo --path /var/log/port-agent.log --tail-lines 50
  port --config examples/port.toml guest forward --machine demo --listen 127.0.0.1:8080 --target 127.0.0.1:80
  `port guest exec`, `copy`, `pty`, `logs`, and `forward` work against launched Firecracker VMs through the live guest transport.
  `port guest forward` is a foreground host-side proxy session; stop it with Ctrl-C when you are done.
  Guest-side `forward --target` addresses still depend on the guest network state. In the sample guest image, bring loopback up before targeting `127.0.0.1`, for example with `port guest exec --machine demo -- /bin/sh -lc 'busybox ifconfig lo up'`.

Platform Support:
  Linux: local Firecracker launch is supported when port doctor passes.
  macOS: run Port on a Linux host today; Apple Virtualization Framework is the first-class planned macOS lane.
  Windows: use WSL or a remote Linux host, then rely on port doctor to confirm whether local launch is supported.
Execution Lanes:
  Firecracker + standard on Linux is the current shipped lane.
  Firecracker + pvm on x86_64 is planned for cloud cost control and depends on a dedicated host kit plus pvm artifact variants.
  Firecracker + pvm on aarch64 remains research only until Port has a supportable Firecracker runtime path.
  Cloud Hypervisor and Apple Virtualization Framework are modeled explicitly as planned lanes.
  The AVF contract keeps the current guest protocol over AVF virtio sockets and uses AVF serial ports for console capture.
Cloud Linux:
  generic-linux, aws, and gcp providers are modeled through the shared config and surfaced by port doctor.
  port machine launch remains local-Linux-only in the MVP and returns provider-aware guidance for remote hosts.
Artifact Mobility:
  `port artifacts build` and `validate` materialize one canonical local variant selected by architecture, substrate, and protection mode.
  `port artifacts push` and `pull` use the artifact's configured mobility backend. The sample config ships a file-backed registry/cache contract; OCI and hosted backends remain modeled but reserved.
Hosted Control:
  Local Port still owns runtime lifecycle directly today.
  Hosted Port will move lifecycle ownership to a node agent plus control plane while preserving the current guest protocol semantics.
  The sample config now declares `[control_planes.demo]` with endpoint `https://port.example.internal`.
  Hosted auth is modeled explicitly as a bearer token read from `PORT_DEMO_TOKEN` through the `authorization` header.
  Remote/cloud sample hosts now use `mode = \"hosted-control-plane\"` and `control_plane = \"demo\"` instead of SSH placeholders.
  Hosted inventory is now modeled through `[nodes.<name>]` and `[host_groups.<name>]` so later scheduler, monitoring, and services work can reuse one placement vocabulary.
  Hosted `machine list`, `status`, and `stop` are now modeled explicitly as control-plane plus node-agent contracts so the canonical machine verbs stay stable as Port moves from local to hosted execution.
  In the MVP, those verbs still run only against the local runtime; the hosted lane is a published routing and ownership contract, not a runnable remote lifecycle yet.
  Hosted guest attach is now modeled explicitly: the control plane authorizes the attach, the node agent opens the host-local guest transport, and `port guest exec|copy|pty|logs|forward` keep the same request and response frames.
  In the MVP, those guest verbs still run only through the local runtime path; the hosted lane is a published bridge contract for the next control-plane, node-agent, and follow-on service slices.
  See `docs/pvm.md` for the explicit Firecracker/PVM host-kit contract and the x86_64 keep versus aarch64 research-only decision.
  See `docs/avf.md` for the AVF launch, guest-transport, serial-console, entitlement, and Rosetta workflow contract.
  Azure remains an explicitly unsupported Firecracker provider lane.";

#[derive(Debug, Parser)]
#[command(
    name = "port",
    version,
    about = "CLI-first Firecracker orchestration for local and cloud Linux hosts",
    long_about = "Port manages microVM-backed workloads through one canonical CLI and shared machine model. Firecracker with standard protection on Linux is the current execution lane; Firecracker/PVM, Cloud Hypervisor, and Apple Virtualization Framework are modeled explicitly as planned or research-backed lanes; remote generic-linux, AWS, and GCP hosts are surfaced through provider-aware diagnostics; macOS and Windows operators use Linux, WSL, or future substrate-specific workflows.",
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
    #[command(subcommand, about = "Build and validate kernel or guest artifacts")]
    Artifacts(ArtifactCommand),
    #[command(subcommand, about = "Launch and inspect Port-managed machines")]
    Machine(MachineCommand),
    #[command(subcommand, about = "Reach guest agent capabilities")]
    Guest(GuestCommand),
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
    #[command(about = "Forward a local listener into the guest through the agent")]
    Forward {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "runtime")]
        runtime_root: PathBuf,
        #[arg(long)]
        listen: String,
        #[arg(long)]
        target: String,
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

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Doctor { format } => doctor(format, cli.config.as_deref()),
        Command::Artifacts(command) => {
            let config = load_config(cli.config)?;
            run_artifacts(command, &config)
        }
        Command::Machine(command) => run_machine(command, cli.config),
        Command::Guest(command) => {
            let config = load_config(cli.config)?;
            run_guest(command, &config)
        }
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
    match command {
        MachineCommand::Launch {
            machine,
            runtime_root,
            boot_wait_secs,
        } => {
            let config = load_config(config_path)?;
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
                "firecracker binary: {}",
                metadata.firecracker_binary.display()
            );
            println!("config path: {}", metadata.config_path.display());
            println!("firecracker log: {}", metadata.log_path.display());
            println!("console stdout: {}", metadata.stdout_path.display());
            println!("console stderr: {}", metadata.stderr_path.display());
            println!("manifest: {}", metadata.manifest_path.display());
        }
        MachineCommand::List { runtime_root } => {
            let machines = port_runtime::list_machines(&runtime_root)?;
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
                    println!("runtime dir: {}", machine.runtime_dir.display());
                    println!("detail: {}", machine.detail);
                    println!();
                }
            }
        }
        MachineCommand::Status {
            machine,
            runtime_root,
        } => {
            let status = port_runtime::machine_status(&runtime_root, &machine)?;
            println!("machine: {}", status.machine_name);
            println!("state: {}", status.state);
            println!(
                "pid: {}",
                status
                    .pid
                    .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
            );
            println!("inventory scope: {}", status.control.inventory_scope);
            println!("inventory owner: {}", status.control.inventory_owner);
            println!("lifecycle owner: {}", status.control.lifecycle_owner);
            println!("guest broker: {}", status.control.guest_broker);
            println!("status source: {}", status.control.status_source);
            println!("launch route: {}", status.control.launch_route);
            println!("inventory route: {}", status.control.inventory_route);
            println!("status route: {}", status.control.status_route);
            println!("stop route: {}", status.control.stop_route);
            println!("guest route: {}", status.control.guest_route);
            println!("runtime dir: {}", status.runtime_dir.display());
            println!("config path: {}", status.config_path.display());
            println!("manifest: {}", status.manifest_path.display());
            println!("pid file: {}", status.pid_path.display());
            println!("firecracker log: {}", status.firecracker_log.display());
            println!("console stdout: {}", status.stdout_log.display());
            println!("console stderr: {}", status.stderr_log.display());
            println!("detail: {}", status.detail);
        }
        MachineCommand::Stop {
            machine,
            runtime_root,
            wait_secs,
        } => {
            let result = port_runtime::stop_machine(
                &runtime_root,
                &machine,
                Duration::from_secs(wait_secs),
            )?;
            println!("machine: {}", result.machine_name);
            println!("previous state: {}", result.previous_state);
            println!("current state: {}", result.current_state);
            println!(
                "pid: {}",
                result
                    .pid
                    .map_or_else(|| String::from("(none)"), |pid| pid.to_string())
            );
            println!("inventory scope: {}", result.control.inventory_scope);
            println!("lifecycle owner: {}", result.control.lifecycle_owner);
            println!("status source: {}", result.control.status_source);
            println!("stop route: {}", result.control.stop_route);
            println!("runtime dir: {}", result.runtime_dir.display());
            println!("detail: {}", result.detail);
        }
    }

    Ok(())
}

fn run_guest(command: GuestCommand, config: &PortConfig) -> Result<()> {
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
            match port_runtime::execute_guest_operation(
                config,
                GuestRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    operation: GuestOperation::Pty(PtyRequest {
                        command,
                        cols: 80,
                        rows: 24,
                    }),
                },
            )? {
                OperationResult::Pty(result) => {
                    print!("{}", result.transcript);
                }
                other => bail!("unexpected guest pty result: {other:?}"),
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
            match port_runtime::execute_guest_operation(
                config,
                GuestRequest {
                    machine_name: &machine,
                    runtime_root: &runtime_root,
                    operation: GuestOperation::Logs(LogsRequest {
                        path,
                        follow,
                        tail_lines,
                    }),
                },
            )? {
                OperationResult::Logs(result) => {
                    print!("{}", result.contents);
                }
                other => bail!("unexpected guest logs result: {other:?}"),
            }
        }
        GuestCommand::Forward {
            machine,
            runtime_root,
            listen,
            target,
        } => {
            ensure_machine_exists(config, &machine)?;
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
    use clap::Parser;

    use super::{
        ArchitectureArg, ArtifactCommand, Cli, Command, CopyDirectionArg, GuestCommand,
        MachineCommand, ProtectionModeArg, SubstrateArg, render_help,
        render_nested_subcommand_help, render_subcommand_help,
    };

    #[test]
    fn help_includes_primary_surfaces() {
        let help = render_help();
        let guest_help = render_subcommand_help("guest").expect("guest help should exist");

        for keyword in [
            "doctor",
            "artifacts",
            "machine",
            "guest",
            "Linux",
            "macOS",
            "Windows",
            "repository root",
            "generic-linux",
            "AWS",
            "GCP",
            "Azure",
            "PVM",
            "Cloud Hypervisor",
            "Apple Virtualization Framework",
            "Hosted Control",
            "push",
            "pull",
            "Artifact Mobility",
            "foreground host-side proxy",
        ] {
            assert!(help.contains(keyword), "missing help keyword: {keyword}");
        }

        let machine_help = render_subcommand_help("machine").expect("machine help should exist");

        for keyword in ["exec", "copy", "pty", "logs", "forward"] {
            assert!(
                guest_help.contains(keyword),
                "missing guest help keyword: {keyword}"
            );
        }

        let artifact_help = render_nested_subcommand_help(&["artifacts", "push"])
            .expect("artifact help should exist");

        for keyword in ["launch", "list", "status", "stop"] {
            assert!(
                machine_help.contains(keyword),
                "missing machine help keyword: {keyword}"
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
