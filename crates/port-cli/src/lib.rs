use std::fmt::Write as _;
use std::fs;
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
    ExecutionSubstrate, HostConnection, HostedSchedulerPolicy, MachineArchitecture, PortConfig,
    ProtectionMode, PvmHostKitPackage,
};
use port_runtime::{
    ArtifactRequest, ControlPlaneServeRequest, DoctorReport, GuestCopyRequest, GuestForwardRequest,
    GuestRequest, HostedNodeBinding, HostedPvmNodePrepareRequest, LaunchRequest,
    NodeAgentServeRequest,
};
use serde::{Deserialize, Serialize};

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
  port --config examples/port.toml machine monitor --machine demo
  port --config examples/port.toml machine top --machine demo
  port --config examples/port.toml machine stop --machine demo

Guest workflow examples:
  port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'uname -a'
  port --config examples/port.toml guest copy --machine demo --direction host-to-guest --source ./host.txt --destination /workspace/host.txt
  port --config examples/port.toml guest pty --machine demo -- /bin/sh -lc 'printf pty-ok'
  port --config examples/port.toml guest logs --machine demo --path /var/log/port-agent.log --tail-lines 50
  port --config examples/port.toml guest logs --machine demo --path /var/log/port-agent.log --follow
  port --config examples/port.toml guest forward --machine demo --listen 127.0.0.1:8080 --target 127.0.0.1:80
  port --config examples/port.toml guest forward --machine demo --listen unix:/tmp/port-demo.sock --target unix:/var/run/app.sock
  port --config examples/port.toml guest forward --machine demo --listen 127.0.0.1:8081 --target 127.0.0.1:80 --lifecycle detached --name demo-web
Service workflow examples:
  port --config examples/port.toml service secret put --machine cloud-aws --name demo-token --value s3cr3t
  port --config examples/port.toml service apply --machine cloud-aws --host-group aws-builders --name web --kind service --secret API_TOKEN=demo-token -- /app/web --listen :8080
  port --config examples/port.toml service apply --machine cloud-aws --host-group aws-builders --name buildbox --kind sandbox --secret API_TOKEN=demo-token -- /bin/sh -lc 'make test'
  port --config examples/port.toml service list --machine cloud-aws
  port --config examples/port.toml service status --machine cloud-aws --name web
  port --config examples/port.toml service stop --machine cloud-aws --name web
Multi-node hosted service workflow:
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node-b --bind 127.0.0.1:9235 --token node-secret-b
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service secret put --machine cloud-aws --name demo-token --value s3cr3t
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service apply --machine cloud-aws --host-group aws-secondary --name api --kind service --secret API_TOKEN=demo-token -- /bin/sh -lc 'trap '\''exit 0'\'' TERM; while :; do sleep 1; done'
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service list --machine cloud-aws
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service status --machine cloud-aws --name api
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service stop --machine cloud-aws --name api
  Registered nodes publish inventory to the configured control plane before service placement starts.
  `port service list|status|stop` surface the selected node, target host group, scheduler, and runtime state for the stored placement.
Current hosted service limits:
  No autoscaling or rescheduling yet.
  Deterministic-first-fit is the only shipped scheduler policy.
  No broader fleet policy yet.
  Imported inventory exists, but it is still a control-plane-owned state file contract rather than a first-class `port inventory import` command.
  `port guest exec`, `copy`, `pty`, `logs`, and `forward` work against launched Firecracker VMs through the live guest transport.
  `port guest forward` now supports foreground and detached lifecycle modes plus TCP and Unix-socket listeners through the same command family.
  Guest-side `forward --target` addresses still depend on the guest network state. In the sample guest image, bring loopback up before targeting `127.0.0.1`, for example with `port guest exec --machine demo -- /bin/sh -lc 'busybox ifconfig lo up'`.

Platform Support:
  Linux: local Firecracker launch is supported when port doctor passes.
  macOS: the native AVF lane now uses the same `port machine` and `port guest` verbs when `PORT_AVF_LAUNCHER` points at a launcher helper that exposes the runtime guest socket and console log.
  Windows: use WSL or a remote Linux host, then rely on port doctor to confirm whether local launch is supported.
Native macOS AVF workflow:
  port --config examples/port.toml doctor
  PORT_AVF_LAUNCHER=/path/to/port-avf-launcher port --config examples/port.toml machine launch --machine demo-avf
  port --config examples/port.toml machine status --machine demo-avf
  port --config examples/port.toml guest exec --machine demo-avf -- /bin/sh -lc 'uname -a'
  port --config examples/port.toml machine monitor --machine demo-avf
  port --config examples/port.toml machine stop --machine demo-avf
  Firecracker launch stays Linux-only; on non-macOS hosts the AVF lane fails fast with an explicit macOS boundary.
Execution Lanes:
  Firecracker + standard on Linux is the current shipped lane.
  Firecracker + pvm on x86_64 now launches through the hosted control-plane and node-agent path on prepared Linux nodes and still depends on a dedicated host kit plus pvm artifact variants.
  Firecracker + pvm on aarch64 remains research-only until Port has a supportable Firecracker runtime path.
PVM foundation workflow:
  port --config examples/port.toml doctor
  port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts push --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
  port --config examples/port.toml artifacts pull --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
  Read the `pvm:local:x86_64:*` doctor checks as the host-kit gate for a prepared Linux node.
  Local PVM launch still requires a prepared x86_64 Linux host with the patched `firecracker-pvm` binary and the required host boot state.
Hosted prepared-node PVM workflow:
  Copy `examples/port.toml` to `/tmp/port-pvm.toml`, point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`, switch `machines.cloud-generic` to `protection_mode = \"pvm\"`, and point the `x86_64/firecracker/pvm` kernel and guest variants at prepared artifact paths.
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
  PORT_PVM_FIRECRACKER_BINARY=/path/to/firecracker-pvm PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml node-agent serve --node generic-linux-node --bind 127.0.0.1:9234 --token node-secret
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane prepare-pvm-node --control-plane demo --node generic-linux-node --architecture x86-64 --provenance repo-proof --package-name firecracker-pvm-host-kit --package-version 2026.03 --host-kernel-release 6.12.0-port-pvm --firecracker-build v1.12.0-port-pvm
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine launch --machine cloud-generic
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine status --machine cloud-generic
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine stop --machine cloud-generic
  Before `prepare-pvm-node`, the same `cloud-generic` launch is denied because `generic-linux-node` stays `planned`.
  `prepare-pvm-node` writes the imported ready record under `.port/hosted/<control-plane>/imported-inventory.json`, which is the canonical repo-local proof that the node moved from planned to ready.
  Missing `firecracker-pvm`, missing host boot prerequisites, or missing PVM artifact paths fail explicitly; Port does not fall back to the standard Firecracker lane.
  Firecracker + pvm on aarch64 stays research-only; there is no supported `prepare-pvm-node` or launch proof for that architecture.
  Other hosted launch paths still return provider-aware guidance until their runtime lanes ship.
Standard lane preservation:
  port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode standard
  port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode standard
  port --config examples/port.toml machine launch --machine demo
  PVM admission failures must never silently fall back to the standard Firecracker lane.
  Cloud Hypervisor and Apple Virtualization Framework are modeled explicitly as planned lanes.
  The AVF contract keeps the current guest protocol over AVF virtio sockets and uses AVF serial ports for console capture.
Cloud Linux:
  generic-linux, aws, and gcp providers are modeled through the shared config and surfaced by port doctor.
Hosted standard cloud workflow:
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine launch --machine cloud-aws
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
  PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine stop --machine cloud-aws
  Repeat the same hosted control-plane flow for `cloud-generic` with `generic-linux-node` or for `cloud-gcp` with `gcp-linux-node`.
Repository-local hosted standard proof:
  cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip
  cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip
Artifact Mobility:
  `port artifacts build` and `validate` materialize one canonical local variant selected by architecture, substrate, and protection mode.
  `port artifacts push` and `pull` use the selected artifact variant's configured mobility backend.
  The checked-in sample config defaults to the file-backed store for local proofs and cache warming.
Hosted artifact workflow:
  Copy `examples/port.toml` to a temp config, point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`, and switch the selected artifact distribution `push` and `pull` backends from `file-system` to `hosted-api` with that same endpoint.
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
  port --config /tmp/port-hosted-artifacts.toml artifacts build --artifact demo-kernel --architecture native
  PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml artifacts push --artifact demo-kernel --architecture native
  Remove the local artifact path printed by `build` or `push`, then run `PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml artifacts pull --artifact demo-kernel --architecture native`.
  Hosted pushes land in `.port/hosted/<control-plane>/artifacts/...` under the control-plane owner, and hosted auth uses the configured bearer token from `PORT_DEMO_TOKEN`.
  OCI remains follow-on work.
Hosted Control:
  Local Port still owns the shipped direct Linux launch path, and hosted standard-cloud plus prepared-node PVM launch now route through the control plane and node agent.
  `port control-plane serve` now exposes the first live hosted HTTP entrypoint for canonical machine and guest routes and reloads durable fleet state from `.port/hosted/<control-plane>/registered-nodes.json` plus `.port/hosted/<control-plane>/imported-inventory.json`.
  `port node-agent serve` now exposes the node-owned runtime endpoint that serves those internal routes from one hosted node runtime root and refreshes durable node registration plus heartbeat ownership to the configured control plane.
  Hosted demo flow:
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine launch --machine cloud-aws`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine list`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine stop --machine cloud-aws`
    `PORT_DEMO_TOKEN=demo-token port --config examples/port.toml guest exec --machine cloud-aws -- /bin/sh -lc 'uname -a'`
  `cloud-generic` and `cloud-gcp` use that same hosted surface with `generic-linux-node` and `gcp-linux-node`.
  Imported inventory is currently a control-plane-owned state-file contract, not a first-class `port inventory import` command: seed or sync `.port/hosted/<control-plane>/imported-inventory.json`, then inspect it through `port machine list|status`.
  Hosted Port now resolves `machine list|status|stop|monitor|top` through control-plane contracts plus node-agent runtime roots while preserving the current machine and guest vocabulary.
  The sample config now declares `[control_planes.demo]` with endpoint `https://port.example.internal`.
  Hosted auth is modeled explicitly as a bearer token read from `PORT_DEMO_TOKEN` through the `authorization` header.
  `--node-binding <node>=<endpoint>,<token>` remains a bootstrap or debug override when a node cannot self-register yet.
  Remote/cloud sample hosts now use `mode = \"hosted-control-plane\"` and `control_plane = \"demo\"` instead of SSH placeholders, and hosted nodes declare `runtime_root` so the first machine-runtime slice has a concrete node-agent state location.
  `port machine list|status|stop|monitor|top` now show both local runtime-root machines and hosted-control-plane machines; hosted `machine status` surfaces configured, imported, registered, freshness, selected-node, and routing-eligibility detail for each hosted node, and that fleet state survives control-plane restart because it is reloaded from the durable state files.
  Hosted `guest exec|copy|pty|logs` now execute through the live hosted HTTP path to the control plane and node agent while keeping the existing guest protocol unchanged.
  Hosted `guest copy` now streams bytes through the live control-plane and node-agent path.
  Hosted `guest forward` now supports foreground and detached lifecycle modes plus `--list`, `--stop`, and `--name` through the live control-plane and node-agent path.
  Hosted detached forward returns a node-owned listener address and keeps detached lifecycle state under the node runtime root.
  `port machine monitor` and `top` currently inspect node-agent-owned runtime state plus detached forward manifests.
  Current hosted fleet limits: no autoscaling, no broader fleet policy, and no first-class `port inventory import` command yet.
  Hosted `port service secret` and `port service apply|list|status|stop` now execute through the live control-plane and node-agent path while keeping `port service` as the canonical secrets/services/sandboxes surface.
Service Control:
  `port service` is the canonical secrets/services/sandboxes family; `--kind sandbox` keeps sandbox work on the same service surface instead of inventing a second runtime model.
  Managed guest-process `start|list|status|stop` is an internal contract beneath that same surface, not a second hosted-only CLI family.
  Secret values are currently stored as runtime-owned JSON files under the resolved machine runtime root, so treat this as a bootstrap operator workflow rather than a hardened secret backend.
  `port service apply|list|status|stop` now also exposes a canonical runtime-state contract and record path; restart policy, health checks, scheduler policy, and hardened secret backends remain explicit follow-on work.
  `port-sdk` now publishes the supported typed hosted client surface for machine, guest, and service request construction.
  See `docs/pvm.md` for the explicit Firecracker/PVM host-kit contract and the x86_64 keep versus aarch64 research-only decision.
  See `docs/avf.md` for the AVF launch, guest-transport, serial-console, entitlement, and Rosetta workflow contract.
  Azure remains an explicitly unsupported Firecracker provider lane.";

#[derive(Debug, Parser)]
#[command(
    name = "port",
    version,
    about = "CLI-first Firecracker orchestration for local and cloud Linux hosts",
    long_about = "Port manages microVM-backed workloads through one canonical CLI and shared machine model. Firecracker with standard protection on Linux is the current execution lane; Firecracker/PVM, Cloud Hypervisor, and Apple Virtualization Framework are modeled explicitly and surfaced through shared machine and guest verbs; remote generic-linux, AWS, and GCP hosts are surfaced through provider-aware diagnostics; macOS and Windows operators use the same CLI while substrate-specific prerequisites stay explicit.",
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

impl From<ServiceKindArg> for port_runtime::ServiceKind {
    fn from(value: ServiceKindArg) -> Self {
        match value {
            ServiceKindArg::Service => Self::Service,
            ServiceKindArg::Sandbox => Self::Sandbox,
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

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Doctor { format } => doctor(format, cli.config.as_deref()),
        Command::Artifacts(command) => {
            let config = load_config(cli.config)?;
            run_artifacts(command, &config)
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
        },
    }
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
            let status = port_runtime::machine_status(&config, &runtime_root, &machine)?;
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
            let result = port_runtime::stop_machine(
                &config,
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

fn print_hosted_fleet_nodes(nodes: &[port_runtime::HostedFleetNodeStatus]) {
    let rendered = format_hosted_fleet_nodes(nodes);
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
        "firecracker log: {}",
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
    println!("firecracker log: {}", report.firecracker_log.display());
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
    let state_dir = port_runtime::guest_forward_state_dir(config, machine, runtime_root)?;
    let manifest_path = state_dir.join(format!("{name}.json"));
    let bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
    let manifest: DetachedForwardManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse '{}'", manifest_path.display()))?;

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
    fs::remove_file(&manifest_path)
        .with_context(|| format!("failed to remove '{}'", manifest_path.display()))?;

    println!("forward name: {}", manifest.name);
    println!("forward lifecycle: detached");
    println!("forward state: stopped");
    println!("forward pid: {}", manifest.pid);
    Ok(())
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
    use std::path::PathBuf;

    use clap::Parser;

    use super::{
        ArchitectureArg, ArtifactCommand, Cli, Command, ControlPlaneCommand, CopyDirectionArg,
        GuestCommand, HostedNodeBindingArg, MachineCommand, NodeAgentCommand, ProtectionModeArg,
        ServiceCommand, ServiceKindArg, ServiceSecretCommand, SubstrateArg,
        format_hosted_fleet_nodes, format_machine_status, render_help,
        render_nested_subcommand_help, render_subcommand_help,
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
            "machine",
            "guest",
            "guest pty",
            "--follow",
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
            "Hosted standard cloud workflow",
            "Repository-local hosted standard proof",
            "push",
            "pull",
            "service",
            "control-plane",
            "prepare-pvm-node",
            "node-agent",
            "Artifact Mobility",
            "research-only",
            "detached lifecycle modes",
            "node-owned listener",
            "--list",
            "--stop",
            "--name",
            "node-binding",
            "bootstrap or debug",
            "machine list",
            "Hosted artifact workflow",
            "OCI remains follow-on work",
            "first-class `port inventory import` command",
            "artifacts push --artifact demo-guest",
            "artifacts pull --artifact demo-guest",
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

        for keyword in ["launch", "list", "status", "stop", "monitor", "top"] {
            assert!(
                machine_help.contains(keyword),
                "missing machine help keyword: {keyword}"
            );
        }

        let service_help = render_subcommand_help("service").expect("service help should exist");
        for keyword in ["secret", "apply", "list", "status", "stop", "sandbox"] {
            assert!(
                service_help.contains(keyword),
                "missing service help keyword: {keyword}"
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
                secret,
                command,
            }) => {
                assert_eq!(machine, "cloud-aws");
                assert_eq!(runtime_root, std::path::Path::new("runtime"));
                assert_eq!(name, "api");
                assert_eq!(kind, ServiceKindArg::Service);
                assert_eq!(host_group.as_deref(), Some("aws-builders"));
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
