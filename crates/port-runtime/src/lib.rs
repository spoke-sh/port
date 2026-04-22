use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    CopyDirection, ExecRequest, ExecResult, ForwardEndpoint, GuestOperation, LogsResult,
    ManagedServiceOperation, ManagedServiceRequest, ManagedServiceResult,
    ManagedServiceRuntimeState, ManagedServiceStatus, OperationResult, PtyResult, RequestEnvelope,
    ResponseEnvelope, StreamRequestFrame, StreamResponseFrame, parse_forward_endpoint, read_frame,
    render_forward_endpoint, write_frame,
};
use port_hosted_protocol::{
    HostedArtifactTransferRequest, HostedArtifactTransferResult, HostedDetachedForwardStartRequest,
    HostedDetachedForwardState, HostedDetachedForwardStatus as HostedDetachedForwardStatusContract,
    HostedDetachedForwardStopResult, HostedError, HostedPreparePvmNodeRequest, HostedRouteContext,
    HostedSuccess,
};
use port_model::{
    ArtifactKind, ArtifactReference, ArtifactSelector, ArtifactStore, ArtifactVariant,
    AvfExecutionContract, ClusterSpec, ExecutionSubstrate, HostConnection, HostPlatform,
    HostProvider, HostedApiIdentityContract, HostedArtifactIdentityContract,
    HostedImportedNodeRecord, HostedPvmCapability, HostedPvmHostKitPackageAttachment,
    HostedSchedulerPolicy, K3sClusterSpec, MachineArchitecture, MachineControlContract,
    MachineRootfsOverlaySpec, MachineRuntimeClassSpec, MachineVolumeBackend,
    MachineVolumePersistence, MachineVolumeSpec, OciRegistryAuth, OciRegistryTransport, PortConfig,
    ProtectionMode, PvmCapabilityState, PvmHostKit, PvmHostKitPackage, ServiceHealthState,
    ServiceSecretSourceStatus,
};
use port_sdk::{
    HostedApiRequest, HostedApiStreamRequest, HostedClient, HttpMethod,
    SecretPutRequest as HostedSecretPutRequest, ServiceApplyRequest as HostedServiceApplyRequest,
};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod hosted_control_plane;

pub use port_model::{
    ServiceHealthPolicy, ServiceHealthcheck, ServiceKind, ServicePolicy, ServiceRestartPolicy,
    ServiceSecretBackend, ServiceSecretBinding, ServiceSecretMaterialization,
};

pub use hosted_control_plane::{
    ControlPlaneServeRequest, HostedNodeBinding, NodeAgentServeRequest, serve_control_plane,
    serve_node_agent,
};

const PORT_IPTABLES_BINARY_ENV: &str = "PORT_IPTABLES_BINARY";
// Hosted K3s cold boot can take longer than two minutes before service health
// turns green, especially when the control plane is forming containerd/CNI for
// the first time after a relaunch.
const HOSTED_HTTP_TIMEOUT: Duration = Duration::from_secs(300);
const HOSTED_MACHINE_LIST_TIMEOUT: Duration = Duration::from_secs(30);
const HOSTED_MACHINE_STATUS_TIMEOUT: Duration = Duration::from_secs(15);
const GUEST_TRANSPORT_IO_TIMEOUT: Duration = Duration::from_secs(300);
const GUEST_TRANSPORT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedPvmNodePrepareRequest {
    pub control_plane: String,
    pub node_name: String,
    pub architecture: MachineArchitecture,
    pub provenance: String,
    pub package: PvmHostKitPackage,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterStagedFile {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub bytes_copied: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClusterBootstrapSource {
    configured: PathBuf,
    resolved: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterStageResult {
    pub cluster_name: String,
    pub machine_name: String,
    pub guest_profile: String,
    pub required_commands: Vec<String>,
    pub stage_root: PathBuf,
    pub staged_files: Vec<ClusterStagedFile>,
    pub preflight_command: Vec<String>,
    pub preflight_stdout: String,
    pub install_command: Vec<String>,
    pub install_stdout: String,
    pub installed_binary: PathBuf,
    pub installed_kubectl: PathBuf,
    pub boundary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClusterReadinessState {
    Ready,
    MachineStopped,
    GuestUnavailable,
    Unhealthy,
}

impl std::fmt::Display for ClusterReadinessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => f.write_str("ready"),
            Self::MachineStopped => f.write_str("machine-stopped"),
            Self::GuestUnavailable => f.write_str("guest-unavailable"),
            Self::Unhealthy => f.write_str("unhealthy"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterStatusReport {
    pub cluster_name: String,
    pub machine_name: String,
    pub runtime_dir: PathBuf,
    pub machine_state: MachineRuntimeState,
    pub pid: Option<u32>,
    pub readiness: ClusterReadinessState,
    pub health_command: Vec<String>,
    pub health_output: String,
    pub kubeconfig_path: PathBuf,
    pub kubeconfig_available: bool,
    pub api_forward_target: String,
    pub kubeconfig_surface: String,
    pub boundary: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterUpResult {
    pub cluster_name: String,
    pub machine_name: String,
    pub launch_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch: Option<LaunchMetadata>,
    pub stage: ClusterStageResult,
    pub status: ClusterStatusReport,
    pub boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterRawKubeconfig {
    pub cluster_name: String,
    pub machine_name: String,
    pub kubeconfig_path: PathBuf,
    pub api_forward_target: String,
    pub kubeconfig_surface: String,
    pub kubeconfig: String,
    pub boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterDownResult {
    pub cluster_name: String,
    pub machine_name: String,
    pub stop: StopResult,
    pub boundary: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_class: Option<MachineRuntimeClassSpec>,
    pub attached_volumes: Vec<MachineVolumeSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sBootstrapResult {
    pub cluster_name: String,
    pub control_plane: String,
    pub host_group: String,
    pub server_machines: Vec<String>,
    pub worker_machines: Vec<String>,
    pub api_endpoint: String,
    pub stable_endpoint_posture: HostedK3sStableEndpointPosture,
    pub stable_endpoint_detail: String,
    pub version: String,
    pub join_token: String,
    pub server_launches: Vec<LaunchMetadata>,
    pub worker_launches: Vec<LaunchMetadata>,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sMachineAccess {
    pub role: String,
    pub route: HostedRouteContext,
    pub network_identity: HostedK3sGuestNetworkIdentity,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sGuestNetworkEndpointScope {
    UniquePerGuest,
    SharedPerExecutionHost,
    Unresolved,
}

impl std::fmt::Display for HostedK3sGuestNetworkEndpointScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UniquePerGuest => f.write_str("unique-per-guest"),
            Self::SharedPerExecutionHost => f.write_str("shared-per-execution-host"),
            Self::Unresolved => f.write_str("unresolved"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sGuestNetworkIdentity {
    pub identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_ip: Option<IpAddr>,
    pub endpoint_scope: HostedK3sGuestNetworkEndpointScope,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with_machines: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sMachineTruth {
    pub role: String,
    pub machine_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_root: Option<PathBuf>,
    pub network_identity: HostedK3sGuestNetworkIdentity,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_refresh_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedged_since_unix_s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedge_class: Option<String>,
    #[serde(default, skip_serializing_if = "RecoveryAttemptCounters::is_empty")]
    pub recovery_attempts: RecoveryAttemptCounters,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_recovery_action: Option<RecoveryActionRecord>,
    #[serde(default, skip_serializing_if = "RecoveryState::is_default")]
    pub recovery_state: RecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sControlPlanePlacement {
    pub machine_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_root: Option<PathBuf>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sHaStatus {
    NonHaTopology,
    PendingPlacement,
    SpreadUnsatisfied,
    SpreadSatisfied,
}

impl std::fmt::Display for HostedK3sHaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonHaTopology => f.write_str("non-ha-topology"),
            Self::PendingPlacement => f.write_str("pending-placement"),
            Self::SpreadUnsatisfied => f.write_str("spread-unsatisfied"),
            Self::SpreadSatisfied => f.write_str("spread-satisfied"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sStableEndpointPosture {
    ManualRewriteRequired,
    HaEligible,
}

impl std::fmt::Display for HostedK3sStableEndpointPosture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManualRewriteRequired => f.write_str("manual-rewrite-required"),
            Self::HaEligible => f.write_str("ha-eligible"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sLegacyRuntimeArtifact {
    pub machine_name: String,
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sLegacyRuntimeDriftState {
    Clear,
    DetachedRuntimeDetected,
}

impl std::fmt::Display for HostedK3sLegacyRuntimeDriftState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clear => f.write_str("clear"),
            Self::DetachedRuntimeDetected => f.write_str("detached-runtime-detected"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sManagedServiceTruthState {
    Missing,
    Stored,
    Starting,
    Running,
    Exited,
    Stopped,
    Failed,
    Unreachable,
}

impl std::fmt::Display for HostedK3sManagedServiceTruthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("missing"),
            Self::Stored => f.write_str("stored"),
            Self::Starting => f.write_str("starting"),
            Self::Running => f.write_str("running"),
            Self::Exited => f.write_str("exited"),
            Self::Stopped => f.write_str("stopped"),
            Self::Failed => f.write_str("failed"),
            Self::Unreachable => f.write_str("unreachable"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sManagedServiceTruth {
    pub role: String,
    pub machine_name: String,
    pub service_name: String,
    pub state: HostedK3sManagedServiceTruthState,
    pub restart_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sReadinessState {
    Ready,
    Degraded,
    Unavailable,
}

impl std::fmt::Display for HostedK3sReadinessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => f.write_str("ready"),
            Self::Degraded => f.write_str("degraded"),
            Self::Unavailable => f.write_str("unavailable"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sReadinessGate {
    pub state: HostedK3sReadinessState,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sClusterAccessReport {
    pub cluster_name: String,
    pub control_plane: String,
    pub host_group: String,
    pub server_machines: Vec<String>,
    pub worker_machines: Vec<String>,
    pub api_endpoint: String,
    pub machines: Vec<HostedK3sMachineTruth>,
    pub managed_services: Vec<HostedK3sManagedServiceTruth>,
    pub stable_endpoint_posture: HostedK3sStableEndpointPosture,
    pub stable_endpoint_detail: String,
    pub ha_status: HostedK3sHaStatus,
    pub ha_status_detail: String,
    pub legacy_runtime_drift: HostedK3sLegacyRuntimeDriftState,
    pub legacy_runtime_drift_detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy_runtime_artifacts: Vec<HostedK3sLegacyRuntimeArtifact>,
    pub control_plane_placements: Vec<HostedK3sControlPlanePlacement>,
    pub machine_runtime_readiness: HostedK3sReadinessGate,
    pub api_surface: String,
    pub api_readiness: HostedK3sReadinessGate,
    pub api_output: String,
    pub kubeconfig_surface: String,
    pub kubeconfig_availability: HostedK3sReadinessGate,
    pub kubeconfig: String,
    pub visibility_surface: String,
    pub node_visibility: HostedK3sReadinessGate,
    pub visibility_output: String,
    pub machine_access: Vec<HostedK3sMachineAccess>,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedK3sDownResult {
    pub cluster_name: String,
    pub control_plane: String,
    pub host_group: String,
    pub server_machines: Vec<String>,
    pub worker_machines: Vec<String>,
    pub api_endpoint: String,
    pub server_stops: Vec<StopResult>,
    pub worker_stops: Vec<StopResult>,
    pub boundary_notes: Vec<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_class: Option<MachineRuntimeClassSpec>,
    pub attached_volumes: Vec<MachineVolumeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosted_fleet_nodes: Vec<HostedFleetNodeStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_refresh_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedged_since_unix_s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedge_class: Option<String>,
    #[serde(default, skip_serializing_if = "RecoveryAttemptCounters::is_empty")]
    pub recovery_attempts: RecoveryAttemptCounters,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_recovery_action: Option<RecoveryActionRecord>,
    #[serde(default, skip_serializing_if = "RecoveryState::is_default")]
    pub recovery_state: RecoveryState,
    pub detail: String,
}

/// Cumulative tier attempts within the configured rolling window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryAttemptCounters {
    #[serde(default)]
    pub tier_1: u32,
    #[serde(default)]
    pub tier_2: u32,
    #[serde(default)]
    pub tier_3: u32,
}

impl RecoveryAttemptCounters {
    pub fn is_empty(&self) -> bool {
        self.tier_1 == 0 && self.tier_2 == 0 && self.tier_3 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineWedgeSignal {
    NodeHeartbeatStale,
    GuestHeartbeatStale,
    HostedK3sServiceRuntime,
    CachedDetectorState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineWedgeServiceEvidence {
    pub name: String,
    pub state: ServiceRuntimeState,
    pub health_state: ServiceHealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exit_detail: Option<String>,
}

/// Wedge and recovery state served directly by the control plane on
/// the dedicated `machines/<name>/wedge` route. Populated from the
/// in-memory `wedge_state` map, on-disk recovery records, and a
/// short best-effort hosted K3s service probe when the machine is
/// part of a hosted cluster. Designed for consumers that poll
/// cluster status frequently and do not want to incur a full
/// `MachineStatus` round trip per machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineWedgeStatus {
    pub machine_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_refresh_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedged_since_unix_s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedge_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wedge_signal: Option<MachineWedgeSignal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosted_k3s_service: Option<MachineWedgeServiceEvidence>,
    #[serde(default, skip_serializing_if = "RecoveryAttemptCounters::is_empty")]
    pub recovery_attempts: RecoveryAttemptCounters,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_recovery_action: Option<RecoveryActionRecord>,
    #[serde(default, skip_serializing_if = "RecoveryState::is_default")]
    pub recovery_state: RecoveryState,
}

/// Most recent recovery-ladder transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryActionRecord {
    pub tier: u8,
    pub timestamp_unix_s: u64,
    pub outcome: String,
}

/// Recovery ladder state for a machine. `Disabled` covers both the feature-flag-off
/// case and the absent-config case — they behave identically.
/// `AwaitingTier3HostRecycle` is the terminal signal state: tier-1 and tier-2
/// have exhausted and Port is waiting on an external consumer to recycle the
/// host. Port takes no further action; heartbeat return auto-clears back to Ok.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryState {
    #[default]
    Ok,
    InProgress,
    Disabled,
    AwaitingTier3HostRecycle,
}

impl RecoveryState {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedFleetFreshnessState {
    Live,
    Stale,
    MissingRegistration,
}

impl std::fmt::Display for HostedFleetFreshnessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live => f.write_str("live"),
            Self::Stale => f.write_str("stale"),
            Self::MissingRegistration => f.write_str("missing-registration"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedFleetRoutingEligibility {
    Eligible,
    Rejected,
    MissingRegistration,
    StaleRegistration,
}

impl std::fmt::Display for HostedFleetRoutingEligibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eligible => f.write_str("eligible"),
            Self::Rejected => f.write_str("rejected"),
            Self::MissingRegistration => f.write_str("missing-registration"),
            Self::StaleRegistration => f.write_str("stale-registration"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedFleetNodeStatus {
    pub node_name: String,
    pub configured: bool,
    pub imported: bool,
    pub registered: bool,
    pub selected: bool,
    pub freshness: HostedFleetFreshnessState,
    pub routing_eligibility: HostedFleetRoutingEligibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_at_unix_s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refreshed_at_unix_s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_age_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fresh_until_unix_s: Option<u64>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_class: Option<MachineRuntimeClassSpec>,
    pub attached_volumes: Vec<MachineVolumeSpec>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_class: Option<MachineRuntimeClassSpec>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceRuntimeState {
    Stored,
    Starting,
    Running,
    Exited,
    Stopped,
    Failed,
}

impl std::fmt::Display for ServiceRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stored => f.write_str("stored"),
            Self::Starting => f.write_str("starting"),
            Self::Running => f.write_str("running"),
            Self::Exited => f.write_str("exited"),
            Self::Stopped => f.write_str("stopped"),
            Self::Failed => f.write_str("failed"),
        }
    }
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
    pub host_group: Option<&'a str>,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
    pub policy: ServicePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineSecretSummary {
    pub machine_name: String,
    pub name: String,
    pub backend: ServiceSecretBackend,
    pub materialization: ServiceSecretMaterialization,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub path: PathBuf,
    pub backend_path: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDefinitionStatus {
    pub machine_name: String,
    pub name: String,
    pub kind: ServiceKind,
    pub desired_state: ServiceDesiredState,
    pub runtime: ServiceRuntimeObservation,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
    pub secret_sources: Vec<ServiceSecretSourceStatus>,
    pub policy: ServicePolicy,
    pub control: MachineControlContract,
    pub control_plane: Option<String>,
    pub node_name: Option<String>,
    pub host_groups: Vec<String>,
    pub host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    pub target_host_group: Option<String>,
    pub scheduler: Option<HostedSchedulerPolicy>,
    pub manifest_path: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRuntimeObservation {
    pub state: ServiceRuntimeState,
    pub record_path: PathBuf,
    pub restart_count: u32,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub last_exit_code: Option<i32>,
    pub last_exit_detail: Option<String>,
    pub health_state: ServiceHealthState,
    pub health_detail: Option<String>,
    pub stdout_path: Option<PathBuf>,
    pub stderr_path: Option<PathBuf>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeGuestStorage {
    rootfs_path: PathBuf,
    rootfs_overlay_path: Option<PathBuf>,
}

fn materialize_runtime_guest_storage(
    paths: &RuntimePaths,
    source: &Path,
    rootfs_read_only: bool,
) -> Result<RuntimeGuestStorage> {
    materialize_runtime_guest_storage_with_overlay(paths, source, rootfs_read_only, None)
}

fn materialize_runtime_guest_storage_with_overlay(
    paths: &RuntimePaths,
    source: &Path,
    rootfs_read_only: bool,
    rootfs_overlay: Option<&MachineRootfsOverlaySpec>,
) -> Result<RuntimeGuestStorage> {
    if let Some(rootfs_overlay) = rootfs_overlay {
        let initrd_source = firecracker_initrd_path_for_rootfs(source).with_context(|| {
            format!(
                "guest image '{}' requires a sibling initrd.cpio.gz when booting with a rootfs overlay",
                source.display()
            )
        })?;
        if !initrd_source.is_file() {
            bail!(
                "guest image '{}' requires a sibling initrd.cpio.gz when booting with a rootfs overlay",
                source.display()
            );
        }

        return Ok(RuntimeGuestStorage {
            rootfs_path: source.to_path_buf(),
            rootfs_overlay_path: Some(materialize_runtime_rootfs_overlay(paths, rootfs_overlay)?),
        });
    }

    if rootfs_read_only {
        return Ok(RuntimeGuestStorage {
            rootfs_path: source.to_path_buf(),
            rootfs_overlay_path: None,
        });
    }

    let file_name = source.file_name().with_context(|| {
        format!(
            "guest image path '{}' must reference a file name for runtime materialization",
            source.display()
        )
    })?;
    let destination = paths.runtime_dir.join(file_name);
    if source != destination {
        ensure_runtime_materialized_copy(source, &destination)?;
    }

    if let Some(initrd_source) = firecracker_initrd_path_for_rootfs(source) {
        let initrd_destination = destination.with_file_name("initrd.cpio.gz");
        if initrd_source != initrd_destination {
            ensure_runtime_materialized_copy(&initrd_source, &initrd_destination)?;
        }
    }

    Ok(RuntimeGuestStorage {
        rootfs_path: destination,
        rootfs_overlay_path: None,
    })
}

fn configure_detached_session(command: &mut Command) {
    // Keep long-lived local runtime processes alive across `nix develop --command`
    // boundaries by moving them into a fresh session before exec.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterStageRequest<'a> {
    pub cluster_name: &'a str,
    pub runtime_root: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterUpRequest<'a> {
    pub cluster_name: &'a str,
    pub runtime_root: &'a Path,
    pub boot_wait: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterStatusRequest<'a> {
    pub cluster_name: &'a str,
    pub runtime_root: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterDownRequest<'a> {
    pub cluster_name: &'a str,
    pub runtime_root: &'a Path,
    pub stop_wait: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineDriverKind {
    FirecrackerLocal,
    CloudHypervisorLocal,
    AvfLocal,
    HostedControlPlane,
    SshManagedRemote,
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
struct CloudHypervisorLocalDriver;

#[derive(Debug, Default, Clone, Copy)]
struct AvfLocalDriver;

#[derive(Debug, Default, Clone, Copy)]
struct HostedControlPlaneDriver;

#[derive(Debug, Default, Clone, Copy)]
struct SshManagedDriver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAction {
    Build,
    Validate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactPipelineIo {
    Inherit,
    Capture,
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
    pub backend_detail: String,
    pub bytes_copied: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactAvailabilityState {
    Local,
    LocalAndCache,
    CacheOnly,
    Missing,
}

impl std::fmt::Display for ArtifactAvailabilityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Local => "local",
            Self::LocalAndCache => "local+cache",
            Self::CacheOnly => "cache-only",
            Self::Missing => "missing",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactVariantInventory {
    pub selector: ArtifactSelector,
    pub path: PathBuf,
    pub local_present: bool,
    pub cache_path: PathBuf,
    pub cache_present: bool,
    pub availability: ArtifactAvailabilityState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactInventoryRecord {
    pub name: String,
    pub kind: ArtifactKind,
    pub reference: ArtifactReference,
    pub build_command: String,
    pub validate_command: String,
    pub variants: Vec<ArtifactVariantInventory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArtifactStoreContract {
    FileSystem {
        store_path: PathBuf,
    },
    OciRegistry {
        oras_binary: PathBuf,
        remote_reference: String,
        store_path: PathBuf,
        transport: OciRegistryTransport,
        auth: OciRegistryAuth,
    },
    HostedApi {
        identity: HostedArtifactIdentityContract,
        transfer: HostedArtifactTransferRequest,
    },
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
    let iptables_binary = iptables_binary();
    checks.push(versioned_binary_check(
        "iptables",
        &iptables_binary,
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
            if let ArtifactStore::OciRegistry { auth, transport } = &artifact.distribution.push {
                checks.push(oci_registry_dependency_check(name, "push", *transport));
                checks.push(oci_registry_auth_check(name, "push", auth));
            }
            if let ArtifactStore::OciRegistry { auth, transport } = &artifact.distribution.pull {
                checks.push(oci_registry_dependency_check(name, "pull", *transport));
                checks.push(oci_registry_auth_check(name, "pull", auth));
            }
        }

        for (name, host) in &config.hosts {
            if let Some(check) = provider_check(name, host.provider, &host.connection) {
                checks.push(check);
            }
            checks.extend(ssh_connection_checks(name, host.provider, &host.connection));
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
            checks.extend(attached_volume_doctor_checks(
                name,
                &machine.host,
                host,
                machine,
            ));
            checks.extend(cloud_hypervisor_machine_checks(name, host, machine, facts));
            checks.extend(avf_machine_checks(name, host, machine, facts));
        }
    }

    let mut notes = vec![
        String::from("port doctor reports the host state without mutating runtime directories."),
        String::from(
            "macOS operators can run Port against the AVF lane locally; Firecracker and Firecracker/PVM still require Linux host access.",
        ),
        String::from(
            "Windows operators should use WSL or a remote Linux host, then rely on port doctor to confirm whether local Firecracker launch is available.",
        ),
        String::from(
            "SSH-managed remote hosts surface separate auth and bootstrap expectations; they do not reuse hosted control-plane bearer-token auth or local runtime ownership.",
        ),
    ];
    if config.is_some() {
        notes.push(String::from(
            "Remote Linux hosts are modeled provider-by-provider, but the MVP launch path is still local Linux only.",
        ));
        notes.push(String::from(
            "Firecracker/PVM readiness is reported as a dedicated host-kit lane; failing PVM checks do not imply the standard Firecracker lane is a compatible fallback.",
        ));
        notes.push(String::from(
            "Cloud Hypervisor readiness is reported through its own host-platform, architecture, protection-mode, and binary checks; Port does not silently fall back to Firecracker when that lane is selected.",
        ));
        notes.push(String::from(
            "AVF install and help surfaces stay on the canonical port CLI: set PORT_AVF_LAUNCHER to an external launcher helper for local macOS workflows, and do not expect a bundled macOS-only fallback workflow in this slice.",
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

pub fn list_artifacts(config: &PortConfig) -> Vec<ArtifactInventoryRecord> {
    let mut inventory = Vec::new();
    inventory.extend(
        config
            .artifacts
            .kernels
            .iter()
            .map(|(name, spec)| artifact_inventory_record(name, ArtifactKind::Kernel, spec)),
    );
    inventory.extend(
        config
            .artifacts
            .guest_images
            .iter()
            .map(|(name, spec)| artifact_inventory_record(name, ArtifactKind::GuestImage, spec)),
    );
    inventory
}

pub fn push_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactTransfer> {
    let artifact = resolve_artifact_metadata(config, request)?;
    let (_, spec) = config
        .artifacts
        .lookup_named(&artifact.name)
        .with_context(|| format!("unknown artifact '{}'", artifact.name))?;
    match resolve_artifact_store_contract(config, &spec.distribution.push, &artifact)? {
        ArtifactStoreContract::FileSystem { store_path } => {
            let bytes_copied = copy_file(&artifact.path, &store_path)?;
            let _ = copy_file(&artifact.path, &artifact.cache_path)?;
            Ok(ArtifactTransfer {
                artifact,
                store_path,
                backend_detail: match &spec.distribution.push {
                    ArtifactStore::FileSystem { root } => {
                        format!("filesystem {}", root.display())
                    }
                    other => format!("{other:?}"),
                },
                bytes_copied,
            })
        }
        ArtifactStoreContract::OciRegistry {
            oras_binary,
            store_path,
            remote_reference,
            transport,
            auth,
        } => push_artifact_to_oci_registry_backend(
            artifact,
            oras_binary,
            store_path,
            remote_reference,
            transport,
            auth,
        ),
        ArtifactStoreContract::HostedApi { identity, transfer } => {
            push_artifact_to_hosted_backend(config, artifact, identity, transfer)
        }
    }
}

pub fn pull_artifact(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
) -> Result<ArtifactTransfer> {
    let artifact = resolve_artifact_metadata(config, request)?;
    let (_, spec) = config
        .artifacts
        .lookup_named(&artifact.name)
        .with_context(|| format!("unknown artifact '{}'", artifact.name))?;
    match resolve_artifact_store_contract(config, &spec.distribution.pull, &artifact)? {
        ArtifactStoreContract::FileSystem { store_path } => {
            let bytes_copied = copy_file(&store_path, &artifact.cache_path)?;
            let _ = copy_file(&artifact.cache_path, &artifact.path)?;
            Ok(ArtifactTransfer {
                artifact,
                store_path,
                backend_detail: match &spec.distribution.pull {
                    ArtifactStore::FileSystem { root } => {
                        format!("filesystem {}", root.display())
                    }
                    other => format!("{other:?}"),
                },
                bytes_copied,
            })
        }
        ArtifactStoreContract::OciRegistry {
            oras_binary,
            store_path,
            remote_reference,
            transport,
            auth,
        } => pull_artifact_from_oci_registry_backend(
            artifact,
            oras_binary,
            store_path,
            remote_reference,
            transport,
            auth,
        ),
        ArtifactStoreContract::HostedApi { identity, transfer } => {
            pull_artifact_from_hosted_backend(config, artifact, identity, transfer)
        }
    }
}

fn push_artifact_to_hosted_backend(
    config: &PortConfig,
    artifact: ArtifactMetadata,
    identity: HostedArtifactIdentityContract,
    transfer: HostedArtifactTransferRequest,
) -> Result<ArtifactTransfer> {
    let client = HostedClient::from_control_plane_env(config, &identity.control_plane)?;
    let request = client.artifacts().push(transfer.clone())?;
    let source = File::open(&artifact.path).with_context(|| {
        format!(
            "failed to open local artifact '{}' for hosted push",
            artifact.path.display()
        )
    })?;
    let response = execute_hosted_stream_request(request, reqwest::blocking::Body::new(source))?;
    let uploaded: HostedSuccess<HostedArtifactTransferResult> =
        response.json().with_context(|| {
            format!(
                "failed to decode hosted artifact push response for '{}'",
                artifact.path.display()
            )
        })?;
    let _ = copy_file(&artifact.path, &artifact.cache_path)?;
    Ok(ArtifactTransfer {
        artifact,
        store_path: uploaded.result.store_path,
        backend_detail: format!(
            "hosted-api {} (control-plane {})",
            identity.endpoint, identity.control_plane
        ),
        bytes_copied: uploaded.result.bytes_copied,
    })
}

fn push_artifact_to_oci_registry_backend(
    artifact: ArtifactMetadata,
    oras_binary: PathBuf,
    store_path: PathBuf,
    remote_reference: String,
    transport: OciRegistryTransport,
    auth: OciRegistryAuth,
) -> Result<ArtifactTransfer> {
    fs::metadata(&artifact.path).with_context(|| {
        format!(
            "failed to inspect local artifact '{}' before OCI push to '{}'",
            artifact.path.display(),
            remote_reference
        )
    })?;

    let mut command = Command::new(&oras_binary);
    command.arg("push");
    if transport == OciRegistryTransport::PlainHttp {
        command.arg("--plain-http");
    }
    let password = match &auth {
        OciRegistryAuth::Anonymous => None,
        OciRegistryAuth::BasicEnv {
            username_variable,
            password_variable,
        } => {
            let username = env::var(username_variable).with_context(|| {
                format!(
                    "OCI registry backend for artifact '{}' requires env:{} before pushing '{}'",
                    artifact.name, username_variable, remote_reference
                )
            })?;
            let password = env::var(password_variable).with_context(|| {
                format!(
                    "OCI registry backend for artifact '{}' requires env:{} before pushing '{}'",
                    artifact.name, password_variable, remote_reference
                )
            })?;
            command
                .arg("--username")
                .arg(username)
                .arg("--password-stdin");
            Some(password)
        }
    };
    command.arg(&remote_reference);
    command.arg(format!(
        "{}:{}",
        artifact.path.display(),
        artifact_oci_layer_media_type(artifact.kind)
    ));
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if password.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to start OCI artifact push for '{}' ({}) from '{}' to '{}' via {} with {} auth",
            artifact.reference,
            format_artifact_selector(artifact.selector),
            artifact.path.display(),
            remote_reference,
            transport.describe(),
            auth.describe()
        )
    })?;
    if let Some(password) = password {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to open oras stdin for OCI password input")?;
        stdin
            .write_all(password.as_bytes())
            .context("failed to write OCI registry password to oras stdin")?;
        stdin
            .write_all(b"\n")
            .context("failed to terminate OCI registry password input")?;
    }
    let output = child.wait_with_output().with_context(|| {
        format!(
            "failed to wait for OCI artifact push of '{}' to '{}'",
            artifact.reference, remote_reference
        )
    })?;
    if !output.status.success() {
        bail!(
            "OCI artifact push failed for '{}' ({}) from '{}' into cache '{}' via {} with {} auth and remote '{}' using '{}'; status {}; stderr: {}; stdout: {}",
            artifact.reference,
            format_artifact_selector(artifact.selector),
            artifact.path.display(),
            artifact.cache_path.display(),
            transport.describe(),
            auth.describe(),
            remote_reference,
            oras_binary.display(),
            output.status,
            summarize_process_output(&output.stderr),
            summarize_process_output(&output.stdout)
        );
    }

    let bytes_copied = copy_file(&artifact.path, &artifact.cache_path).with_context(|| {
        format!(
            "failed to refresh cache '{}' after OCI push to '{}'",
            artifact.cache_path.display(),
            remote_reference
        )
    })?;

    Ok(ArtifactTransfer {
        artifact,
        store_path,
        backend_detail: format!(
            "oci-registry {} {} via {}",
            transport.describe(),
            auth.describe(),
            oras_binary.display()
        ),
        bytes_copied,
    })
}

fn pull_artifact_from_hosted_backend(
    config: &PortConfig,
    artifact: ArtifactMetadata,
    identity: HostedArtifactIdentityContract,
    transfer: HostedArtifactTransferRequest,
) -> Result<ArtifactTransfer> {
    let client = HostedClient::from_control_plane_env(config, &identity.control_plane)?;
    let request = client.artifacts().pull(transfer.clone())?;
    let response = execute_hosted_request(request)?;
    let bytes_copied = copy_reader_to_path(response, &artifact.cache_path)?;
    let _ = copy_file(&artifact.cache_path, &artifact.path)?;
    Ok(ArtifactTransfer {
        artifact,
        store_path: transfer.store_path,
        backend_detail: format!(
            "hosted-api {} (control-plane {})",
            identity.endpoint, identity.control_plane
        ),
        bytes_copied,
    })
}

fn pull_artifact_from_oci_registry_backend(
    artifact: ArtifactMetadata,
    oras_binary: PathBuf,
    store_path: PathBuf,
    remote_reference: String,
    transport: OciRegistryTransport,
    auth: OciRegistryAuth,
) -> Result<ArtifactTransfer> {
    let scratch_dir = oci_pull_scratch_dir(&artifact);
    fs::create_dir_all(&scratch_dir).with_context(|| {
        format!(
            "failed to create OCI pull staging directory '{}'",
            scratch_dir.display()
        )
    })?;

    let mut command = Command::new(&oras_binary);
    command.arg("pull");
    if transport == OciRegistryTransport::PlainHttp {
        command.arg("--plain-http");
    }
    let password = match &auth {
        OciRegistryAuth::Anonymous => None,
        OciRegistryAuth::BasicEnv {
            username_variable,
            password_variable,
        } => {
            let username = env::var(username_variable).with_context(|| {
                format!(
                    "OCI registry backend for artifact '{}' requires env:{} before pulling '{}'",
                    artifact.name, username_variable, remote_reference
                )
            })?;
            let password = env::var(password_variable).with_context(|| {
                format!(
                    "OCI registry backend for artifact '{}' requires env:{} before pulling '{}'",
                    artifact.name, password_variable, remote_reference
                )
            })?;
            command
                .arg("--username")
                .arg(username)
                .arg("--password-stdin");
            Some(password)
        }
    };
    command
        .arg("--output")
        .arg(&scratch_dir)
        .arg(&remote_reference);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if password.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to start OCI artifact pull for '{}' ({}) into '{}' from '{}' via {} with {} auth",
            artifact.reference,
            format_artifact_selector(artifact.selector),
            artifact.path.display(),
            remote_reference,
            transport.describe(),
            auth.describe()
        )
    })?;
    if let Some(password) = password {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to open oras stdin for OCI password input")?;
        stdin
            .write_all(password.as_bytes())
            .context("failed to write OCI registry password to oras stdin")?;
        stdin
            .write_all(b"\n")
            .context("failed to terminate OCI registry password input")?;
    }
    let output = child.wait_with_output().with_context(|| {
        format!(
            "failed to wait for OCI artifact pull of '{}' from '{}'",
            artifact.reference, remote_reference
        )
    })?;
    if !output.status.success() {
        bail!(
            "OCI artifact pull failed for '{}' ({}) into cache '{}' and local path '{}' via {} with {} auth and remote '{}' using '{}'; status {}; stderr: {}; stdout: {}",
            artifact.reference,
            format_artifact_selector(artifact.selector),
            artifact.cache_path.display(),
            artifact.path.display(),
            transport.describe(),
            auth.describe(),
            remote_reference,
            oras_binary.display(),
            output.status,
            summarize_process_output(&output.stderr),
            summarize_process_output(&output.stdout)
        );
    }

    let staged_path = locate_oci_pulled_artifact(&scratch_dir, &artifact.path).with_context(|| {
        format!(
            "OCI artifact pull for '{}' ({}) expected '{}' in staging directory '{}' after pulling '{}' via {} with {} auth",
            artifact.reference,
            format_artifact_selector(artifact.selector),
            artifact.path.display(),
            scratch_dir.display(),
            remote_reference,
            transport.describe(),
            auth.describe()
        )
    })?;

    let bytes_copied = copy_file(&staged_path, &artifact.cache_path).with_context(|| {
        format!(
            "failed to materialize cache '{}' from OCI pull staging '{}'",
            artifact.cache_path.display(),
            staged_path.display()
        )
    })?;
    copy_file(&artifact.cache_path, &artifact.path).with_context(|| {
        format!(
            "failed to restore local artifact '{}' from cache '{}'",
            artifact.path.display(),
            artifact.cache_path.display()
        )
    })?;
    let _ = fs::remove_dir_all(&scratch_dir);

    Ok(ArtifactTransfer {
        artifact,
        store_path,
        backend_detail: format!(
            "oci-registry {} {} via {}",
            transport.describe(),
            auth.describe(),
            oras_binary.display()
        ),
        bytes_copied,
    })
}

pub fn launch_local_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    driver_for_machine(config, request.machine_name)?.launch(config, request)
}

fn validate_machine_runtime_launch_config(config: &PortConfig) -> Result<()> {
    let mut effective = config.clone();
    effective.k3s_clusters.clear();
    effective
        .validate()
        .map_err(|error| anyhow!("invalid port config: {error}"))
}

pub fn up_local_cluster(
    config: &PortConfig,
    request: ClusterUpRequest<'_>,
) -> Result<ClusterUpResult> {
    let cluster = load_local_cluster(config, request.cluster_name)?;
    let current_status = local_cluster_machine_status(config, &cluster, request.runtime_root)?;
    let (launch_action, launch) = if current_status.state == MachineRuntimeState::Running {
        (String::from("already-running"), None)
    } else {
        ensure_local_cluster_guest_image_ready(config, request.cluster_name, &cluster)?;
        (
            String::from("launched"),
            Some(launch_local_machine(
                config,
                &LaunchRequest {
                    machine_name: &cluster.machine,
                    runtime_root: request.runtime_root,
                    boot_wait: request.boot_wait,
                },
            )?),
        )
    };

    wait_for_local_cluster_guest_control(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
    )?;
    let stage = stage_local_cluster_bootstrap(
        config,
        ClusterStageRequest {
            cluster_name: request.cluster_name,
            runtime_root: request.runtime_root,
        },
    )?;
    let status = local_cluster_status(
        config,
        ClusterStatusRequest {
            cluster_name: request.cluster_name,
            runtime_root: request.runtime_root,
        },
    )?;
    if status.readiness != ClusterReadinessState::Ready {
        bail!(
            "cluster '{}' did not become ready after bootstrap: {} {}",
            request.cluster_name,
            status.detail,
            status.boundary
        );
    }

    Ok(ClusterUpResult {
        cluster_name: request.cluster_name.to_string(),
        machine_name: cluster.machine.clone(),
        launch_action,
        launch,
        stage,
        status,
        boundary: cluster_boundary_note().to_string(),
    })
}

pub fn local_cluster_status(
    config: &PortConfig,
    request: ClusterStatusRequest<'_>,
) -> Result<ClusterStatusReport> {
    let cluster = load_local_cluster(config, request.cluster_name)?;
    let machine_status = local_cluster_machine_status(config, &cluster, request.runtime_root)?;
    let runtime_dir = machine_status.runtime_dir.clone();
    if machine_status.state != MachineRuntimeState::Running {
        return Ok(ClusterStatusReport {
            cluster_name: request.cluster_name.to_string(),
            machine_name: cluster.machine.clone(),
            runtime_dir,
            machine_state: machine_status.state,
            pid: machine_status.pid,
            readiness: ClusterReadinessState::MachineStopped,
            health_command: cluster.lifecycle.health_command.clone(),
            health_output: String::new(),
            kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
            kubeconfig_available: false,
            api_forward_target: cluster.lifecycle.api_forward_target.clone(),
            kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
            boundary: cluster_boundary_note().to_string(),
            detail: format!(
                "machine '{}' is '{}'; Port cluster readiness is not satisfied until the local machine is running. Downstream bootstrap and broader networking remain out of scope.",
                cluster.machine, machine_status.state
            ),
        });
    }

    if let Err(error) = wait_for_local_cluster_guest_control(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
    ) {
        return Ok(ClusterStatusReport {
            cluster_name: request.cluster_name.to_string(),
            machine_name: cluster.machine.clone(),
            runtime_dir,
            machine_state: machine_status.state,
            pid: machine_status.pid,
            readiness: ClusterReadinessState::GuestUnavailable,
            health_command: cluster.lifecycle.health_command.clone(),
            health_output: String::new(),
            kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
            kubeconfig_available: false,
            api_forward_target: cluster.lifecycle.api_forward_target.clone(),
            kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
            boundary: cluster_boundary_note().to_string(),
            detail: format!(
                "machine '{}' is running but the guest control path is unavailable: {error}",
                cluster.machine
            ),
        });
    }

    let health_output = match execute_cluster_exec(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        cluster.lifecycle.health_command.clone(),
        "inspect local cluster readiness",
    ) {
        Ok(result) => result.stdout.trim_end().to_string(),
        Err(error) => {
            return Ok(ClusterStatusReport {
                cluster_name: request.cluster_name.to_string(),
                machine_name: cluster.machine.clone(),
                runtime_dir,
                machine_state: machine_status.state,
                pid: machine_status.pid,
                readiness: ClusterReadinessState::Unhealthy,
                health_command: cluster.lifecycle.health_command.clone(),
                health_output: String::new(),
                kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
                kubeconfig_available: false,
                api_forward_target: cluster.lifecycle.api_forward_target.clone(),
                kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
                boundary: cluster_boundary_note().to_string(),
                detail: format!(
                    "Port launched the local machine but the cluster health command failed: {error}",
                ),
            });
        }
    };

    let kubeconfig = match read_local_cluster_kubeconfig_raw(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster,
    ) {
        Ok(kubeconfig) => kubeconfig,
        Err(error) => {
            return Ok(ClusterStatusReport {
                cluster_name: request.cluster_name.to_string(),
                machine_name: cluster.machine.clone(),
                runtime_dir,
                machine_state: machine_status.state,
                pid: machine_status.pid,
                readiness: ClusterReadinessState::Unhealthy,
                health_command: cluster.lifecycle.health_command.clone(),
                health_output,
                kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
                kubeconfig_available: false,
                api_forward_target: cluster.lifecycle.api_forward_target.clone(),
                kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
                boundary: cluster_boundary_note().to_string(),
                detail: format!(
                    "Port confirmed guest-reported node health but could not read kubeconfig '{}': {error}",
                    cluster.lifecycle.kubeconfig_path.display()
                ),
            });
        }
    };

    Ok(ClusterStatusReport {
        cluster_name: request.cluster_name.to_string(),
        machine_name: cluster.machine.clone(),
        runtime_dir,
        machine_state: machine_status.state,
        pid: machine_status.pid,
        readiness: ClusterReadinessState::Ready,
        health_command: cluster.lifecycle.health_command.clone(),
        health_output,
        kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
        kubeconfig_available: !kubeconfig.trim().is_empty(),
        api_forward_target: cluster.lifecycle.api_forward_target.clone(),
        kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
        boundary: cluster_boundary_note().to_string(),
        detail: String::from(
            "Port owns machine launch, guest bootstrap, node-health confirmation, and kubeconfig handoff for this first local cluster slice. Downstream GitOps/bootstrap convergence remains separate work.",
        ),
    })
}

pub fn local_cluster_kubeconfig(
    config: &PortConfig,
    request: ClusterStatusRequest<'_>,
) -> Result<ClusterRawKubeconfig> {
    let cluster = load_local_cluster(config, request.cluster_name)?;
    let status = local_cluster_status(config, request.clone())?;
    if status.readiness != ClusterReadinessState::Ready {
        bail!(
            "cluster '{}' is not ready for kubeconfig handoff: {} {}",
            request.cluster_name,
            status.detail,
            status.boundary
        );
    }

    let kubeconfig = read_local_cluster_kubeconfig_raw(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster,
    )?;

    Ok(ClusterRawKubeconfig {
        cluster_name: request.cluster_name.to_string(),
        machine_name: cluster.machine.clone(),
        kubeconfig_path: cluster.lifecycle.kubeconfig_path.clone(),
        api_forward_target: cluster.lifecycle.api_forward_target.clone(),
        kubeconfig_surface: cluster_kubeconfig_surface(request.cluster_name),
        kubeconfig,
        boundary: cluster_boundary_note().to_string(),
    })
}

pub fn down_local_cluster(
    config: &PortConfig,
    request: ClusterDownRequest<'_>,
) -> Result<ClusterDownResult> {
    let cluster = load_local_cluster(config, request.cluster_name)?;
    let paths = RuntimePaths::for_machine(request.runtime_root, &cluster.machine);
    let stop = if !paths.runtime_dir.exists() {
        StopResult {
            machine_name: cluster.machine.clone(),
            previous_state: MachineRuntimeState::Stopped,
            current_state: MachineRuntimeState::Stopped,
            pid: None,
            control: MachineControlContract::local_runtime_root(),
            runtime_dir: paths.runtime_dir,
            runtime_class: None,
            attached_volumes: Vec::new(),
            detail: String::from("cluster machine was already stopped; no runtime state existed"),
        }
    } else {
        stop_machine(
            config,
            request.runtime_root,
            &cluster.machine,
            request.stop_wait,
        )?
    };

    Ok(ClusterDownResult {
        cluster_name: request.cluster_name.to_string(),
        machine_name: cluster.machine,
        stop,
        boundary: cluster_boundary_note().to_string(),
    })
}

pub fn stage_local_cluster_bootstrap(
    config: &PortConfig,
    request: ClusterStageRequest<'_>,
) -> Result<ClusterStageResult> {
    let cluster = config
        .clusters
        .get(request.cluster_name)
        .with_context(|| format!("cluster '{}' not found in config", request.cluster_name))?;
    let install_script_name = cluster
        .bootstrap
        .install_script
        .file_name()
        .with_context(|| {
            format!(
                "cluster '{}' bootstrap install_script '{}' must reference a file",
                request.cluster_name,
                cluster.bootstrap.install_script.display()
            )
        })?;
    let binary_name = cluster.bootstrap.binary.file_name().with_context(|| {
        format!(
            "cluster '{}' bootstrap binary '{}' must reference a file",
            request.cluster_name,
            cluster.bootstrap.binary.display()
        )
    })?;
    let install_script_destination = cluster.bootstrap.stage_root.join(install_script_name);
    let binary_destination = cluster.bootstrap.stage_root.join(binary_name);
    let install_script_source =
        resolve_cluster_bootstrap_source(&cluster.bootstrap.install_script)?;
    let binary_source = resolve_cluster_bootstrap_source(&cluster.bootstrap.binary)?;
    let mut staged_files = vec![
        stage_cluster_bootstrap_file(
            config,
            request.runtime_root,
            request.cluster_name,
            &cluster.machine,
            &install_script_source,
            &install_script_destination,
        )?,
        stage_cluster_bootstrap_file(
            config,
            request.runtime_root,
            request.cluster_name,
            &cluster.machine,
            &binary_source,
            &binary_destination,
        )?,
    ];
    staged_files.extend(stage_cluster_bootstrap_runtime_dependencies(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        &binary_source,
    )?);

    execute_cluster_exec(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        vec![
            String::from("/bin/sh"),
            String::from("-lc"),
            format!(
                "chmod 0755 {} {}",
                shell_single_quote(&guest_shell_path(&install_script_destination)),
                shell_single_quote(&guest_shell_path(&binary_destination))
            ),
        ],
        "mark staged cluster bootstrap inputs executable",
    )?;

    let preflight_command = cluster_preflight_command(
        &cluster.bootstrap.guest_profile.required_commands,
        &install_script_destination,
        &binary_destination,
    );
    let preflight_stdout = execute_cluster_exec(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        preflight_command.clone(),
        "verify the staged cluster bootstrap kit and guest profile",
    )?
    .stdout;

    let install_command = cluster_install_command(cluster, &install_script_destination);
    let mut install_stdout = execute_cluster_exec(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        install_command.clone(),
        "run the offline cluster bootstrap proof",
    )?
    .stdout;
    let installed_binary = cluster.bootstrap.stage_root.join("bin").join("k3s");
    let installed_kubectl = cluster.bootstrap.stage_root.join("bin").join("kubectl");
    let install_validation = execute_cluster_exec(
        config,
        request.runtime_root,
        request.cluster_name,
        &cluster.machine,
        cluster_install_validation_command(&installed_binary, &installed_kubectl),
        "verify the offline cluster bootstrap outputs",
    )?
    .stdout;
    install_stdout.push_str(&install_validation);

    Ok(ClusterStageResult {
        cluster_name: request.cluster_name.to_string(),
        machine_name: cluster.machine.clone(),
        guest_profile: cluster.bootstrap.guest_profile.name.clone(),
        required_commands: cluster.bootstrap.guest_profile.required_commands.clone(),
        stage_root: cluster.bootstrap.stage_root.clone(),
        staged_files,
        preflight_command,
        preflight_stdout,
        install_command,
        install_stdout,
        installed_binary,
        installed_kubectl,
        boundary: String::from(
            "staged Port-owned bootstrap inputs only; cluster lifecycle, health, and kubeconfig remain follow-on work",
        ),
    })
}

fn stage_cluster_bootstrap_file(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
    source: &ClusterBootstrapSource,
    destination: &Path,
) -> Result<ClusterStagedFile> {
    let result = copy_guest_file(
        config,
        GuestCopyRequest {
            machine_name,
            runtime_root,
            source: &source.resolved,
            destination,
            direction: CopyDirection::HostToGuest,
        },
    )
    .with_context(|| {
        format!(
            "failed to stage '{}' into '{}' for cluster '{}'",
            source.configured.display(),
            destination.display(),
            cluster_name
        )
    })?;
    Ok(ClusterStagedFile {
        source: source.configured.clone(),
        destination: destination.to_path_buf(),
        bytes_copied: result.bytes_copied,
    })
}

fn resolve_cluster_bootstrap_source(source: &Path) -> Result<ClusterBootstrapSource> {
    if source.is_absolute() {
        return Ok(ClusterBootstrapSource {
            configured: source.to_path_buf(),
            resolved: source.to_path_buf(),
        });
    }

    let candidates = cluster_bootstrap_source_candidates(source);
    if let Some(path) = candidates.iter().find(|candidate| candidate.is_file()) {
        return Ok(ClusterBootstrapSource {
            configured: source.to_path_buf(),
            resolved: path.clone(),
        });
    }

    let searched = candidates
        .iter()
        .map(|candidate| format!("'{}'", candidate.display()))
        .collect::<Vec<_>>()
        .join(", ");
    bail!(
        "failed to resolve cluster bootstrap source '{}'; searched {}",
        source.display(),
        searched
    )
}

fn cluster_bootstrap_source_candidates(source: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if source.is_absolute() {
        candidates.push(source.to_path_buf());
        return candidates;
    }

    if let Some(configured) = env::var_os("PORT_SHARE_ROOT") {
        push_cluster_bootstrap_candidate(&mut candidates, PathBuf::from(configured).join(source));
    }

    if let Some(configured) = env::var_os("PORT_REPO_ROOT") {
        push_cluster_bootstrap_candidate(&mut candidates, PathBuf::from(configured).join(source));
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(prefix_root) = current_exe.parent().and_then(Path::parent) {
            push_cluster_bootstrap_candidate(
                &mut candidates,
                prefix_root.join("share/port").join(source),
            );
            push_cluster_bootstrap_candidate(&mut candidates, prefix_root.join(source));
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        for candidate in current_dir.ancestors() {
            push_cluster_bootstrap_candidate(&mut candidates, candidate.join(source));
        }
    }

    if cfg!(debug_assertions) {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if let Some(candidate) = manifest_dir.parent().and_then(Path::parent) {
            push_cluster_bootstrap_candidate(&mut candidates, candidate.join(source));
        }
    }

    candidates
}

fn push_cluster_bootstrap_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn stage_cluster_bootstrap_runtime_dependencies(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
    source: &ClusterBootstrapSource,
) -> Result<Vec<ClusterStagedFile>> {
    let dependencies = binary_runtime_dependencies(&source.resolved)?;
    let mut staged = Vec::new();
    for dependency in dependencies {
        staged.push(stage_cluster_bootstrap_file(
            config,
            runtime_root,
            cluster_name,
            machine_name,
            &ClusterBootstrapSource {
                configured: source.configured.clone(),
                resolved: dependency.clone(),
            },
            &dependency,
        )?);
    }
    Ok(staged)
}

fn binary_runtime_dependencies(source: &Path) -> Result<Vec<PathBuf>> {
    if !cfg!(target_os = "linux") {
        return Ok(Vec::new());
    }

    let mut header = [0u8; 4];
    let mut file = File::open(source).with_context(|| {
        format!(
            "failed to open '{}' for dependency inspection",
            source.display()
        )
    })?;
    if file.read(&mut header)? < header.len() || &header != b"\x7fELF" {
        return Ok(Vec::new());
    }

    let output = Command::new("ldd").arg(source).output().with_context(|| {
        format!(
            "failed to inspect runtime dependencies for '{}'",
            source.display()
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ldd failed for '{}': {}", source.display(), stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut dependencies = Vec::new();
    for token in stdout.split_whitespace() {
        if token.starts_with('/') {
            let path = PathBuf::from(token);
            if path.is_file() && !dependencies.iter().any(|existing| existing == &path) {
                dependencies.push(path);
            }
        }
    }
    Ok(dependencies)
}

fn guest_shell_path(path: &Path) -> String {
    path.strip_prefix(Path::new("/"))
        .unwrap_or(path)
        .display()
        .to_string()
}

fn execute_cluster_exec(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
    command: Vec<String>,
    action: &str,
) -> Result<ExecResult> {
    if driver_for_machine(config, machine_name)?.kind() == MachineDriverKind::HostedControlPlane {
        return execute_hosted_k3s_exec(
            config,
            runtime_root,
            machine_name,
            command,
            action,
            cluster_name,
        )
        .with_context(|| {
            format!(
                "failed to {} on machine '{}' for cluster '{}'",
                action, machine_name, cluster_name
            )
        });
    }

    let operation = execute_guest_operation(
        config,
        GuestRequest {
            machine_name,
            runtime_root,
            operation: GuestOperation::Exec(ExecRequest {
                command,
                cwd: Some(String::from("/")),
                env: Default::default(),
            }),
        },
    )
    .with_context(|| {
        format!(
            "failed to {} on machine '{}' for cluster '{}'",
            action, machine_name, cluster_name
        )
    })?;
    match operation {
        OperationResult::Exec(result) => Ok(result),
        other => bail!(
            "cluster '{}' expected exec output from machine '{}' while trying to {}, received {other:?}",
            cluster_name,
            machine_name,
            action
        ),
    }
}

fn cluster_preflight_command(
    required_commands: &[String],
    install_script_destination: &Path,
    binary_destination: &Path,
) -> Vec<String> {
    let mut script = String::from("set -eu;");
    for command in required_commands {
        script.push_str(&format!(
            " command -v {} >/dev/null;",
            shell_single_quote(command)
        ));
        script.push_str(&format!(
            " printf 'required-command:%s\\n' {};",
            shell_single_quote(command)
        ));
    }
    script.push_str(&format!(
        " test -x {};",
        shell_single_quote(&guest_shell_path(install_script_destination))
    ));
    script.push_str(&format!(
        " printf 'staged-file:%s\\n' {};",
        shell_single_quote(&install_script_destination.display().to_string())
    ));
    script.push_str(&format!(
        " test -x {};",
        shell_single_quote(&guest_shell_path(binary_destination))
    ));
    script.push_str(&format!(
        " printf 'staged-file:%s\\n' {};",
        shell_single_quote(&binary_destination.display().to_string())
    ));
    script.push_str(" printf 'guest-profile-ok\\n';");

    vec![String::from("/bin/sh"), String::from("-lc"), script]
}

fn cluster_install_command(
    cluster: &ClusterSpec,
    install_script_destination: &Path,
) -> Vec<String> {
    let install_bin_dir = cluster.bootstrap.stage_root.join("bin");
    let mut script = format!(
        "set -eu; PORT_K3S_BIN_DIR={} PORT_K3S_KUBECONFIG_PATH={} {} server",
        shell_single_quote(&guest_shell_path(&install_bin_dir)),
        shell_single_quote(&guest_shell_path(&cluster.lifecycle.kubeconfig_path)),
        shell_single_quote(&guest_shell_path(install_script_destination)),
    );
    for arg in &cluster.args {
        script.push(' ');
        script.push_str(&shell_single_quote(arg));
    }

    vec![String::from("/bin/sh"), String::from("-lc"), script]
}

fn cluster_install_validation_command(
    installed_binary: &Path,
    installed_kubectl: &Path,
) -> Vec<String> {
    let mut script = String::from("set -eu;");
    script.push_str(&format!(
        " test -x {};",
        shell_single_quote(&guest_shell_path(installed_binary))
    ));
    script.push_str(&format!(
        " printf 'installed-binary:%s\\n' {};",
        shell_single_quote(&installed_binary.display().to_string())
    ));
    script.push_str(&format!(
        " test -L {};",
        shell_single_quote(&guest_shell_path(installed_kubectl))
    ));
    script.push_str(&format!(
        " printf 'installed-kubectl:%s\\n' {};",
        shell_single_quote(&installed_kubectl.display().to_string())
    ));

    vec![String::from("/bin/sh"), String::from("-lc"), script]
}

fn load_local_cluster(config: &PortConfig, cluster_name: &str) -> Result<ClusterSpec> {
    config
        .clusters
        .get(cluster_name)
        .cloned()
        .ok_or_else(|| anyhow!("cluster '{}' not found in config", cluster_name))
}

fn local_cluster_machine_status(
    config: &PortConfig,
    cluster: &ClusterSpec,
    runtime_root: &Path,
) -> Result<MachineStatus> {
    let paths = RuntimePaths::for_machine(runtime_root, &cluster.machine);
    if !paths.runtime_dir.exists() {
        return Ok(synthetic_machine_status(
            &cluster.machine,
            &paths,
            MachineControlContract::local_runtime_root(),
            MachineRuntimeState::Stopped,
            format!(
                "cluster machine '{}' has not been launched beneath '{}'",
                cluster.machine,
                runtime_root.display()
            ),
        ));
    }
    machine_status(config, runtime_root, &cluster.machine)
}

fn read_local_cluster_kubeconfig_raw(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    cluster: &ClusterSpec,
) -> Result<String> {
    let output = execute_cluster_exec(
        config,
        runtime_root,
        cluster_name,
        &cluster.machine,
        vec![
            String::from("/bin/sh"),
            String::from("-lc"),
            format!(
                "cat {}",
                shell_single_quote(&guest_shell_path(&cluster.lifecycle.kubeconfig_path))
            ),
        ],
        "read local cluster kubeconfig",
    )?
    .stdout;
    let kubeconfig = output.trim_end().to_string();
    if kubeconfig.trim().is_empty() {
        bail!(
            "cluster '{}' returned an empty kubeconfig from '{}'",
            cluster_name,
            cluster.lifecycle.kubeconfig_path.display()
        );
    }
    Ok(kubeconfig)
}

fn ensure_local_cluster_guest_image_ready(
    config: &PortConfig,
    cluster_name: &str,
    cluster: &ClusterSpec,
) -> Result<()> {
    let machine = config
        .machines
        .get(&cluster.machine)
        .with_context(|| format!("unknown machine '{}'", cluster.machine))?;
    let selector = ArtifactRequest {
        name: &machine.guest_image,
        architecture: machine.architecture,
        substrate: machine.substrate,
        protection_mode: machine.protection_mode,
    };
    ensure_repo_managed_local_cluster_artifact_ready(
        config,
        ArtifactRequest {
            name: &machine.kernel,
            ..selector
        },
        cluster_name,
        "kernel",
    )?;
    ensure_repo_managed_local_cluster_artifact_ready(
        config,
        selector,
        cluster_name,
        "guest image",
    )?;

    Ok(())
}

fn ensure_repo_managed_local_cluster_artifact_ready(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
    cluster_name: &str,
    artifact_label: &str,
) -> Result<()> {
    let artifact = resolve_artifact_metadata(config, request)?;
    if !uses_repo_managed_artifact_pipeline(&artifact.path) {
        return Ok(());
    }

    if run_artifact_pipeline_quiet(config, request, ArtifactAction::Validate).is_ok() {
        return Ok(());
    }

    run_artifact_pipeline_quiet(config, request, ArtifactAction::Build).with_context(|| {
        format!(
            "failed to rebuild {artifact_label} '{}' for local cluster '{}'",
            artifact.name, cluster_name
        )
    })?;
    run_artifact_pipeline_quiet(config, request, ArtifactAction::Validate).with_context(|| {
        format!(
            "{artifact_label} '{}' for local cluster '{}' remained invalid after rebuild",
            artifact.name, cluster_name
        )
    })?;

    Ok(())
}

fn uses_repo_managed_artifact_pipeline(path: &Path) -> bool {
    for relative_root in [Path::new("artifacts/kernel"), Path::new("artifacts/guest")] {
        if path.starts_with(relative_root) {
            return true;
        }
    }

    if let Ok(root) = repo_root() {
        for relative_root in [Path::new("artifacts/kernel"), Path::new("artifacts/guest")] {
            if path.starts_with(root.join(relative_root)) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
fn uses_repo_managed_guest_image_pipeline(path: &Path) -> bool {
    let relative_root = Path::new("artifacts/guest");
    if path.starts_with(relative_root) {
        return true;
    }

    repo_root()
        .map(|root| path.starts_with(root.join(relative_root)))
        .unwrap_or(false)
}

fn wait_for_local_cluster_guest_control(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> Result<()> {
    const SOCKET_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
    const SOCKET_WAIT_INTERVAL: Duration = Duration::from_millis(20);

    let paths = RuntimePaths::for_machine(runtime_root, machine_name);
    let started = Instant::now();
    let mut last_error = String::from("guest control probe did not run");
    while started.elapsed() < SOCKET_WAIT_TIMEOUT {
        match execute_cluster_exec(
            config,
            runtime_root,
            cluster_name,
            machine_name,
            vec![
                String::from("/bin/sh"),
                String::from("-lc"),
                String::from("true"),
            ],
            "probe local cluster guest control",
        ) {
            Ok(_) => return Ok(()),
            Err(error) => {
                last_error = error.to_string();
                thread::sleep(SOCKET_WAIT_INTERVAL);
            }
        }
    }

    bail!(
        "guest control did not become ready for machine '{}' within {:?} (runtime socket: '{}', vsock transport: '{}'): {}",
        machine_name,
        SOCKET_WAIT_TIMEOUT,
        paths.guest_agent_socket.display(),
        paths.vsock_path.display(),
        last_error
    )
}

fn cluster_boundary_note() -> &'static str {
    "Port readiness in this slice covers local machine launch, guest bootstrap, guest-reported node visibility, and kubeconfig handoff. Downstream GitOps/bootstrap convergence, richer networking, and multi-node expansion remain follow-on work."
}

fn cluster_kubeconfig_surface(cluster_name: &str) -> String {
    format!("port cluster kubeconfig --cluster {cluster_name} --runtime-root <runtime-root>")
}

fn hosted_k3s_boundary_notes() -> Vec<String> {
    vec![
        String::from(
            "Hosted K3s requires one hosted control plane and one host group with live placement capacity in this slice.",
        ),
        String::from(
            "Hosted K3s remains stateless in this slice; attached volumes, persistent storage, and CSI are out of scope.",
        ),
        String::from(
            "Real HA requires at least three control-plane microVMs, a stable HTTPS api endpoint that fronts them, and placement across distinct execution hosts; Port models the endpoint but does not ship the load balancer, VIP, or ingress layer itself in this slice.",
        ),
    ]
}

fn hosted_k3s_topology_note(cluster: &K3sClusterSpec) -> String {
    match cluster.ha_topology_posture() {
        port_model::HostedK3sHaTopologyPosture::NonHaTopology => format!(
            "This config declares {} control-plane microVM{} with scheduler '{}'; Port classifies that topology as non-HA until at least {} control-plane microVMs and spread scheduling are both present.",
            cluster.server_machines.len(),
            if cluster.server_machines.len() == 1 {
                ""
            } else {
                "s"
            },
            cluster.control_plane_scheduler,
            port_model::HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES
        ),
        port_model::HostedK3sHaTopologyPosture::HaEligibleTopology => format!(
            "This config declares {} control-plane microVMs with spread scheduling; Port treats it as HA-eligible topology, but runtime truth still depends on keeping quorum across distinct execution hosts behind the configured HTTPS api endpoint.",
            cluster.server_machines.len()
        ),
    }
}

fn hosted_k3s_scheduler_note(cluster: &K3sClusterSpec) -> String {
    match cluster.control_plane_scheduler {
        HostedSchedulerPolicy::DeterministicFirstFit => String::from(
            "Control-plane scheduler 'deterministic-first-fit' will reuse the earliest eligible execution host when capacity matches.",
        ),
        HostedSchedulerPolicy::Spread => String::from(
            "Control-plane scheduler 'spread' requires newly placed control-plane microVMs to land on distinct eligible execution hosts and fails placement instead of collapsing onto an already used host.",
        ),
    }
}

fn hosted_k3s_runtime_ha_note(
    cluster: &K3sClusterSpec,
    machine_access: &[HostedK3sMachineAccess],
) -> Option<String> {
    let placements = hosted_k3s_control_plane_placements(machine_access);
    if placements.is_empty() {
        None
    } else {
        Some(hosted_k3s_ha_status_detail(cluster, &placements))
    }
}

fn hosted_k3s_report_boundary_notes(
    cluster: &K3sClusterSpec,
    machine_access: &[HostedK3sMachineAccess],
) -> Vec<String> {
    let mut notes = hosted_k3s_boundary_notes();
    notes.push(hosted_k3s_topology_note(cluster));
    notes.push(hosted_k3s_scheduler_note(cluster));
    if let Some(note) = hosted_k3s_runtime_ha_note(cluster, machine_access) {
        notes.push(note);
    }
    notes
}

fn hosted_k3s_cluster_boundary_notes(cluster: &K3sClusterSpec) -> Vec<String> {
    let mut notes = hosted_k3s_boundary_notes();
    notes.push(hosted_k3s_topology_note(cluster));
    notes.push(hosted_k3s_scheduler_note(cluster));
    notes
}

fn hosted_k3s_control_plane_placements(
    machine_access: &[HostedK3sMachineAccess],
) -> Vec<HostedK3sControlPlanePlacement> {
    machine_access
        .iter()
        .filter(|machine| machine.role == "control-plane")
        .map(|machine| HostedK3sControlPlanePlacement {
            machine_name: machine
                .route
                .machine_name
                .clone()
                .unwrap_or_else(|| String::from("(unknown)")),
            node_name: machine.route.node_name.clone(),
            runtime_root: machine.route.runtime_root.clone(),
            detail: machine.detail.clone(),
        })
        .collect()
}

fn hosted_k3s_machine_truth(
    config: &PortConfig,
    machine_access: &[HostedK3sMachineAccess],
) -> Vec<HostedK3sMachineTruth> {
    let network_identities = hosted_k3s_machine_network_identities(config, machine_access);
    let mut machines = machine_access
        .iter()
        .map(|machine| {
            let machine_name = machine
                .route
                .machine_name
                .clone()
                .unwrap_or_else(|| String::from("(unknown)"));
            // Best-effort wedge enrichment: the dedicated control-plane
            // route may do a short hosted K3s service probe. If it
            // fails, leave wedge fields at serde defaults and let the
            // existing managed_services row carry the unreachable
            // signal.
            let wedge = hosted_control_plane_machine_wedge(config, &machine_name).ok();
            HostedK3sMachineTruth {
                role: machine.role.clone(),
                machine_name: machine_name.clone(),
                node_name: machine.route.node_name.clone(),
                runtime_root: machine.route.runtime_root.clone(),
                network_identity: network_identities
                    .get(&machine_name)
                    .cloned()
                    .unwrap_or_else(|| machine.network_identity.clone()),
                detail: machine.detail.clone(),
                // The wedge route can surface guest freshness, but the
                // cluster aggregate intentionally keeps that field off
                // this row until a consumer needs it alongside the
                // existing wedge metadata.
                guest_refresh_age_seconds: None,
                wedged_since_unix_s: wedge.as_ref().and_then(|w| w.wedged_since_unix_s),
                wedge_class: wedge.as_ref().and_then(|w| w.wedge_class.clone()),
                recovery_attempts: wedge
                    .as_ref()
                    .map(|w| w.recovery_attempts)
                    .unwrap_or_default(),
                last_recovery_action: wedge.as_ref().and_then(|w| w.last_recovery_action.clone()),
                recovery_state: wedge.as_ref().map(|w| w.recovery_state).unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    machines.sort_by(|left, right| {
        left.role
            .cmp(&right.role)
            .then(left.machine_name.cmp(&right.machine_name))
    });
    machines
}

fn hosted_k3s_service_role(role: &str) -> Option<&'static str> {
    match role {
        "control-plane" => Some("server"),
        "worker" => Some("agent"),
        _ => None,
    }
}

fn hosted_k3s_managed_service_truth_state(
    state: ServiceRuntimeState,
) -> HostedK3sManagedServiceTruthState {
    match state {
        ServiceRuntimeState::Stored => HostedK3sManagedServiceTruthState::Stored,
        ServiceRuntimeState::Starting => HostedK3sManagedServiceTruthState::Starting,
        ServiceRuntimeState::Running => HostedK3sManagedServiceTruthState::Running,
        ServiceRuntimeState::Exited => HostedK3sManagedServiceTruthState::Exited,
        ServiceRuntimeState::Stopped => HostedK3sManagedServiceTruthState::Stopped,
        ServiceRuntimeState::Failed => HostedK3sManagedServiceTruthState::Failed,
    }
}

fn hosted_k3s_managed_service_truth(
    config: &PortConfig,
    runtime_root: &Path,
    machine_access: &[HostedK3sMachineAccess],
) -> Vec<HostedK3sManagedServiceTruth> {
    let mut services = Vec::new();
    for machine in machine_access {
        let Some(service_role) = hosted_k3s_service_role(&machine.role) else {
            continue;
        };
        let machine_name = machine
            .route
            .machine_name
            .clone()
            .unwrap_or_else(|| String::from("(unknown)"));
        let service_name = hosted_k3s_service_name(service_role).to_string();
        let service_truth = match list_machine_services(config, runtime_root, &machine_name) {
            Ok(service_statuses) => match service_statuses
                .into_iter()
                .find(|status| status.name == service_name)
            {
                Some(status) => HostedK3sManagedServiceTruth {
                    role: machine.role.clone(),
                    machine_name: machine_name.clone(),
                    service_name: status.name,
                    state: hosted_k3s_managed_service_truth_state(status.runtime.state),
                    restart_count: status.runtime.restart_count,
                    pid: status.runtime.pid,
                    node_name: status.node_name.or_else(|| machine.route.node_name.clone()),
                    detail: status.detail,
                },
                None => HostedK3sManagedServiceTruth {
                    role: machine.role.clone(),
                    machine_name: machine_name.clone(),
                    service_name: service_name.clone(),
                    state: HostedK3sManagedServiceTruthState::Missing,
                    restart_count: 0,
                    pid: None,
                    node_name: machine.route.node_name.clone(),
                    detail: format!(
                        "Canonical managed service '{}' is not recorded for machine '{}'; hosted cluster status must treat service ownership as missing instead of inferring detached runtime truth.",
                        service_name, machine_name
                    ),
                },
            },
            Err(error) => HostedK3sManagedServiceTruth {
                role: machine.role.clone(),
                machine_name: machine_name.clone(),
                service_name: service_name.clone(),
                state: HostedK3sManagedServiceTruthState::Unreachable,
                restart_count: 0,
                pid: None,
                node_name: machine.route.node_name.clone(),
                detail: format!(
                    "Canonical managed service '{}' could not be inspected for machine '{}': {error}",
                    service_name, machine_name
                ),
            },
        };
        services.push(service_truth);
    }
    services.sort_by(|left, right| {
        left.role
            .cmp(&right.role)
            .then(left.machine_name.cmp(&right.machine_name))
            .then(left.service_name.cmp(&right.service_name))
    });
    services
}

fn hosted_k3s_machine_runtime_readiness(
    machine_access: &[HostedK3sMachineAccess],
    managed_services: &[HostedK3sManagedServiceTruth],
) -> HostedK3sReadinessGate {
    let mut unavailable = Vec::new();
    let mut degraded = Vec::new();

    for machine in machine_access {
        let machine_name = machine.route.machine_name.as_deref().unwrap_or("(unknown)");
        if machine.route.node_name.is_none() || machine.route.runtime_root.is_none() {
            unavailable.push(format!(
                "machine '{machine_name}' is unresolved ({})",
                machine.detail
            ));
        }
    }

    for service in managed_services {
        let rendered = format!(
            "service '{}' on '{}' is {}",
            service.service_name, service.machine_name, service.state
        );
        match service.state {
            HostedK3sManagedServiceTruthState::Running => {}
            HostedK3sManagedServiceTruthState::Stored
            | HostedK3sManagedServiceTruthState::Starting => degraded.push(rendered),
            HostedK3sManagedServiceTruthState::Missing
            | HostedK3sManagedServiceTruthState::Exited
            | HostedK3sManagedServiceTruthState::Stopped
            | HostedK3sManagedServiceTruthState::Failed
            | HostedK3sManagedServiceTruthState::Unreachable => unavailable.push(rendered),
        }
    }

    if unavailable.is_empty() && degraded.is_empty() {
        return hosted_k3s_readiness_gate(
            HostedK3sReadinessState::Ready,
            format!(
                "Hosted machine/runtime readiness is ready: {} machine routes resolved and {} canonical K3s managed services are running.",
                machine_access.len(),
                managed_services.len()
            ),
        );
    }

    if unavailable.is_empty() {
        return hosted_k3s_readiness_gate(
            HostedK3sReadinessState::Degraded,
            format!(
                "Hosted machine/runtime readiness is degraded: {}.",
                degraded.join("; ")
            ),
        );
    }

    hosted_k3s_readiness_gate(
        HostedK3sReadinessState::Unavailable,
        format!(
            "Hosted machine/runtime readiness is unavailable: {}.",
            unavailable
                .into_iter()
                .chain(degraded)
                .collect::<Vec<_>>()
                .join("; ")
        ),
    )
}

fn hosted_k3s_ha_status(
    cluster: &K3sClusterSpec,
    placements: &[HostedK3sControlPlanePlacement],
) -> HostedK3sHaStatus {
    if cluster.ha_topology_posture() == port_model::HostedK3sHaTopologyPosture::NonHaTopology {
        return HostedK3sHaStatus::NonHaTopology;
    }
    if placements
        .iter()
        .any(|placement| placement.node_name.is_none())
    {
        return HostedK3sHaStatus::PendingPlacement;
    }

    let distinct_nodes = placements
        .iter()
        .filter_map(|placement| placement.node_name.clone())
        .collect::<BTreeSet<_>>();
    if distinct_nodes.len() < cluster.server_machines.len() {
        HostedK3sHaStatus::SpreadUnsatisfied
    } else {
        HostedK3sHaStatus::SpreadSatisfied
    }
}

fn hosted_k3s_ha_status_detail(
    cluster: &K3sClusterSpec,
    placements: &[HostedK3sControlPlanePlacement],
) -> String {
    let resolved_nodes = placements
        .iter()
        .filter_map(|placement| placement.node_name.clone())
        .collect::<BTreeSet<_>>();
    match hosted_k3s_ha_status(cluster, placements) {
        HostedK3sHaStatus::NonHaTopology => format!(
            "Hosted AWS x86_64 PVM real-HA status is non-ha-topology: this cluster declares {} control-plane microVM{} with scheduler '{}'; Port requires at least {} control-plane microVMs plus spread scheduling before the topology can claim real HA.",
            cluster.server_machines.len(),
            if cluster.server_machines.len() == 1 {
                ""
            } else {
                "s"
            },
            cluster.control_plane_scheduler,
            port_model::HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES
        ),
        HostedK3sHaStatus::PendingPlacement => {
            let unresolved = placements
                .iter()
                .filter(|placement| placement.node_name.is_none())
                .map(|placement| placement.machine_name.clone())
                .collect::<Vec<_>>();
            format!(
                "Hosted AWS x86_64 PVM real-HA status is pending-placement: HA-eligible topology exists, but control-plane placement is still unresolved for {}.",
                unresolved.join(", ")
            )
        }
        HostedK3sHaStatus::SpreadUnsatisfied => format!(
            "Hosted AWS x86_64 PVM real-HA status is spread-unsatisfied: {} control-plane microVMs currently resolve onto {} distinct execution hosts: {}.",
            cluster.server_machines.len(),
            resolved_nodes.len(),
            resolved_nodes.into_iter().collect::<Vec<_>>().join(", ")
        ),
        HostedK3sHaStatus::SpreadSatisfied => format!(
            "Hosted AWS x86_64 PVM real-HA status is spread-satisfied: {} control-plane microVMs currently resolve onto {} distinct execution hosts: {}.",
            cluster.server_machines.len(),
            resolved_nodes.len(),
            resolved_nodes.into_iter().collect::<Vec<_>>().join(", ")
        ),
    }
}

fn hosted_k3s_bootstrap_stable_endpoint_posture(
    cluster: &K3sClusterSpec,
) -> HostedK3sStableEndpointPosture {
    match cluster.ha_topology_posture() {
        port_model::HostedK3sHaTopologyPosture::NonHaTopology => {
            HostedK3sStableEndpointPosture::ManualRewriteRequired
        }
        port_model::HostedK3sHaTopologyPosture::HaEligibleTopology => {
            HostedK3sStableEndpointPosture::HaEligible
        }
    }
}

fn hosted_k3s_access_stable_endpoint_posture(
    ha_status: HostedK3sHaStatus,
) -> HostedK3sStableEndpointPosture {
    match ha_status {
        HostedK3sHaStatus::SpreadSatisfied => HostedK3sStableEndpointPosture::HaEligible,
        HostedK3sHaStatus::NonHaTopology
        | HostedK3sHaStatus::PendingPlacement
        | HostedK3sHaStatus::SpreadUnsatisfied => {
            HostedK3sStableEndpointPosture::ManualRewriteRequired
        }
    }
}

fn hosted_k3s_bootstrap_stable_endpoint_detail(cluster: &K3sClusterSpec) -> String {
    match hosted_k3s_bootstrap_stable_endpoint_posture(cluster) {
        HostedK3sStableEndpointPosture::ManualRewriteRequired => format!(
            "Hosted AWS x86_64 PVM stable endpoint posture is manual-rewrite-required: Port hands off configured api_endpoint '{}' but this topology still declares {} control-plane microVM{} with scheduler '{}'. Losing the selected control-plane guest would still require manual downstream rewrites or unsupported operator intervention.",
            cluster.api_endpoint,
            cluster.server_machines.len(),
            if cluster.server_machines.len() == 1 {
                ""
            } else {
                "s"
            },
            cluster.control_plane_scheduler
        ),
        HostedK3sStableEndpointPosture::HaEligible => format!(
            "Hosted AWS x86_64 PVM stable endpoint posture is ha-eligible: Port hands off configured api_endpoint '{}' as the canonical cluster address for this spread-scheduled topology. Supported failover condition: one control-plane guest replacement or host loss while at least {} spread control-plane microVMs continue backing that endpoint. External LB/DNS ownership remains outside Port.",
            cluster.api_endpoint,
            port_model::HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES
        ),
    }
}

fn hosted_k3s_access_stable_endpoint_detail(
    cluster: &K3sClusterSpec,
    ha_status: HostedK3sHaStatus,
) -> String {
    match hosted_k3s_access_stable_endpoint_posture(ha_status) {
        HostedK3sStableEndpointPosture::ManualRewriteRequired => format!(
            "Hosted AWS x86_64 PVM stable endpoint posture is manual-rewrite-required: Port rewrites kubeconfig to configured api_endpoint '{}' but the current real-HA status is '{}'. Losing the selected control-plane guest would still require manual downstream rewrites or unsupported operator intervention.",
            cluster.api_endpoint, ha_status
        ),
        HostedK3sStableEndpointPosture::HaEligible => format!(
            "Hosted AWS x86_64 PVM stable endpoint posture is ha-eligible: Port rewrites kubeconfig to configured api_endpoint '{}' and the current real-HA status is '{}'. Supported failover condition: one control-plane guest replacement or host loss while at least {} spread control-plane microVMs continue backing that endpoint. External LB/DNS ownership remains outside Port.",
            cluster.api_endpoint,
            ha_status,
            port_model::HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES
        ),
    }
}

fn hosted_k3s_boundary_summary() -> String {
    hosted_k3s_boundary_notes().join(" ")
}

fn load_hosted_k3s_cluster(config: &PortConfig, cluster_name: &str) -> Result<K3sClusterSpec> {
    config.validate().map_err(|error| {
        anyhow!(
            "invalid hosted k3s cluster '{}': {} {}",
            cluster_name,
            error,
            hosted_k3s_boundary_summary()
        )
    })?;
    config
        .k3s_clusters
        .get(cluster_name)
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "unknown hosted k3s cluster '{}'. {}",
                cluster_name,
                hosted_k3s_boundary_summary()
            )
        })
}

fn hosted_k3s_primary_server_machine<'a>(
    cluster_name: &str,
    cluster: &'a K3sClusterSpec,
) -> Result<&'a str> {
    cluster
        .server_machines
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            anyhow!(
                "hosted k3s cluster '{}' has no control-plane machines",
                cluster_name
            )
        })
}

fn hosted_k3s_kubeconfig_surface(machine_name: &str) -> String {
    format_guest_exec_surface(machine_name, &hosted_k3s_kubeconfig_command())
}

fn hosted_k3s_api_surface(machine_name: &str) -> String {
    format_guest_exec_surface(machine_name, &hosted_k3s_api_readiness_command())
}

fn hosted_k3s_visibility_surface(machine_name: &str) -> String {
    format_guest_exec_surface(machine_name, &hosted_k3s_visibility_command())
}

fn format_guest_exec_surface(machine_name: &str, command: &[String]) -> String {
    let rendered = command
        .iter()
        .map(|part| shell_single_quote(part))
        .collect::<Vec<_>>()
        .join(" ");
    format!("port guest exec --machine {machine_name} -- {rendered}")
}

fn hosted_k3s_readiness_gate(
    state: HostedK3sReadinessState,
    detail: impl Into<String>,
) -> HostedK3sReadinessGate {
    HostedK3sReadinessGate {
        state,
        detail: detail.into(),
    }
}

struct HostedK3sProbeResult {
    gate: HostedK3sReadinessGate,
    output: String,
}

fn hosted_k3s_api_readiness_probe(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> HostedK3sProbeResult {
    let surface = hosted_k3s_api_surface(machine_name);
    match execute_hosted_k3s_exec(
        config,
        runtime_root,
        machine_name,
        hosted_k3s_api_readiness_command(),
        "inspect hosted K3s API readiness",
        cluster_name,
    ) {
        Ok(result) => {
            let output = result.stdout.trim_end().to_string();
            if output.trim().is_empty() {
                return HostedK3sProbeResult {
                    gate: hosted_k3s_readiness_gate(
                        HostedK3sReadinessState::Unavailable,
                        format!(
                            "Hosted K3s API readiness is unavailable: server '{machine_name}' returned empty /readyz output through '{surface}'."
                        ),
                    ),
                    output,
                };
            }

            HostedK3sProbeResult {
                gate: hosted_k3s_readiness_gate(
                    HostedK3sReadinessState::Ready,
                    format!(
                        "Hosted K3s API readiness is ready: server '{machine_name}' answered /readyz through '{surface}'."
                    ),
                ),
                output,
            }
        }
        Err(error) => HostedK3sProbeResult {
            gate: hosted_k3s_readiness_gate(
                HostedK3sReadinessState::Unavailable,
                format!(
                    "Hosted K3s API readiness is unavailable: server '{machine_name}' could not answer /readyz through '{surface}': {error}"
                ),
            ),
            output: String::new(),
        },
    }
}

fn hosted_k3s_kubeconfig_probe(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> HostedK3sProbeResult {
    let surface = hosted_k3s_kubeconfig_surface(machine_name);
    match execute_hosted_k3s_exec(
        config,
        runtime_root,
        machine_name,
        hosted_k3s_kubeconfig_command(),
        "read the hosted K3s kubeconfig",
        cluster_name,
    ) {
        Ok(result) => {
            let output = result.stdout.trim_end().to_string();
            if output.trim().is_empty() {
                return HostedK3sProbeResult {
                    gate: hosted_k3s_readiness_gate(
                        HostedK3sReadinessState::Unavailable,
                        format!(
                            "Hosted K3s kubeconfig availability is unavailable: server '{machine_name}' returned an empty kubeconfig through '{surface}'."
                        ),
                    ),
                    output,
                };
            }

            HostedK3sProbeResult {
                gate: hosted_k3s_readiness_gate(
                    HostedK3sReadinessState::Ready,
                    format!(
                        "Hosted K3s kubeconfig availability is ready: server '{machine_name}' returned kubeconfig content through '{surface}'."
                    ),
                ),
                output,
            }
        }
        Err(error) => HostedK3sProbeResult {
            gate: hosted_k3s_readiness_gate(
                HostedK3sReadinessState::Unavailable,
                format!(
                    "Hosted K3s kubeconfig availability is unavailable: server '{machine_name}' could not read '/etc/rancher/k3s/k3s.yaml' through '{surface}': {error}"
                ),
            ),
            output: String::new(),
        },
    }
}

fn hosted_k3s_visibility_probe(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> HostedK3sProbeResult {
    let surface = hosted_k3s_visibility_surface(machine_name);
    match execute_hosted_k3s_exec(
        config,
        runtime_root,
        machine_name,
        hosted_k3s_visibility_command(),
        "inspect hosted K3s node visibility",
        cluster_name,
    ) {
        Ok(result) => {
            let output = result.stdout.trim_end().to_string();
            if output.trim().is_empty() {
                return HostedK3sProbeResult {
                    gate: hosted_k3s_readiness_gate(
                        HostedK3sReadinessState::Unavailable,
                        format!(
                            "Hosted K3s node visibility is unavailable: server '{machine_name}' returned empty node visibility through '{surface}'."
                        ),
                    ),
                    output,
                };
            }

            HostedK3sProbeResult {
                gate: hosted_k3s_readiness_gate(
                    HostedK3sReadinessState::Ready,
                    format!(
                        "Hosted K3s node visibility is ready: server '{machine_name}' returned node visibility through '{surface}'."
                    ),
                ),
                output,
            }
        }
        Err(error) => HostedK3sProbeResult {
            gate: hosted_k3s_readiness_gate(
                HostedK3sReadinessState::Unavailable,
                format!(
                    "Hosted K3s node visibility is unavailable: server '{machine_name}' could not run 'k3s kubectl get nodes -o wide' through '{surface}': {error}"
                ),
            ),
            output: String::new(),
        },
    }
}

fn hosted_k3s_machine_access(
    config: &PortConfig,
    cluster_name: &str,
    control_plane: &str,
    host_group: &str,
    machine_name: &str,
    role: &str,
) -> Result<HostedK3sMachineAccess> {
    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let summary = effective_config
        .hosted_machine_summary_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!(
                "hosted k3s cluster '{}' {} machine '{}' does not resolve to a hosted machine summary. {}",
                cluster_name,
                role,
                machine_name,
                hosted_k3s_boundary_summary()
            )
        })?;

    if summary.control_plane != control_plane {
        bail!(
            "hosted k3s cluster '{}' {} machine '{}' resolved control plane '{}' instead of '{}'. {}",
            cluster_name,
            role,
            machine_name,
            summary.control_plane,
            control_plane,
            hosted_k3s_boundary_summary()
        );
    }
    if !summary.host_groups.iter().any(|group| group == host_group) {
        bail!(
            "hosted k3s cluster '{}' {} machine '{}' is not available through host group '{}'; available groups: {}. {}",
            cluster_name,
            role,
            machine_name,
            host_group,
            if summary.host_groups.is_empty() {
                String::from("(none)")
            } else {
                summary.host_groups.join(", ")
            },
            hosted_k3s_boundary_summary()
        );
    }
    if summary.candidate_nodes.is_empty() {
        bail!(
            "hosted k3s cluster '{}' {} machine '{}' has no hosted placement capacity in host group '{}': {}. {}",
            cluster_name,
            role,
            machine_name,
            host_group,
            summary.placement_detail,
            hosted_k3s_boundary_summary()
        );
    }

    let resolution = hosted_machine_resolution(config, machine_name)?;
    let mut route = HostedRouteContext::from_machine_summary(&summary);
    if let Some(node_name) = resolution.node_name.as_ref() {
        route = route.with_selected_node(node_name.clone(), resolution.runtime_root.clone());
    }
    let detail = match resolution.node_name.as_ref() {
        Some(node_name) => format!(
            "{} host group '{}'; selected node '{}'. {}",
            resolution.status.detail, host_group, node_name, summary.placement_detail
        ),
        None => format!(
            "{} host group '{}'. {}",
            resolution.status.detail, host_group, summary.placement_detail
        ),
    };

    Ok(HostedK3sMachineAccess {
        role: role.to_string(),
        route,
        network_identity: hosted_k3s_default_guest_network_identity(
            &summary.control_plane,
            &resolution,
        ),
        detail,
    })
}

fn hosted_k3s_guest_network_identity_uri(
    control_plane: &str,
    node_name: Option<&str>,
    machine_name: &str,
) -> String {
    match node_name.filter(|value| !value.trim().is_empty()) {
        Some(node_name) => {
            format!("port-hosted://{control_plane}/nodes/{node_name}/machines/{machine_name}")
        }
        None => format!("port-hosted://{control_plane}/machines/{machine_name}"),
    }
}

fn hosted_k3s_default_guest_network_identity(
    control_plane: &str,
    resolution: &HostedMachineResolution,
) -> HostedK3sGuestNetworkIdentity {
    let machine_name = resolution.status.machine_name.as_str();
    let identity = hosted_k3s_guest_network_identity_uri(
        control_plane,
        resolution.node_name.as_deref(),
        machine_name,
    );
    HostedK3sGuestNetworkIdentity {
        identity: identity.clone(),
        endpoint_ip: None,
        endpoint_scope: HostedK3sGuestNetworkEndpointScope::Unresolved,
        shared_with_machines: Vec::new(),
        detail: format!(
            "Hosted guest network identity '{identity}' is unresolved because Port has not derived a current guest-underlay IP for machine '{machine_name}' yet."
        ),
    }
}

fn hosted_k3s_machine_network_identities(
    config: &PortConfig,
    machine_access: &[HostedK3sMachineAccess],
) -> BTreeMap<String, HostedK3sGuestNetworkIdentity> {
    let mut endpoint_ips = BTreeMap::new();
    let mut endpoint_aliases: BTreeMap<IpAddr, Vec<String>> = BTreeMap::new();
    for machine in machine_access {
        let Some(machine_name) = machine.route.machine_name.as_ref() else {
            continue;
        };
        let endpoint_ip = hosted_k3s_machine_external_ip(config, machine_name).unwrap_or_default();
        if let Some(endpoint_ip) = endpoint_ip {
            endpoint_aliases
                .entry(endpoint_ip)
                .or_default()
                .push(machine_name.clone());
        }
        endpoint_ips.insert(machine_name.clone(), endpoint_ip);
    }

    let mut identities = BTreeMap::new();
    for machine in machine_access {
        let machine_name = machine
            .route
            .machine_name
            .clone()
            .unwrap_or_else(|| String::from("(unknown)"));
        let control_plane = machine
            .route
            .control_plane
            .as_deref()
            .unwrap_or("(unknown)");
        let identity = hosted_k3s_guest_network_identity_uri(
            control_plane,
            machine.route.node_name.as_deref(),
            &machine_name,
        );
        let endpoint_ip = endpoint_ips.get(&machine_name).copied().flatten();
        let (endpoint_scope, shared_with_machines, detail) = match endpoint_ip {
            Some(endpoint_ip) => {
                let shared_with_machines = endpoint_aliases
                    .get(&endpoint_ip)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|peer| peer != &machine_name)
                    .collect::<Vec<_>>();
                if shared_with_machines.is_empty() {
                    (
                        HostedK3sGuestNetworkEndpointScope::UniquePerGuest,
                        shared_with_machines,
                        format!(
                            "Hosted guest network identity '{identity}' currently resolves to unique guest-underlay IP '{endpoint_ip}' for machine '{machine_name}'."
                        ),
                    )
                } else {
                    let peers = shared_with_machines.join(", ");
                    (
                        HostedK3sGuestNetworkEndpointScope::SharedPerExecutionHost,
                        shared_with_machines,
                        format!(
                            "Hosted guest network identity '{identity}' currently shares guest-underlay IP '{endpoint_ip}' with machine(s) {peers}. Safe hosted multi-guest networking requires explicit host-side demultiplexing and unique per-guest underlay identity."
                        ),
                    )
                }
            }
            None => (
                HostedK3sGuestNetworkEndpointScope::Unresolved,
                Vec::new(),
                format!(
                    "Hosted guest network identity '{identity}' is unresolved because Port could not derive a current guest-underlay IP for machine '{machine_name}'."
                ),
            ),
        };
        identities.insert(
            machine_name,
            HostedK3sGuestNetworkIdentity {
                identity,
                endpoint_ip,
                endpoint_scope,
                shared_with_machines,
                detail,
            },
        );
    }

    identities
}

pub fn hosted_k3s_kubeconfig_command() -> Vec<String> {
    vec![
        String::from("/bin/sh"),
        String::from("-lc"),
        String::from("cat /etc/rancher/k3s/k3s.yaml"),
    ]
}

pub fn hosted_k3s_visibility_command() -> Vec<String> {
    vec![
        String::from("/bin/sh"),
        String::from("-lc"),
        String::from("k3s kubectl get nodes -o wide"),
    ]
}

pub fn hosted_k3s_api_readiness_command() -> Vec<String> {
    vec![
        String::from("/bin/sh"),
        String::from("-lc"),
        String::from(
            "k3s kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml --request-timeout=10s get --raw=/readyz",
        ),
    ]
}

const HOSTED_K3S_SERVER_LEGACY_RUNTIME_ARTIFACT_PATHS: [&str; 2] =
    ["/run/port/k3s-server.pid", "/var/log/k3s-server.log"];

fn hosted_k3s_legacy_runtime_drift_command() -> Vec<String> {
    let legacy_paths = HOSTED_K3S_SERVER_LEGACY_RUNTIME_ARTIFACT_PATHS
        .iter()
        .map(|path| shell_single_quote(path))
        .collect::<Vec<_>>()
        .join(" ");
    vec![
        String::from("/bin/sh"),
        String::from("-lc"),
        format!(
            "set -eu; for path in {legacy_paths}; do if [ -e \"$path\" ]; then printf '%s\\n' \"$path\"; fi; done"
        ),
    ]
}

fn hosted_k3s_legacy_runtime_artifact_detail(path: &str) -> String {
    format!(
        "Legacy detached K3s server artifact '{}' sits outside the canonical managed-service runtime path (/run/port/services/*).",
        path
    )
}

fn hosted_k3s_legacy_runtime_artifacts(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> Result<Vec<HostedK3sLegacyRuntimeArtifact>> {
    let mut artifacts = Vec::new();
    let output = execute_hosted_k3s_exec(
        config,
        runtime_root,
        machine_name,
        hosted_k3s_legacy_runtime_drift_command(),
        "inspect legacy detached K3s runtime drift",
        cluster_name,
    )?
    .stdout;
    for path in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        artifacts.push(HostedK3sLegacyRuntimeArtifact {
            machine_name: machine_name.to_string(),
            path: path.to_string(),
            detail: hosted_k3s_legacy_runtime_artifact_detail(path),
        });
    }
    artifacts.sort_by(|left, right| {
        left.machine_name
            .cmp(&right.machine_name)
            .then(left.path.cmp(&right.path))
    });
    Ok(artifacts)
}

fn hosted_k3s_legacy_runtime_drift_state(
    artifacts: &[HostedK3sLegacyRuntimeArtifact],
) -> HostedK3sLegacyRuntimeDriftState {
    if artifacts.is_empty() {
        HostedK3sLegacyRuntimeDriftState::Clear
    } else {
        HostedK3sLegacyRuntimeDriftState::DetachedRuntimeDetected
    }
}

fn hosted_k3s_legacy_runtime_drift_detail(artifacts: &[HostedK3sLegacyRuntimeArtifact]) -> String {
    if artifacts.is_empty() {
        return String::from(
            "Hosted AWS x86_64 PVM legacy-runtime drift is clear: no detached K3s server PID/log artifacts were found outside the canonical managed-service runtime path (/run/port/services/*) on the primary control-plane machine.",
        );
    }

    let mut by_machine: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for artifact in artifacts {
        by_machine
            .entry(artifact.machine_name.clone())
            .or_default()
            .push(artifact.path.clone());
    }
    let rendered = by_machine
        .into_iter()
        .map(|(machine_name, paths)| format!("{machine_name}: {}", paths.join(", ")))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Hosted AWS x86_64 PVM legacy-runtime drift is detached-runtime-detected: detached K3s server PID/log artifacts remain outside the canonical managed-service runtime path (/run/port/services/*) on {rendered}."
    )
}

pub fn hosted_k3s_cluster_access(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
) -> Result<HostedK3sClusterAccessReport> {
    let cluster = load_hosted_k3s_cluster(config, cluster_name)?;
    let primary_server = hosted_k3s_primary_server_machine(cluster_name, &cluster)?.to_string();
    let server_access = hosted_k3s_machine_access(
        config,
        cluster_name,
        &cluster.control_plane,
        &cluster.host_group,
        &primary_server,
        "control-plane",
    )?;
    let mut machine_access = vec![server_access];
    for server_machine in cluster.server_machines.iter().skip(1) {
        machine_access.push(hosted_k3s_machine_access(
            config,
            cluster_name,
            &cluster.control_plane,
            &cluster.host_group,
            server_machine,
            "control-plane",
        )?);
    }
    for worker_machine in &cluster.worker_machines {
        machine_access.push(hosted_k3s_machine_access(
            config,
            cluster_name,
            &cluster.control_plane,
            &cluster.host_group,
            worker_machine,
            "worker",
        )?);
    }

    let machines = hosted_k3s_machine_truth(config, &machine_access);
    let managed_services = hosted_k3s_managed_service_truth(config, runtime_root, &machine_access);
    let machine_runtime_readiness =
        hosted_k3s_machine_runtime_readiness(&machine_access, &managed_services);
    let api_probe =
        hosted_k3s_api_readiness_probe(config, runtime_root, cluster_name, &primary_server);
    let kubeconfig_probe =
        hosted_k3s_kubeconfig_probe(config, runtime_root, cluster_name, &primary_server);
    let visibility_probe =
        hosted_k3s_visibility_probe(config, runtime_root, cluster_name, &primary_server);
    let boundary_notes = hosted_k3s_report_boundary_notes(&cluster, &machine_access);
    let control_plane_placements = hosted_k3s_control_plane_placements(&machine_access);
    let ha_status = hosted_k3s_ha_status(&cluster, &control_plane_placements);
    let ha_status_detail = hosted_k3s_ha_status_detail(&cluster, &control_plane_placements);
    let stable_endpoint_posture = hosted_k3s_access_stable_endpoint_posture(ha_status);
    let stable_endpoint_detail = hosted_k3s_access_stable_endpoint_detail(&cluster, ha_status);
    let legacy_runtime_artifacts =
        hosted_k3s_legacy_runtime_artifacts(config, runtime_root, cluster_name, &primary_server)?;
    let legacy_runtime_drift = hosted_k3s_legacy_runtime_drift_state(&legacy_runtime_artifacts);
    let legacy_runtime_drift_detail =
        hosted_k3s_legacy_runtime_drift_detail(&legacy_runtime_artifacts);
    let network_identities = hosted_k3s_machine_network_identities(config, &machine_access);
    for machine in &mut machine_access {
        let machine_name = machine
            .route
            .machine_name
            .clone()
            .unwrap_or_else(|| String::from("(unknown)"));
        if let Some(identity) = network_identities.get(&machine_name) {
            machine.network_identity = identity.clone();
        }
    }

    Ok(HostedK3sClusterAccessReport {
        cluster_name: cluster_name.to_string(),
        control_plane: cluster.control_plane,
        host_group: cluster.host_group,
        server_machines: cluster.server_machines.clone(),
        worker_machines: cluster.worker_machines,
        api_endpoint: cluster.api_endpoint,
        machines,
        managed_services,
        stable_endpoint_posture,
        stable_endpoint_detail,
        ha_status,
        ha_status_detail,
        legacy_runtime_drift,
        legacy_runtime_drift_detail,
        legacy_runtime_artifacts,
        control_plane_placements,
        machine_runtime_readiness,
        api_surface: hosted_k3s_api_surface(&primary_server),
        api_readiness: api_probe.gate,
        api_output: api_probe.output,
        kubeconfig_surface: hosted_k3s_kubeconfig_surface(&primary_server),
        kubeconfig_availability: kubeconfig_probe.gate,
        kubeconfig: kubeconfig_probe.output,
        visibility_surface: hosted_k3s_visibility_surface(&primary_server),
        node_visibility: visibility_probe.gate,
        visibility_output: visibility_probe.output,
        machine_access,
        boundary_notes,
    })
}

fn hosted_k3s_cluster_readiness_summary(report: &HostedK3sClusterAccessReport) -> String {
    format!(
        "machine-runtime={} ({}) ; api={} ({}) ; node-visibility={} ({}) ; kubeconfig={} ({})",
        report.machine_runtime_readiness.state,
        report.machine_runtime_readiness.detail,
        report.api_readiness.state,
        report.api_readiness.detail,
        report.node_visibility.state,
        report.node_visibility.detail,
        report.kubeconfig_availability.state,
        report.kubeconfig_availability.detail
    )
}

pub fn hosted_k3s_cluster_kubeconfig(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
) -> Result<HostedK3sClusterAccessReport> {
    let report = hosted_k3s_cluster_access(config, runtime_root, cluster_name)?;
    if !matches!(
        report.kubeconfig_availability.state,
        HostedK3sReadinessState::Ready
    ) || report.kubeconfig.trim().is_empty()
    {
        bail!(
            "hosted k3s cluster '{}' kubeconfig handoff is unavailable: {}",
            cluster_name,
            hosted_k3s_cluster_readiness_summary(&report)
        );
    }
    Ok(report)
}

pub fn bootstrap_hosted_k3s_cluster(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
) -> Result<HostedK3sBootstrapResult> {
    const K3S_LAUNCH_WAIT: Duration = Duration::from_millis(50);

    let cluster = load_hosted_k3s_cluster(config, cluster_name)?;
    let primary_server = hosted_k3s_primary_server_machine(cluster_name, &cluster)?.to_string();
    let primary_server_launch = launch_local_machine(
        config,
        &LaunchRequest {
            machine_name: &primary_server,
            runtime_root,
            boot_wait: K3S_LAUNCH_WAIT,
        },
    )
    .with_context(|| {
        format!(
            "failed to launch hosted k3s server machine '{}' for cluster '{}'",
            primary_server, cluster_name
        )
    })?;
    let primary_server_args =
        hosted_k3s_effective_args(config, "server", &primary_server, &cluster.server_args)?;

    execute_hosted_k3s_managed_service_start(
        config,
        runtime_root,
        &primary_server,
        &cluster.host_group,
        "server",
        &primary_server_args,
        Some("--cluster-init"),
        None,
        None,
        "bootstrap the K3s server",
        cluster_name,
    )?;

    let join_token =
        wait_for_hosted_k3s_join_token(config, runtime_root, cluster_name, &primary_server)?;

    let mut server_launches = vec![primary_server_launch];
    for server_machine in cluster.server_machines.iter().skip(1) {
        let launch = launch_local_machine(
            config,
            &LaunchRequest {
                machine_name: server_machine,
                runtime_root,
                boot_wait: K3S_LAUNCH_WAIT,
            },
        )
        .with_context(|| {
            format!(
                "failed to launch hosted k3s control-plane machine '{}' for cluster '{}'",
                server_machine, cluster_name
            )
        })?;
        let server_args =
            hosted_k3s_effective_args(config, "server", server_machine, &cluster.server_args)?;
        execute_hosted_k3s_managed_service_start(
            config,
            runtime_root,
            server_machine,
            &cluster.host_group,
            "server",
            &server_args,
            None,
            Some(&cluster.api_endpoint),
            Some(&join_token),
            "join the K3s control-plane node",
            cluster_name,
        )?;
        server_launches.push(launch);
    }

    let mut worker_launches = Vec::with_capacity(cluster.worker_machines.len());
    for worker_machine in &cluster.worker_machines {
        let launch = launch_local_machine(
            config,
            &LaunchRequest {
                machine_name: worker_machine,
                runtime_root,
                boot_wait: K3S_LAUNCH_WAIT,
            },
        )
        .with_context(|| {
            format!(
                "failed to launch hosted k3s worker machine '{}' for cluster '{}'",
                worker_machine, cluster_name
            )
        })?;
        let worker_args =
            hosted_k3s_effective_args(config, "agent", worker_machine, &cluster.worker_args)?;
        execute_hosted_k3s_managed_service_start(
            config,
            runtime_root,
            worker_machine,
            &cluster.host_group,
            "agent",
            &worker_args,
            None,
            Some(&cluster.api_endpoint),
            Some(&join_token),
            "join the K3s worker",
            cluster_name,
        )?;
        worker_launches.push(launch);
    }

    let boundary_notes = hosted_k3s_cluster_boundary_notes(&cluster);
    let stable_endpoint_posture = hosted_k3s_bootstrap_stable_endpoint_posture(&cluster);
    let stable_endpoint_detail = hosted_k3s_bootstrap_stable_endpoint_detail(&cluster);
    Ok(HostedK3sBootstrapResult {
        cluster_name: cluster_name.to_string(),
        control_plane: cluster.control_plane,
        host_group: cluster.host_group,
        server_machines: cluster.server_machines,
        worker_machines: cluster.worker_machines,
        api_endpoint: cluster.api_endpoint,
        stable_endpoint_posture,
        stable_endpoint_detail,
        version: port_model::render_k3s_version_label(cluster.version.as_deref()),
        join_token,
        server_launches,
        worker_launches,
        boundary_notes,
    })
}

pub fn down_hosted_k3s_cluster(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    stop_wait: Duration,
) -> Result<HostedK3sDownResult> {
    let cluster = load_hosted_k3s_cluster(config, cluster_name)?;
    let mut worker_stops = Vec::with_capacity(cluster.worker_machines.len());
    for worker_machine in &cluster.worker_machines {
        worker_stops.push(
            stop_machine(config, runtime_root, worker_machine, stop_wait).with_context(|| {
                format!(
                    "failed to stop hosted k3s worker machine '{}' for cluster '{}'",
                    worker_machine, cluster_name
                )
            })?,
        );
    }

    let mut server_stops = Vec::with_capacity(cluster.server_machines.len());
    for server_machine in cluster.server_machines.iter().rev() {
        server_stops.push(
            stop_machine(config, runtime_root, server_machine, stop_wait).with_context(|| {
                format!(
                    "failed to stop hosted k3s control-plane machine '{}' for cluster '{}'",
                    server_machine, cluster_name
                )
            })?,
        );
    }

    let boundary_notes = hosted_k3s_cluster_boundary_notes(&cluster);
    Ok(HostedK3sDownResult {
        cluster_name: cluster_name.to_string(),
        control_plane: cluster.control_plane,
        host_group: cluster.host_group,
        server_machines: cluster.server_machines,
        worker_machines: cluster.worker_machines,
        api_endpoint: cluster.api_endpoint,
        server_stops,
        worker_stops,
        boundary_notes,
    })
}

fn execute_hosted_k3s_exec(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    command: Vec<String>,
    action: &str,
    cluster_name: &str,
) -> Result<ExecResult> {
    const GUEST_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(10);
    const GUEST_RETRY_TIMEOUT: Duration = Duration::from_secs(15);
    const GUEST_RETRY_INTERVAL: Duration = Duration::from_millis(100);

    let request = ExecRequest {
        command,
        cwd: None,
        env: Default::default(),
    };
    let started = Instant::now();
    let last_error = loop {
        match hosted_control_plane_guest_operation_with_timeout(
            config,
            GuestRequest {
                machine_name,
                runtime_root,
                operation: GuestOperation::Exec(request.clone()),
            },
            GUEST_ATTEMPT_TIMEOUT,
        ) {
            Ok(OperationResult::Exec(result)) => return Ok(result),
            Ok(other) => {
                bail!(
                    "hosted k3s cluster '{}' expected exec result from machine '{}' while trying to {}, received {other:?}",
                    cluster_name,
                    machine_name,
                    action
                );
            }
            Err(error) => {
                if started.elapsed() >= GUEST_RETRY_TIMEOUT
                    || !hosted_k3s_exec_error_is_retryable(&error)
                {
                    break error;
                }
                thread::sleep(GUEST_RETRY_INTERVAL);
            }
        }
    };

    Err(last_error).with_context(|| {
        format!(
            "failed to {} on machine '{}' for hosted k3s cluster '{}'",
            action, machine_name, cluster_name
        )
    })
}

fn hosted_k3s_exec_error_is_retryable(error: &anyhow::Error) -> bool {
    let rendered = error.to_string();
    !(rendered.contains("guest operation failed with exit code")
        || rendered.contains("guest agent returned an error"))
}

#[allow(clippy::too_many_arguments)]
fn execute_hosted_k3s_managed_service_start(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    host_group: &str,
    role: &str,
    args: &[String],
    bootstrap_flag: Option<&str>,
    server_url: Option<&str>,
    join_token: Option<&str>,
    action: &str,
    cluster_name: &str,
) -> Result<ManagedServiceStatus> {
    execute_hosted_k3s_managed_service_start_with_retry(
        config,
        runtime_root,
        machine_name,
        host_group,
        role,
        args,
        bootstrap_flag,
        server_url,
        join_token,
        action,
        cluster_name,
        Duration::from_secs(15),
        Duration::from_secs(5),
        Duration::from_secs(60),
        Duration::from_millis(100),
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_hosted_k3s_managed_service_start_with_retry(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    host_group: &str,
    role: &str,
    args: &[String],
    bootstrap_flag: Option<&str>,
    server_url: Option<&str>,
    join_token: Option<&str>,
    action: &str,
    cluster_name: &str,
    apply_timeout: Duration,
    status_timeout: Duration,
    retry_timeout: Duration,
    retry_interval: Duration,
) -> Result<ManagedServiceStatus> {
    let request = ServiceApplyRequest {
        machine_name,
        runtime_root,
        name: hosted_k3s_service_name(role),
        kind: ServiceKind::Service,
        host_group: Some(host_group),
        command: hosted_k3s_service_command(role, args, bootstrap_flag, server_url, join_token),
        secret_bindings: Vec::new(),
        policy: hosted_k3s_service_policy(role, machine_name),
    };
    let started = Instant::now();
    let last_error = loop {
        match hosted_control_plane_apply_machine_service_with_timeout(
            config,
            request.clone(),
            apply_timeout,
        ) {
            Ok(status) => {
                return Ok(managed_service_status_from_service_definition_status(
                    status,
                ));
            }
            Err(error) => {
                match hosted_control_plane_machine_service_status_with_timeout(
                    config,
                    machine_name,
                    request.name,
                    status_timeout,
                ) {
                    Ok(status)
                        if hosted_service_status_matches_apply_request(&status, &request) =>
                    {
                        return Ok(managed_service_status_from_service_definition_status(
                            status,
                        ));
                    }
                    Ok(_) if started.elapsed() >= retry_timeout => {
                        break anyhow!(
                            "{error}; follow-up status for service '{}' on machine '{}' returned a different live command than the requested hosted k3s bootstrap command",
                            request.name,
                            machine_name
                        );
                    }
                    Err(status_error) if started.elapsed() >= retry_timeout => {
                        break anyhow!(
                            "{error}; follow-up status for service '{}' on machine '{}' also failed: {status_error}",
                            request.name,
                            machine_name
                        );
                    }
                    Ok(_) | Err(_) => {}
                }
                thread::sleep(retry_interval);
            }
        }
    };

    Err(last_error).with_context(|| {
        format!(
            "failed to {} on machine '{}' for hosted k3s cluster '{}'",
            action, machine_name, cluster_name
        )
    })
}

fn hosted_service_status_matches_apply_request(
    status: &ServiceDefinitionStatus,
    request: &ServiceApplyRequest<'_>,
) -> bool {
    status.machine_name == request.machine_name
        && status.name == request.name
        && status.kind == request.kind
        && status.command == request.command
        && status.desired_state == ServiceDesiredState::Active
}

fn managed_service_status_from_service_definition_status(
    status: ServiceDefinitionStatus,
) -> ManagedServiceStatus {
    ManagedServiceStatus {
        name: status.name,
        kind: match status.kind {
            ServiceKind::Service => port_agent_protocol::ManagedServiceKind::Service,
            ServiceKind::Sandbox => port_agent_protocol::ManagedServiceKind::Sandbox,
        },
        state: match status.runtime.state {
            ServiceRuntimeState::Stored => ManagedServiceRuntimeState::Stored,
            ServiceRuntimeState::Starting => ManagedServiceRuntimeState::Starting,
            ServiceRuntimeState::Running => ManagedServiceRuntimeState::Running,
            ServiceRuntimeState::Exited => ManagedServiceRuntimeState::Exited,
            ServiceRuntimeState::Stopped => ManagedServiceRuntimeState::Stopped,
            ServiceRuntimeState::Failed => ManagedServiceRuntimeState::Failed,
        },
        restart_count: status.runtime.restart_count,
        pid: status.runtime.pid,
        exit_code: status.runtime.exit_code,
        last_exit_code: status.runtime.last_exit_code,
        last_exit_detail: status.runtime.last_exit_detail,
        health_state: status.runtime.health_state,
        health_detail: status.runtime.health_detail,
        stdout_path: status
            .runtime
            .stdout_path
            .map(|path| path.display().to_string()),
        stderr_path: status
            .runtime
            .stderr_path
            .map(|path| path.display().to_string()),
        detail: status.detail,
    }
}

fn hosted_k3s_join_token_command() -> Vec<String> {
    vec![
        String::from("/bin/sh"),
        String::from("-lc"),
        String::from(
            "cat /var/lib/rancher/k3s/server/token 2>/dev/null || cat /var/lib/rancher/k3s/server/node-token",
        ),
    ]
}

fn wait_for_hosted_k3s_join_token(
    config: &PortConfig,
    runtime_root: &Path,
    cluster_name: &str,
    machine_name: &str,
) -> Result<String> {
    const JOIN_TOKEN_TIMEOUT: Duration = Duration::from_secs(120);
    const JOIN_TOKEN_INTERVAL: Duration = Duration::from_secs(1);

    let started = Instant::now();
    let mut last_detail = String::from("join token did not appear yet");
    while started.elapsed() < JOIN_TOKEN_TIMEOUT {
        match execute_hosted_k3s_exec(
            config,
            runtime_root,
            machine_name,
            hosted_k3s_join_token_command(),
            "read the K3s join token",
            cluster_name,
        ) {
            Ok(result) => {
                let join_token = result.stdout.trim().to_string();
                if !join_token.is_empty() {
                    return Ok(join_token);
                }
                last_detail = format!(
                    "hosted k3s cluster '{}' returned an empty join token from server '{}'",
                    cluster_name, machine_name
                );
            }
            Err(error) => {
                last_detail = error.to_string();
            }
        }
        thread::sleep(JOIN_TOKEN_INTERVAL);
    }

    bail!(
        "hosted k3s cluster '{}' did not expose a join token on server '{}' within {}s: {}",
        cluster_name,
        machine_name,
        JOIN_TOKEN_TIMEOUT.as_secs(),
        last_detail
    )
}

fn hosted_k3s_effective_args(
    config: &PortConfig,
    role: &str,
    machine_name: &str,
    args: &[String],
) -> Result<Vec<String>> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{}'", machine_name))?;
    let mut effective = args.to_vec();
    let snapshotter_configured = effective
        .iter()
        .any(|arg| arg == "--snapshotter" || arg.starts_with("--snapshotter="));
    if machine.rootfs_overlay.is_some() && !snapshotter_configured {
        effective.push(String::from("--snapshotter=native"));
    }
    let node_name_configured = effective
        .iter()
        .any(|arg| arg == "--node-name" || arg.starts_with("--node-name="));
    if !node_name_configured {
        effective.push(String::from("--node-name"));
        effective.push(machine_name.to_string());
    }
    let node_external_ip_configured = effective
        .iter()
        .any(|arg| arg == "--node-external-ip" || arg.starts_with("--node-external-ip="));
    let flannel_external_ip_configured = effective
        .iter()
        .any(|arg| arg == "--flannel-external-ip" || arg.starts_with("--flannel-external-ip="));
    if let Some(node_external_ip) = hosted_k3s_machine_external_ip(config, machine_name)? {
        if !node_external_ip_configured {
            effective.push(String::from("--node-external-ip"));
            effective.push(node_external_ip.to_string());
        }
        if role == "server" && !flannel_external_ip_configured {
            effective.push(String::from("--flannel-external-ip"));
        }
    }
    Ok(effective)
}

fn hosted_k3s_machine_external_ip(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Option<IpAddr>> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{}'", machine_name))?;
    if let Some(network) = machine.network.as_ref()
        && network.enabled
    {
        return network
            .guest_ip
            .parse()
            .with_context(|| {
                format!(
                    "failed to parse guest underlay IP '{}' for machine '{}'",
                    network.guest_ip, machine_name
                )
            })
            .map(Some);
    }
    hosted_k3s_registered_node_external_ip(config, machine_name)
}

fn hosted_k3s_registered_node_external_ip(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Option<IpAddr>> {
    let Some(placement) = hosted_stored_machine_placement(config, machine_name)? else {
        return Ok(None);
    };
    let hosted_identity = config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    if let Some(state) = read_hosted_registered_node_state(config, &hosted_identity.control_plane)?
        && let Some(registration) = state.nodes.get(&placement.node_name)
    {
        return hosted_endpoint_ip(&registration.endpoint)
            .with_context(|| {
                format!(
                    "failed to derive hosted K3s node external IP for machine '{}' from registered node '{}' endpoint '{}'",
                    machine_name, placement.node_name, registration.endpoint
                )
            })
            .map(Some);
    }
    hosted_imported_node_external_ip(config, &hosted_identity.control_plane, &placement.node_name)
}

fn hosted_imported_node_external_ip(
    config: &PortConfig,
    control_plane: &str,
    node_name: &str,
) -> Result<Option<IpAddr>> {
    let Some(state) = read_hosted_imported_inventory_state(config, control_plane)? else {
        return Ok(None);
    };
    let Some(node) = state.nodes.get(node_name) else {
        return Ok(None);
    };
    hosted_provenance_ip(&node.provenance)
}

fn hosted_provenance_ip(provenance: &str) -> Result<Option<IpAddr>> {
    let trimmed = provenance.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return Ok(Some(ip));
    }
    let Ok(url) = Url::parse(trimmed) else {
        return Ok(None);
    };
    let Some(host) = url.host_str() else {
        return Ok(None);
    };
    match host.parse::<IpAddr>() {
        Ok(ip) => Ok(Some(ip)),
        Err(_) => Ok(None),
    }
}

fn hosted_endpoint_ip(endpoint: &str) -> Result<IpAddr> {
    let parsed = Url::parse(endpoint)
        .with_context(|| format!("failed to parse hosted node endpoint '{}'", endpoint))?;
    let host = parsed.host_str().with_context(|| {
        format!(
            "hosted node endpoint '{}' does not declare a host",
            endpoint
        )
    })?;
    host.parse::<IpAddr>().with_context(|| {
        format!(
            "hosted node endpoint '{}' host '{}' is not an IP literal",
            endpoint, host
        )
    })
}

fn hosted_k3s_service_name(role: &str) -> &'static str {
    match role {
        "server" => "k3s-server",
        "agent" => "k3s-agent",
        _ => "k3s",
    }
}

fn hosted_k3s_service_command(
    role: &str,
    args: &[String],
    bootstrap_flag: Option<&str>,
    server_url: Option<&str>,
    join_token: Option<&str>,
) -> Vec<String> {
    let mut command = vec![String::from("/usr/bin/k3s"), role.to_string()];
    if let Some(bootstrap_flag) = bootstrap_flag {
        command.push(bootstrap_flag.to_string());
    }
    if let Some(server_url) = server_url {
        command.push(String::from("--server"));
        command.push(server_url.to_string());
    }
    if let Some(join_token) = join_token {
        command.push(String::from("--token"));
        command.push(join_token.to_string());
    }
    for arg in args {
        if role == "agent"
            && (arg == "--flannel-external-ip" || arg.starts_with("--flannel-external-ip="))
        {
            continue;
        }
        command.push(arg.clone());
    }
    command
}

const HOSTED_K3S_AGENT_LEASE_MAX_AGE_SECONDS: u64 = 120;
const HOSTED_K3S_AGENT_TRANSIENT_FAILURE_GRACE_SECONDS: u64 = 300;
const HOSTED_K3S_AGENT_BOOTSTRAP_GRACE_SECONDS: u64 = 600;

fn hosted_k3s_service_healthcheck_command(role: &str, machine_name: &str) -> Vec<String> {
    let k3s = "/usr/bin/k3s";
    let busybox = "/bin/busybox";
    let shell = match role {
        "server" => format!(
            "{k3s} crictl info >/dev/null 2>&1 && {k3s} kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml --request-timeout=10s get --raw=/readyz >/dev/null 2>&1"
        ),
        "agent" => format!(
            "state_dir=/run/port/health; \
            last_ok_file=\"$state_dir/k3s-agent-cluster-ok\"; \
            bootstrap_start_file=\"$state_dir/k3s-agent-bootstrap-start\"; \
            mkdir -p \"$state_dir\"; \
            now_epoch=$({busybox} date -u +%s); \
            if [ ! -f \"$bootstrap_start_file\" ]; then \
                printf '%s\n' \"$now_epoch\" > \"$bootstrap_start_file\"; \
            fi; \
            bootstrap_epoch=$(cat \"$bootstrap_start_file\" 2>/dev/null); \
            {k3s} crictl info >/dev/null 2>&1 || exit 1; \
            cluster_ok=0; \
            if {k3s} kubectl --kubeconfig /var/lib/rancher/k3s/agent/kubelet.kubeconfig --request-timeout=10s get --raw=/readyz >/dev/null 2>&1; then \
                lease_renew_time=$({k3s} kubectl --kubeconfig /var/lib/rancher/k3s/agent/kubelet.kubeconfig --request-timeout=10s -n kube-node-lease get lease {} -o jsonpath='{{.spec.renewTime}}' 2>/dev/null); \
                if [ -n \"$lease_renew_time\" ]; then \
                    lease_epoch=$({busybox} date -u -D '%Y-%m-%dT%H:%M:%S' -d \"$lease_renew_time\" +%s 2>/dev/null); \
                    if [ -n \"$lease_epoch\" ]; then \
                        if test $((now_epoch - lease_epoch)) -le {HOSTED_K3S_AGENT_LEASE_MAX_AGE_SECONDS}; then \
                            cluster_ok=1; \
                        fi; \
                    fi; \
                fi; \
            fi; \
            if [ \"$cluster_ok\" -eq 1 ]; then \
                {busybox} date -u +%s > \"$last_ok_file\"; \
                exit 0; \
            fi; \
            if [ -f \"$last_ok_file\" ]; then \
                last_ok_epoch=$(cat \"$last_ok_file\" 2>/dev/null); \
                if [ -n \"$last_ok_epoch\" ]; then \
                    test $((now_epoch - last_ok_epoch)) -le {HOSTED_K3S_AGENT_TRANSIENT_FAILURE_GRACE_SECONDS}; \
                fi; \
            fi; \
            if [ ! -f \"$last_ok_file\" ] && [ -n \"$bootstrap_epoch\" ]; then \
                test $((now_epoch - bootstrap_epoch)) -le {HOSTED_K3S_AGENT_BOOTSTRAP_GRACE_SECONDS} && exit 0; \
            fi; \
            exit 1",
            shell_single_quote(machine_name)
        ),
        _ => String::from("/usr/bin/k3s crictl info >/dev/null 2>&1"),
    };
    vec![String::from("/bin/sh"), String::from("-lc"), shell]
}

fn hosted_k3s_service_policy(role: &str, machine_name: &str) -> ServicePolicy {
    ServicePolicy {
        restart: ServiceRestartPolicy::Always,
        healthcheck: ServiceHealthcheck {
            policy: ServiceHealthPolicy::Command,
            command: hosted_k3s_service_healthcheck_command(role, machine_name),
            restart_on_unhealthy: true,
        },
    }
}

#[cfg(test)]
fn k3s_bootstrap_command(
    role: &str,
    args: &[String],
    bootstrap_flag: Option<&str>,
    server_url: Option<&str>,
    join_token: Option<&str>,
) -> Vec<String> {
    hosted_k3s_service_command(role, args, bootstrap_flag, server_url, join_token)
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn firecracker_local_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    validate_machine_runtime_launch_config(config)?;

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
                &host.connection,
                hosted_identity.as_ref(),
            )
        );
    }
    let control = MachineControlContract::local_runtime_root();

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
        .chain(attached_volume_preflight_checks(
            request.machine_name,
            &machine.volumes,
            &control,
        ))
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

    let runtime_guest_storage = materialize_runtime_guest_storage_with_overlay(
        &paths,
        &guest_variant.path,
        machine.rootfs_read_only,
        machine.rootfs_overlay.as_ref(),
    )?;

    let effective_network = machine.network.clone().unwrap_or_default();
    if effective_network.enabled {
        setup_host_networking(request.machine_name, &effective_network)
            .context("failed to set up host-side networking for guest VM")?;
    }

    let config_payload = build_firecracker_config(
        kernel_variant.path.clone(),
        runtime_guest_storage.rootfs_path,
        runtime_guest_storage.rootfs_overlay_path,
        &machine.volumes,
        machine.vcpu_count,
        machine.memory_mib,
        machine.kernel_args.clone(),
        machine.rootfs_read_only,
        machine.guest.control_port,
        machine.guest.vsock_cid,
        paths.vsock_path.clone(),
        request.machine_name,
        Some(&effective_network),
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
    File::create(&paths.firecracker_log)
        .with_context(|| format!("failed to create '{}'", paths.firecracker_log.display()))?;

    let mut command = Command::new(&firecracker_binary);
    command
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
        .stderr(Stdio::from(stderr));
    configure_detached_session(&mut command);
    let mut child = command
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
        runtime_class: machine.runtime_class.clone(),
        attached_volumes: machine.volumes.clone(),
    };

    let manifest = serde_json::to_string_pretty(&metadata).context("failed to encode manifest")?;
    fs::write(&paths.manifest_path, format!("{manifest}\n")).with_context(|| {
        format!(
            "failed to write manifest '{}'",
            paths.manifest_path.display()
        )
    })?;

    if effective_network.enabled {
        let state_path = network_state_path(&paths);
        let state_json = serde_json::to_string_pretty(&effective_network)
            .context("failed to encode network state JSON")?;
        fs::write(&state_path, format!("{state_json}\n"))
            .with_context(|| format!("failed to write network state '{}'", state_path.display()))?;
    }

    Ok(metadata)
}

fn avf_launcher_from_env() -> Option<PathBuf> {
    env::var_os("PORT_AVF_LAUNCHER").map(PathBuf::from)
}

fn cloud_hypervisor_local_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    validate_machine_runtime_launch_config(config)?;

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
            "Cloud Hypervisor local launch requires a Linux host; machine '{}' targets host '{}' with platform {:?}",
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
                &host.connection,
                hosted_identity.as_ref(),
            )
        );
    }

    if machine.protection_mode != ProtectionMode::Standard {
        bail!(
            "Cloud Hypervisor local launch only supports protection_mode = standard; machine '{}' requested {:?}",
            request.machine_name,
            machine.protection_mode
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
        bail!("cloud-hypervisor host preflight failed: {details}");
    }

    let cloud_hypervisor_binary = find_binary("cloud-hypervisor").ok_or_else(|| {
        anyhow!(
            "Cloud Hypervisor local launch requires 'cloud-hypervisor' on PATH for machine '{}'",
            request.machine_name
        )
    })?;

    let paths = RuntimePaths::for_machine(request.runtime_root, request.machine_name);
    fs::create_dir_all(&paths.runtime_dir).with_context(|| {
        format!(
            "failed to create runtime directory '{}'",
            paths.runtime_dir.display()
        )
    })?;
    prepare_cloud_hypervisor_runtime_state(&paths, request.machine_name)?;

    let config_path = cloud_hypervisor_config_path(&paths);
    let log_path = cloud_hypervisor_log_path(&paths);
    let api_socket_path = cloud_hypervisor_api_socket_path(&paths);
    let boot_args = format!(
        "{} init=/init port.guest_control_port={}",
        machine.kernel_args, machine.guest.control_port
    );
    let vsock_arg = format!(
        "cid={},socket={}",
        machine.guest.vsock_cid,
        paths.vsock_path.display()
    );
    let runtime_guest_storage =
        materialize_runtime_guest_storage(&paths, &guest_variant.path, machine.rootfs_read_only)?;

    let config_payload = CloudHypervisorLaunchConfig {
        machine_name: request.machine_name.to_string(),
        runtime_dir: paths.runtime_dir.clone(),
        kernel_path: kernel_variant.path.clone(),
        guest_image_path: runtime_guest_storage.rootfs_path.clone(),
        vcpu_count: machine.vcpu_count,
        memory_mib: machine.memory_mib,
        kernel_args: boot_args.clone(),
        rootfs_read_only: machine.rootfs_read_only,
        guest_vsock_cid: machine.guest.vsock_cid,
        guest_control_port: machine.guest.control_port,
        vsock_path: paths.vsock_path.clone(),
        api_socket_path: api_socket_path.clone(),
        console_log: log_path.clone(),
    };
    write_json_file(&config_path, &config_payload)?;
    File::create(&log_path)
        .with_context(|| format!("failed to create '{}'", log_path.display()))?;

    let stdout = File::create(&paths.stdout_log)
        .with_context(|| format!("failed to create '{}'", paths.stdout_log.display()))?;
    let stderr = File::create(&paths.stderr_log)
        .with_context(|| format!("failed to create '{}'", paths.stderr_log.display()))?;

    let disk_arg = format!(
        "path={},readonly={}",
        runtime_guest_storage.rootfs_path.display(),
        if machine.rootfs_read_only {
            "on"
        } else {
            "off"
        }
    );
    let serial_arg = format!("file={}", log_path.display());
    let api_socket_arg = format!("path={}", api_socket_path.display());
    let cpu_arg = format!("boot={}", machine.vcpu_count);
    let memory_arg = format!("size={}M", machine.memory_mib);

    let mut command = Command::new(&cloud_hypervisor_binary);
    command
        .arg("--kernel")
        .arg(&kernel_variant.path)
        .arg("--disk")
        .arg(disk_arg)
        .arg("--cmdline")
        .arg(&boot_args)
        .arg("--cpus")
        .arg(cpu_arg)
        .arg("--memory")
        .arg(memory_arg)
        .arg("--vsock")
        .arg(vsock_arg)
        .arg("--console")
        .arg("off")
        .arg("--serial")
        .arg(serial_arg)
        .arg("--api-socket")
        .arg(api_socket_arg)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_detached_session(&mut command);
    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to start Cloud Hypervisor '{}'",
            cloud_hypervisor_binary.display()
        )
    })?;

    if let Some(status) = wait_for_boot(&mut child, request.boot_wait)? {
        bail!(
            "cloud-hypervisor exited before boot wait elapsed with status {status}; inspect '{}' and '{}'",
            paths.stdout_log.display(),
            paths.stderr_log.display()
        );
    }

    let launched_at_unix_s = unix_timestamp_now()?;
    fs::write(&paths.pid_path, format!("{}\n", child.id()))
        .with_context(|| format!("failed to write pid file '{}'", paths.pid_path.display()))?;

    let metadata = LaunchMetadata {
        machine_name: request.machine_name.to_string(),
        pid: child.id(),
        launched_at_unix_s,
        runtime_dir: paths.runtime_dir.clone(),
        firecracker_binary: cloud_hypervisor_binary.clone(),
        config_path: config_path.clone(),
        log_path: log_path.clone(),
        stdout_path: paths.stdout_log.clone(),
        stderr_path: paths.stderr_log.clone(),
        manifest_path: paths.manifest_path.clone(),
        runtime_class: machine.runtime_class.clone(),
        attached_volumes: Vec::new(),
    };
    write_json_file(&paths.manifest_path, &metadata)?;

    let runtime_metadata = CloudHypervisorRuntimeMetadata {
        machine_name: request.machine_name.to_string(),
        pid: child.id(),
        binary: cloud_hypervisor_binary,
        config_path,
        metadata_path: cloud_hypervisor_runtime_metadata_path(&paths),
        vsock_path: paths.vsock_path.clone(),
        guest_vsock_cid: machine.guest.vsock_cid,
        guest_control_port: machine.guest.control_port,
        api_socket_path,
        console_log: log_path,
        launched_at_unix_s,
    };
    write_json_file(
        &cloud_hypervisor_runtime_metadata_path(&paths),
        &runtime_metadata,
    )?;

    Ok(metadata)
}

fn avf_local_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    avf_local_launch_machine_with_host_os(config, request, env::consts::OS, avf_launcher_from_env())
}

fn avf_local_launch_machine_with_host_os(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
    host_os: &str,
    launcher_override: Option<PathBuf>,
) -> Result<LaunchMetadata> {
    validate_machine_runtime_launch_config(config)?;

    let machine = config
        .machines
        .get(request.machine_name)
        .with_context(|| format!("unknown machine '{}'", request.machine_name))?;
    let host = config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
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
    if host_os != "macos" {
        bail!(
            "AVF local launch requires running Port on macOS; detected host OS '{}'",
            host_os
        );
    }

    let launcher = launcher_override.ok_or_else(|| {
        anyhow!(
            "AVF local driver is configured, but no AVF launcher helper is set. Set PORT_AVF_LAUNCHER to a macOS AVF launcher binary; Port does not ship a bundled macOS-only helper in this slice."
        )
    })?;
    if !launcher.exists() {
        bail!(
            "AVF launcher helper '{}' does not exist",
            launcher.display()
        );
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
        bail!("avf host preflight failed: {details}");
    }

    let paths = RuntimePaths::for_machine(request.runtime_root, request.machine_name);
    fs::create_dir_all(&paths.runtime_dir).with_context(|| {
        format!(
            "failed to create runtime directory '{}'",
            paths.runtime_dir.display()
        )
    })?;
    prepare_avf_runtime_state(&paths, request.machine_name)?;

    let runtime_guest_storage =
        materialize_runtime_guest_storage(&paths, &guest_variant.path, machine.rootfs_read_only)?;

    let contract = AvfExecutionContract::linux_guest();
    let config_payload = AvfLaunchConfig {
        machine_name: request.machine_name.to_string(),
        runtime_dir: paths.runtime_dir.clone(),
        kernel_path: kernel_variant.path.clone(),
        guest_image_path: runtime_guest_storage.rootfs_path,
        vcpu_count: machine.vcpu_count,
        memory_mib: machine.memory_mib,
        kernel_args: machine.kernel_args.clone(),
        rootfs_read_only: machine.rootfs_read_only,
        guest_vsock_cid: machine.guest.vsock_cid,
        guest_control_port: machine.guest.control_port,
        guest_agent_socket: paths.guest_agent_socket.clone(),
        guest_transport: contract.guest_transport,
        console_transport: contract.console_transport,
        console_log: paths.firecracker_log.clone(),
    };
    write_json_file(&paths.config_path, &config_payload)?;
    File::create(&paths.firecracker_log)
        .with_context(|| format!("failed to create '{}'", paths.firecracker_log.display()))?;

    let stdout = File::create(&paths.stdout_log)
        .with_context(|| format!("failed to create '{}'", paths.stdout_log.display()))?;
    let stderr = File::create(&paths.stderr_log)
        .with_context(|| format!("failed to create '{}'", paths.stderr_log.display()))?;

    let mut command = Command::new(&launcher);
    command
        .arg("--machine")
        .arg(request.machine_name)
        .arg("--config")
        .arg(&paths.config_path)
        .arg("--runtime-dir")
        .arg(&paths.runtime_dir)
        .arg("--guest-agent-socket")
        .arg(&paths.guest_agent_socket)
        .arg("--console-log")
        .arg(&paths.firecracker_log)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_detached_session(&mut command);
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to start AVF launcher '{}'", launcher.display()))?;

    if let Some(status) = wait_for_boot(&mut child, request.boot_wait)? {
        bail!(
            "AVF launcher exited before boot wait elapsed with status {status}; inspect '{}' and '{}'",
            paths.stdout_log.display(),
            paths.stderr_log.display()
        );
    }

    let launched_at_unix_s = unix_timestamp_now()?;
    fs::write(&paths.pid_path, format!("{}\n", child.id()))
        .with_context(|| format!("failed to write pid file '{}'", paths.pid_path.display()))?;

    let metadata = LaunchMetadata {
        machine_name: request.machine_name.to_string(),
        pid: child.id(),
        launched_at_unix_s,
        runtime_dir: paths.runtime_dir.clone(),
        firecracker_binary: launcher.clone(),
        config_path: paths.config_path.clone(),
        log_path: paths.firecracker_log.clone(),
        stdout_path: paths.stdout_log.clone(),
        stderr_path: paths.stderr_log.clone(),
        manifest_path: paths.manifest_path.clone(),
        runtime_class: machine.runtime_class.clone(),
        attached_volumes: Vec::new(),
    };
    write_json_file(&paths.manifest_path, &metadata)?;

    let avf_metadata = AvfRuntimeMetadata {
        machine_name: request.machine_name.to_string(),
        pid: child.id(),
        launcher,
        config_path: paths.config_path.clone(),
        metadata_path: avf_runtime_metadata_path(&paths),
        guest_agent_socket: paths.guest_agent_socket.clone(),
        console_log: paths.firecracker_log.clone(),
        guest_transport: contract.guest_transport,
        console_transport: contract.console_transport,
        launched_at_unix_s,
    };
    write_json_file(&avf_runtime_metadata_path(&paths), &avf_metadata)?;

    Ok(metadata)
}

pub fn list_machines(config: &PortConfig, runtime_root: &Path) -> Result<Vec<MachineStatus>> {
    let mut machines = BTreeMap::new();
    for machine in local_runtime_driver().list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }
    for machine in CloudHypervisorLocalDriver.list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }
    for machine in AvfLocalDriver.list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }
    for machine in hosted_control_plane_driver().list_machines(config, runtime_root)? {
        machines.insert(machine.machine_name.clone(), machine);
    }

    Ok(machines.into_values().collect())
}

fn is_missing_hosted_auth_token(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .to_string()
            .contains("hosted auth token is missing from environment variable")
    })
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

fn resolve_live_pid_by_existence(
    pid_from_file: Option<u32>,
    manifest_pid: Option<u32>,
) -> Result<Option<u32>> {
    if let Some(pid) = pid_from_file {
        if process_is_live(pid)? {
            return Ok(Some(pid));
        }
    }
    if let Some(pid) = manifest_pid {
        if Some(pid) != pid_from_file && process_is_live(pid)? {
            return Ok(Some(pid));
        }
    }
    Ok(None)
}

fn avf_local_machine_status(runtime_root: &Path, machine_name: &str) -> Result<MachineStatus> {
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);
    if !paths.runtime_dir.exists() {
        bail!(
            "runtime state for machine '{}' does not exist under '{}'",
            machine_name,
            runtime_root.display()
        );
    }

    let pid_from_file = match read_pid_file(&paths.pid_path) {
        Ok(pid) => pid,
        Err(error) => {
            return Ok(malformed_machine_status(
                machine_name,
                &paths,
                MachineControlContract::local_runtime_root(),
                error.to_string(),
            ));
        }
    };
    if !paths.manifest_path.exists() {
        return Ok(malformed_machine_status(
            machine_name,
            &paths,
            MachineControlContract::local_runtime_root(),
            format!(
                "runtime manifest '{}' is missing",
                paths.manifest_path.display()
            ),
        ));
    }

    let metadata_path = avf_runtime_metadata_path(&paths);
    if !metadata_path.exists() {
        return Ok(malformed_machine_status(
            machine_name,
            &paths,
            MachineControlContract::local_runtime_root(),
            format!(
                "AVF runtime metadata '{}' is missing",
                metadata_path.display()
            ),
        ));
    }

    let manifest = match read_launch_metadata(&paths.manifest_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(malformed_machine_status(
                machine_name,
                &paths,
                MachineControlContract::local_runtime_root(),
                format!(
                    "failed to parse manifest '{}': {error}",
                    paths.manifest_path.display()
                ),
            ));
        }
    };
    let avf_metadata: AvfRuntimeMetadata = match read_json_file(&metadata_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(malformed_machine_status(
                machine_name,
                &paths,
                MachineControlContract::local_runtime_root(),
                format!(
                    "failed to parse AVF runtime metadata '{}': {error}",
                    metadata_path.display()
                ),
            ));
        }
    };

    let live_pid = resolve_live_pid_by_existence(pid_from_file, Some(manifest.pid))?;
    let pid = live_pid.or(pid_from_file).or(Some(manifest.pid));
    let (state, detail) = match live_pid {
        Some(_) => (
            MachineRuntimeState::Running,
            format!(
                "live AVF launcher '{}' matches runtime manifest; metadata '{}'",
                avf_metadata.launcher.display(),
                metadata_path.display()
            ),
        ),
        None if pid_from_file.is_some() => (
            MachineRuntimeState::Stale,
            format!(
                "recorded AVF launcher pid is no longer live; metadata '{}'",
                metadata_path.display()
            ),
        ),
        None => (
            MachineRuntimeState::Stopped,
            format!(
                "launch manifest exists but no live AVF launcher process is recorded; metadata '{}'",
                metadata_path.display()
            ),
        ),
    };

    Ok(MachineStatus {
        machine_name: machine_name.to_string(),
        state,
        pid,
        control: MachineControlContract::local_runtime_root(),
        runtime_dir: paths.runtime_dir,
        config_path: paths.config_path,
        manifest_path: paths.manifest_path,
        pid_path: paths.pid_path,
        firecracker_log: paths.firecracker_log,
        stdout_log: paths.stdout_log,
        stderr_log: paths.stderr_log,
        runtime_class: manifest.runtime_class,
        attached_volumes: Vec::new(),
        hosted_fleet_nodes: Vec::new(),
        guest_refresh_age_seconds: None,
        wedged_since_unix_s: None,
        wedge_class: None,
        recovery_attempts: RecoveryAttemptCounters::default(),
        last_recovery_action: None,
        recovery_state: RecoveryState::default(),
        detail,
    })
}

fn cloud_hypervisor_local_machine_status(
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

    let pid_from_file = match read_pid_file(&paths.pid_path) {
        Ok(pid) => pid,
        Err(error) => {
            return Ok(cloud_hypervisor_malformed_machine_status(
                machine_name,
                &paths,
                error.to_string(),
            ));
        }
    };
    if !paths.manifest_path.exists() {
        return Ok(cloud_hypervisor_malformed_machine_status(
            machine_name,
            &paths,
            format!(
                "runtime manifest '{}' is missing",
                paths.manifest_path.display()
            ),
        ));
    }

    let metadata_path = cloud_hypervisor_runtime_metadata_path(&paths);
    if !metadata_path.exists() {
        return Ok(cloud_hypervisor_malformed_machine_status(
            machine_name,
            &paths,
            format!(
                "Cloud Hypervisor runtime metadata '{}' is missing",
                metadata_path.display()
            ),
        ));
    }

    let manifest = match read_launch_metadata(&paths.manifest_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(cloud_hypervisor_malformed_machine_status(
                machine_name,
                &paths,
                format!(
                    "failed to parse manifest '{}': {error}",
                    paths.manifest_path.display()
                ),
            ));
        }
    };
    let metadata: CloudHypervisorRuntimeMetadata = match read_json_file(&metadata_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(cloud_hypervisor_malformed_machine_status(
                machine_name,
                &paths,
                format!(
                    "failed to parse Cloud Hypervisor runtime metadata '{}': {error}",
                    metadata_path.display()
                ),
            ));
        }
    };

    let live_pid = resolve_live_pid_by_existence(pid_from_file, Some(manifest.pid))?;
    let pid = live_pid.or(pid_from_file).or(Some(manifest.pid));
    let (state, detail) = match live_pid {
        Some(_) => (
            MachineRuntimeState::Running,
            format!(
                "live Cloud Hypervisor process matches runtime manifest; metadata '{}'",
                metadata_path.display()
            ),
        ),
        None if pid_from_file.is_some() => (
            MachineRuntimeState::Stale,
            format!(
                "recorded Cloud Hypervisor pid is no longer live; metadata '{}'",
                metadata_path.display()
            ),
        ),
        None => (
            MachineRuntimeState::Stopped,
            format!(
                "launch manifest exists but no live Cloud Hypervisor process is recorded; metadata '{}'",
                metadata_path.display()
            ),
        ),
    };

    Ok(MachineStatus {
        machine_name: machine_name.to_string(),
        state,
        pid,
        control: MachineControlContract::local_runtime_root(),
        runtime_dir: paths.runtime_dir,
        config_path: metadata.config_path,
        manifest_path: paths.manifest_path,
        pid_path: paths.pid_path,
        firecracker_log: metadata.console_log,
        stdout_log: paths.stdout_log,
        stderr_log: paths.stderr_log,
        runtime_class: manifest.runtime_class,
        attached_volumes: Vec::new(),
        hosted_fleet_nodes: Vec::new(),
        guest_refresh_age_seconds: None,
        wedged_since_unix_s: None,
        wedge_class: None,
        recovery_attempts: RecoveryAttemptCounters::default(),
        last_recovery_action: None,
        recovery_state: RecoveryState::default(),
        detail,
    })
}

fn firecracker_local_machine_monitor(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    let status = firecracker_local_machine_status(runtime_root, machine_name)?;
    machine_monitor_report(status, None, None, Vec::new())
}

fn avf_local_machine_monitor(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    let status = avf_local_machine_status(runtime_root, machine_name)?;
    machine_monitor_report(status, None, None, Vec::new())
}

fn cloud_hypervisor_local_machine_monitor(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineMonitorReport> {
    let status = cloud_hypervisor_local_machine_status(runtime_root, machine_name)?;
    machine_monitor_report(status, None, None, Vec::new())
}

fn firecracker_local_machine_top(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineTopReport> {
    let status = firecracker_local_machine_status(runtime_root, machine_name)?;
    machine_top_report(status, None, None, Vec::new())
}

fn avf_local_machine_top(runtime_root: &Path, machine_name: &str) -> Result<MachineTopReport> {
    let status = avf_local_machine_status(runtime_root, machine_name)?;
    let avf_command = match status.pid {
        Some(pid) => process_cmdline(pid)?,
        None => None,
    };
    let metadata_path =
        avf_runtime_metadata_path(&RuntimePaths::for_machine(runtime_root, machine_name));
    let mut entries = Vec::new();
    if status.pid.is_some() || status.manifest_path.exists() {
        entries.push(MachineTopEntry {
            kind: MachineTopEntryKind::Hypervisor,
            name: String::from("avf"),
            state: status.state,
            pid: status.pid,
            command: avf_command,
            source: metadata_path,
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
        control_plane: None,
        node_name: None,
        host_groups: Vec::new(),
        runtime_dir: status.runtime_dir,
        detail: status.detail,
        entries,
    })
}

fn cloud_hypervisor_local_machine_top(
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineTopReport> {
    let status = cloud_hypervisor_local_machine_status(runtime_root, machine_name)?;
    let command = match status.pid {
        Some(pid) => process_cmdline(pid)?,
        None => None,
    };
    let metadata_path = cloud_hypervisor_runtime_metadata_path(&RuntimePaths::for_machine(
        runtime_root,
        machine_name,
    ));
    let mut entries = Vec::new();
    if status.pid.is_some() || status.manifest_path.exists() {
        entries.push(MachineTopEntry {
            kind: MachineTopEntryKind::Hypervisor,
            name: String::from("cloud-hypervisor"),
            state: status.state,
            pid: status.pid,
            command,
            source: metadata_path,
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
        control_plane: None,
        node_name: None,
        host_groups: Vec::new(),
        runtime_dir: status.runtime_dir,
        detail: status.detail,
        entries,
    })
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
    if machine_is_hosted(config, request.machine_name)? {
        return hosted_control_plane_put_machine_secret(config, request);
    }
    put_machine_secret_local(config, request)
}

pub(crate) fn put_machine_secret_local(
    config: &PortConfig,
    request: SecretPutRequest<'_>,
) -> Result<MachineSecretSummary> {
    let context =
        resolve_service_runtime_context(config, request.runtime_root, request.machine_name, None)?;
    let secret = store_machine_secret(&context.status.runtime_dir, request.name, request.value)?;
    let detail = String::from(
        "stored secret metadata plus runtime-file backend under the resolved machine runtime; service execution now materializes the value through the same runtime owner",
    );
    Ok(machine_secret_summary_from_storage(
        request.machine_name,
        context,
        &secret,
        detail,
    ))
}

fn store_machine_secret(
    runtime_dir: &Path,
    secret_name: &str,
    value: &str,
) -> Result<StoredMachineSecret> {
    validate_identifier(secret_name, "secret name")?;
    let dir = service_secret_dir(runtime_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create secret directory '{}'", dir.display()))?;
    let backend_dir = service_secret_backend_dir(runtime_dir, ServiceSecretBackend::RuntimeFile);
    fs::create_dir_all(&backend_dir).with_context(|| {
        format!(
            "failed to create secret backend directory '{}'",
            backend_dir.display()
        )
    })?;
    let record = MachineSecretRecord {
        name: secret_name.to_string(),
        backend: ServiceSecretBackend::RuntimeFile,
        materialization: ServiceSecretMaterialization::Env,
    };
    let path = service_secret_metadata_path(runtime_dir, secret_name);
    let backend_path = service_secret_value_path(runtime_dir, secret_name, record.backend);
    write_secret_value_file(&backend_path, value)?;
    write_json_file(&path, &record)?;
    Ok(StoredMachineSecret {
        record,
        path,
        backend_path,
    })
}

pub fn list_machine_secrets(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<MachineSecretSummary>> {
    if machine_is_hosted(config, machine_name)? {
        return hosted_control_plane_list_machine_secrets(config, machine_name);
    }
    list_machine_secrets_local(config, runtime_root, machine_name)
}

pub(crate) fn list_machine_secrets_local(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<MachineSecretSummary>> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name, None)?;
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
        if entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("json")
        {
            continue;
        }
        let record: MachineSecretRecord = read_json_file(&entry.path())?;
        let backend_path =
            service_secret_value_path(&context.status.runtime_dir, &record.name, record.backend);
        secrets.push(machine_secret_summary_from_storage(
            machine_name,
            context.clone(),
            &StoredMachineSecret {
                record,
                path: entry.path(),
                backend_path,
            },
            String::from(
                "secret metadata is available to service and sandbox bindings through the runtime-file backend",
            ),
        ));
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
    if machine_is_hosted(config, machine_name)? {
        return hosted_control_plane_delete_machine_secret(config, machine_name, secret_name);
    }
    delete_machine_secret_local(config, runtime_root, machine_name, secret_name)
}

pub(crate) fn delete_machine_secret_local(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    secret_name: &str,
) -> Result<MachineSecretSummary> {
    let context = resolve_service_runtime_context(config, runtime_root, machine_name, None)?;
    validate_identifier(secret_name, "secret name")?;
    let references = service_references_secret(&context.status.runtime_dir, secret_name)?;
    if !references.is_empty() {
        bail!(
            "cannot remove secret '{}' because it is referenced by service definitions: {}",
            secret_name,
            references.join(", ")
        );
    }
    let secret = load_machine_secret(&context.status.runtime_dir, secret_name)?;
    fs::remove_file(&secret.backend_path).with_context(|| {
        format!(
            "failed to remove secret backend value '{}'",
            secret.backend_path.display()
        )
    })?;
    fs::remove_file(&secret.path)
        .with_context(|| format!("failed to remove secret '{}'", secret.path.display()))?;
    Ok(machine_secret_summary_from_storage(
        machine_name,
        context,
        &secret,
        String::from(
            "removed secret metadata and runtime-file backend value from the resolved machine runtime",
        ),
    ))
}

pub fn apply_machine_service(
    config: &PortConfig,
    request: ServiceApplyRequest<'_>,
) -> Result<ServiceDefinitionStatus> {
    if machine_is_hosted(config, request.machine_name)? {
        return hosted_control_plane_apply_machine_service(config, request);
    }
    apply_machine_service_live(config, config, request)
}

pub(crate) fn apply_machine_service_local(
    config: &PortConfig,
    request: ServiceApplyRequest<'_>,
) -> Result<ServiceDefinitionStatus> {
    validate_identifier(request.name, "service name")?;
    if request.command.is_empty() {
        bail!("service apply requires a command");
    }
    request
        .policy
        .validate_for_kind(request.kind)
        .map_err(anyhow::Error::msg)?;
    let context = resolve_service_runtime_context(
        config,
        request.runtime_root,
        request.machine_name,
        request.host_group,
    )?;
    validate_secret_bindings(&request.secret_bindings)?;
    for binding in &request.secret_bindings {
        let secret = load_machine_secret(&context.status.runtime_dir, &binding.secret)
            .with_context(|| {
                format!(
                    "secret '{}' referenced by '{}' is unavailable for machine '{}'",
                    binding.secret, binding.env, request.machine_name
                )
            })?;
        if !matches!(
            secret.record.materialization,
            ServiceSecretMaterialization::Env
        ) {
            bail!(
                "secret '{}' referenced by '{}' does not support env materialization",
                binding.secret,
                binding.env
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
        policy: request.policy,
        control: context.status.control.clone(),
        control_plane: context.control_plane.clone(),
        node_name: context.node_name.clone(),
        host_groups: context.host_groups.clone(),
        host_group_policies: context.host_group_policies.clone(),
        target_host_group: context.target_host_group.clone(),
        scheduler: context.scheduler,
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
    if machine_is_hosted(config, machine_name)? {
        return hosted_control_plane_list_machine_services(config, machine_name);
    }
    refresh_machine_service_list(config, config, runtime_root, machine_name)
}

pub(crate) fn list_machine_services_stored_local(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<ServiceDefinitionStatus>> {
    let _ = config;
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    if !runtime_dir.exists() {
        bail!(
            "service operations require an existing Port runtime for machine '{}': runtime state does not exist under '{}'",
            machine_name,
            runtime_root.display()
        );
    }
    let dir = service_definition_dir(&runtime_dir);
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
    if machine_is_hosted(config, machine_name)? {
        return hosted_control_plane_machine_service_status(config, machine_name, service_name);
    }
    refresh_machine_service_runtime(config, config, runtime_root, machine_name, service_name)
}

pub(crate) fn machine_service_status_stored_local(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let _ = config;
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    if !runtime_dir.exists() {
        bail!(
            "service operations require an existing Port runtime for machine '{}': runtime state does not exist under '{}'",
            machine_name,
            runtime_root.display()
        );
    }
    validate_identifier(service_name, "service name")?;
    let path = service_definition_dir(&runtime_dir).join(format!("{service_name}.json"));
    let record: ServiceDefinitionRecord = read_json_file(&path)?;
    Ok(service_status_from_record(record, path))
}

pub fn stop_machine_service(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    if machine_is_hosted(config, machine_name)? {
        return hosted_control_plane_stop_machine_service(config, machine_name, service_name);
    }
    stop_machine_service_live(config, config, runtime_root, machine_name, service_name)
}

pub(crate) fn stop_machine_service_stored_local(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let _ = config;
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    if !runtime_dir.exists() {
        bail!(
            "service operations require an existing Port runtime for machine '{}': runtime state does not exist under '{}'",
            machine_name,
            runtime_root.display()
        );
    }
    validate_identifier(service_name, "service name")?;
    let path = service_definition_dir(&runtime_dir).join(format!("{service_name}.json"));
    let mut record: ServiceDefinitionRecord = read_json_file(&path)?;
    record.desired_state = ServiceDesiredState::Stopped;
    record.detail = String::from(
        "service definition is retained with desired state stopped; hosted execution and teardown remain a follow-on slice",
    );
    write_json_file(&path, &record)?;
    Ok(service_status_from_record(record, path))
}

fn load_service_secret_env(
    runtime_dir: &Path,
    bindings: &[ServiceSecretBinding],
) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for binding in bindings {
        let secret = load_machine_secret(runtime_dir, &binding.secret)?;
        if !matches!(
            secret.record.materialization,
            ServiceSecretMaterialization::Env
        ) {
            bail!(
                "secret '{}' cannot be materialized as an environment binding",
                binding.secret
            );
        }
        env.insert(
            binding.env.clone(),
            read_secret_value_file(&secret.backend_path)?,
        );
    }
    Ok(env)
}

fn service_runtime_record_from_managed_status(
    status: &ManagedServiceStatus,
) -> ServiceRuntimeRecord {
    let state = match status.state {
        ManagedServiceRuntimeState::Stored => ServiceRuntimeState::Stored,
        ManagedServiceRuntimeState::Starting => ServiceRuntimeState::Starting,
        ManagedServiceRuntimeState::Running => ServiceRuntimeState::Running,
        ManagedServiceRuntimeState::Exited => ServiceRuntimeState::Exited,
        ManagedServiceRuntimeState::Stopped => ServiceRuntimeState::Stopped,
        ManagedServiceRuntimeState::Failed => ServiceRuntimeState::Failed,
    };
    ServiceRuntimeRecord {
        state,
        restart_count: status.restart_count,
        pid: status.pid,
        exit_code: status.exit_code,
        last_exit_code: status.last_exit_code,
        last_exit_detail: status.last_exit_detail.clone(),
        health_state: status.health_state,
        health_detail: status.health_detail.clone(),
        stdout_path: status.stdout_path.as_ref().map(PathBuf::from),
        stderr_path: status.stderr_path.as_ref().map(PathBuf::from),
        detail: status.detail.clone(),
    }
}

fn managed_service_result_status(result: OperationResult) -> Result<ManagedServiceStatus> {
    let OperationResult::ManagedService(ManagedServiceResult::Status(status)) = result else {
        bail!("guest agent returned an unexpected managed service result");
    };
    Ok(status)
}

fn managed_service_result_list(result: OperationResult) -> Result<Vec<ManagedServiceStatus>> {
    let OperationResult::ManagedService(ManagedServiceResult::List { services }) = result else {
        bail!("guest agent returned an unexpected managed service list result");
    };
    Ok(services)
}

pub(crate) fn apply_machine_service_live(
    metadata_config: &PortConfig,
    guest_config: &PortConfig,
    request: ServiceApplyRequest<'_>,
) -> Result<ServiceDefinitionStatus> {
    let stored = apply_machine_service_local(metadata_config, request.clone())?;
    let runtime_dir =
        RuntimePaths::for_machine(request.runtime_root, request.machine_name).runtime_dir;
    let env = load_service_secret_env(&runtime_dir, &stored.secret_bindings)?;
    let managed = managed_service_result_status(execute_guest_operation(
        guest_config,
        GuestRequest {
            machine_name: request.machine_name,
            runtime_root: request.runtime_root,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: request.name.to_string(),
                    kind: match request.kind {
                        ServiceKind::Service => port_agent_protocol::ManagedServiceKind::Service,
                        ServiceKind::Sandbox => port_agent_protocol::ManagedServiceKind::Sandbox,
                    },
                    command: request.command.clone(),
                    env,
                    cwd: None,
                    policy: request.policy.clone(),
                },
            }),
        },
    )?)?;
    write_service_runtime_record(
        &runtime_dir,
        request.name,
        &service_runtime_record_from_managed_status(&managed),
    )?;
    machine_service_status_stored_local(
        metadata_config,
        request.runtime_root,
        request.machine_name,
        request.name,
    )
}

pub(crate) fn refresh_machine_service_runtime(
    metadata_config: &PortConfig,
    guest_config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    match execute_guest_operation(
        guest_config,
        GuestRequest {
            machine_name,
            runtime_root,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: service_name.to_string(),
                },
            }),
        },
    )
    .and_then(managed_service_result_status)
    {
        Ok(status) => {
            write_service_runtime_record(
                &runtime_dir,
                service_name,
                &service_runtime_record_from_managed_status(&status),
            )?;
        }
        Err(error) if error.to_string().contains("does not exist") => {}
        Err(error) => {
            let mut stored = machine_service_status_stored_local(
                metadata_config,
                runtime_root,
                machine_name,
                service_name,
            )?;
            stored.detail = format!(
                "{} Stored runtime record returned because live refresh failed: {}",
                stored.detail, error
            );
            return Ok(stored);
        }
    }
    machine_service_status_stored_local(metadata_config, runtime_root, machine_name, service_name)
}

pub(crate) fn refresh_machine_service_list(
    metadata_config: &PortConfig,
    guest_config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Vec<ServiceDefinitionStatus>> {
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    match execute_guest_operation(
        guest_config,
        GuestRequest {
            machine_name,
            runtime_root,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::List,
            }),
        },
    )
    .and_then(managed_service_result_list)
    {
        Ok(statuses) => {
            for status in statuses {
                write_service_runtime_record(
                    &runtime_dir,
                    &status.name,
                    &service_runtime_record_from_managed_status(&status),
                )?;
            }
        }
        Err(error) => {
            let mut stored =
                list_machine_services_stored_local(metadata_config, runtime_root, machine_name)?;
            if stored.is_empty() {
                return Err(error);
            }
            for status in &mut stored {
                status.detail = format!(
                    "{} Stored runtime record returned because live refresh failed: {}",
                    status.detail, error
                );
            }
            return Ok(stored);
        }
    }
    list_machine_services_stored_local(metadata_config, runtime_root, machine_name)
}

pub(crate) fn stop_machine_service_live(
    metadata_config: &PortConfig,
    guest_config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let _ = stop_machine_service_stored_local(
        metadata_config,
        runtime_root,
        machine_name,
        service_name,
    )?;
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    match managed_service_result_status(execute_guest_operation(
        guest_config,
        GuestRequest {
            machine_name,
            runtime_root,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Stop {
                    name: service_name.to_string(),
                },
            }),
        },
    )?) {
        Ok(status) => {
            write_service_runtime_record(
                &runtime_dir,
                service_name,
                &service_runtime_record_from_managed_status(&status),
            )?;
        }
        Err(error) if error.to_string().contains("does not exist") => {}
        Err(error) => return Err(error),
    }
    machine_service_status_stored_local(metadata_config, runtime_root, machine_name, service_name)
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

    teardown_host_networking_from_state(&paths, machine_name);

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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: status.attached_volumes.clone(),
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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: status.attached_volumes.clone(),
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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: status.attached_volumes.clone(),
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

fn wait_for_pid_exit(pid: u32, timeout: Duration) -> Result<bool> {
    let step = Duration::from_millis(100);
    let mut waited = Duration::ZERO;

    while waited < timeout {
        if process_cmdline(pid)?.is_none() {
            return Ok(true);
        }
        thread::sleep(step);
        waited += step;
    }

    Ok(process_cmdline(pid)?.is_none())
}

fn avf_local_stop_machine(
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let status = avf_local_machine_status(runtime_root, machine_name)?;
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);

    match status.state {
        MachineRuntimeState::Running => {
            let pid = status
                .pid
                .context("running machine status did not include a pid")?;
            signal_process(pid, libc::SIGTERM).with_context(|| {
                format!("failed to stop AVF machine '{}' with SIGTERM", machine_name)
            })?;
            if !wait_for_pid_exit(pid, timeout)? {
                signal_process(pid, libc::SIGKILL).with_context(|| {
                    format!(
                        "failed to force-stop AVF machine '{}' with SIGKILL",
                        machine_name
                    )
                })?;
                if !wait_for_pid_exit(pid, Duration::from_secs(1))? {
                    bail!(
                        "AVF machine '{}' did not stop after SIGTERM/SIGKILL for pid {}",
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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from(
                    "sent SIGTERM to AVF launcher pid and cleaned transient runtime paths",
                ),
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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from("AVF machine was already stopped"),
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
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from("removed stale AVF runtime pid and transient paths"),
            })
        }
        MachineRuntimeState::Malformed => bail!("cannot stop malformed AVF runtime state"),
    }
}

fn cloud_hypervisor_local_stop_machine(
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let status = cloud_hypervisor_local_machine_status(runtime_root, machine_name)?;
    let paths = RuntimePaths::for_machine(runtime_root, machine_name);

    match status.state {
        MachineRuntimeState::Running => {
            let pid = status
                .pid
                .context("running machine status did not include a pid")?;
            signal_process(pid, libc::SIGTERM).with_context(|| {
                format!(
                    "failed to stop Cloud Hypervisor machine '{}' with SIGTERM",
                    machine_name
                )
            })?;
            if !wait_for_pid_exit(pid, timeout)? {
                signal_process(pid, libc::SIGKILL).with_context(|| {
                    format!(
                        "failed to force-stop Cloud Hypervisor machine '{}' with SIGKILL",
                        machine_name
                    )
                })?;
                if !wait_for_pid_exit(pid, Duration::from_secs(1))? {
                    bail!(
                        "Cloud Hypervisor machine '{}' did not stop after SIGTERM/SIGKILL for pid {}",
                        machine_name,
                        pid
                    );
                }
            }
            cleanup_cloud_hypervisor_runtime_transient_paths(&paths)?;

            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Running,
                current_state: MachineRuntimeState::Stopped,
                pid: Some(pid),
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from(
                    "sent SIGTERM to Cloud Hypervisor pid and cleaned transient runtime paths",
                ),
            })
        }
        MachineRuntimeState::Stopped => {
            cleanup_cloud_hypervisor_runtime_transient_paths(&paths)?;

            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Stopped,
                current_state: MachineRuntimeState::Stopped,
                pid: status.pid,
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from("Cloud Hypervisor machine was already stopped"),
            })
        }
        MachineRuntimeState::Stale => {
            cleanup_cloud_hypervisor_runtime_transient_paths(&paths)?;

            Ok(StopResult {
                machine_name: machine_name.to_string(),
                previous_state: MachineRuntimeState::Stale,
                current_state: MachineRuntimeState::Stopped,
                pid: status.pid,
                control: MachineControlContract::local_runtime_root(),
                runtime_dir: paths.runtime_dir,
                runtime_class: status.runtime_class.clone(),
                attached_volumes: Vec::new(),
                detail: String::from(
                    "cleaned stale Cloud Hypervisor runtime state for already-stopped machine",
                ),
            })
        }
        MachineRuntimeState::Malformed => {
            bail!("cannot stop malformed Cloud Hypervisor runtime state")
        }
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

fn avf_runtime_metadata_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("avf-runtime.json")
}

fn cloud_hypervisor_config_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("cloud-hypervisor-config.json")
}

fn cloud_hypervisor_log_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("cloud-hypervisor.log")
}

fn cloud_hypervisor_api_socket_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("cloud-hypervisor.api.sock")
}

fn cloud_hypervisor_runtime_metadata_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("cloud-hypervisor-runtime.json")
}

fn prepare_avf_runtime_state(paths: &RuntimePaths, machine_name: &str) -> Result<()> {
    if let Some(pid) = read_pid_file(&paths.pid_path)? {
        if process_is_live(pid)? {
            bail!(
                "machine '{}' already appears to be running with pid {} in '{}'; stop it first or choose a different --runtime-root",
                machine_name,
                pid,
                paths.runtime_dir.display()
            );
        }
    }

    remove_stale_runtime_path(&paths.pid_path, "pid file")?;
    remove_stale_runtime_path(&paths.vsock_path, "vsock socket")?;
    remove_stale_runtime_path(&paths.guest_agent_socket, "guest-agent socket")?;

    Ok(())
}

fn prepare_cloud_hypervisor_runtime_state(paths: &RuntimePaths, machine_name: &str) -> Result<()> {
    if let Some(pid) = read_pid_file(&paths.pid_path)? {
        if process_is_live(pid)? {
            bail!(
                "machine '{}' already appears to be running with pid {} in '{}'; stop it first or choose a different --runtime-root",
                machine_name,
                pid,
                paths.runtime_dir.display()
            );
        }
    }

    cleanup_cloud_hypervisor_runtime_transient_paths(paths)
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
        runtime_class: manifest.runtime_class,
        attached_volumes: manifest.attached_volumes,
        hosted_fleet_nodes: Vec::new(),
        guest_refresh_age_seconds: None,
        wedged_since_unix_s: None,
        wedge_class: None,
        recovery_attempts: RecoveryAttemptCounters::default(),
        last_recovery_action: None,
        recovery_state: RecoveryState::default(),
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
    if !process_is_live(pid)? {
        return Ok(false);
    }

    let Some(cmdline) = process_cmdline(pid)? else {
        return Ok(false);
    };

    Ok(matches_firecracker_process(&cmdline, machine_name))
}

#[cfg(target_os = "linux")]
fn process_state_code(pid: u32) -> Result<Option<char>> {
    let status_path = PathBuf::from("/proc").join(pid.to_string()).join("status");
    if !status_path.exists() {
        return Ok(None);
    }

    let status = fs::read_to_string(&status_path)
        .with_context(|| format!("failed to read process status '{}'", status_path.display()))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("State:"))
        .and_then(|state| state.trim().chars().next()))
}

#[cfg(not(target_os = "linux"))]
fn process_state_code(_pid: u32) -> Result<Option<char>> {
    Ok(None)
}

fn process_is_live(pid: u32) -> Result<bool> {
    if !process_exists(pid)? {
        return Ok(false);
    }

    Ok(!matches!(process_state_code(pid)?, Some('Z')))
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

fn cleanup_cloud_hypervisor_runtime_transient_paths(paths: &RuntimePaths) -> Result<()> {
    cleanup_runtime_transient_paths(paths)?;
    remove_stale_runtime_path(
        &cloud_hypervisor_api_socket_path(paths),
        "Cloud Hypervisor API socket",
    )?;
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

fn cloud_hypervisor_malformed_machine_status(
    machine_name: &str,
    paths: &RuntimePaths,
    detail: String,
) -> MachineStatus {
    MachineStatus {
        machine_name: machine_name.to_string(),
        state: MachineRuntimeState::Malformed,
        pid: None,
        control: MachineControlContract::local_runtime_root(),
        runtime_dir: paths.runtime_dir.clone(),
        config_path: cloud_hypervisor_config_path(paths),
        manifest_path: paths.manifest_path.clone(),
        pid_path: paths.pid_path.clone(),
        firecracker_log: cloud_hypervisor_log_path(paths),
        stdout_log: paths.stdout_log.clone(),
        stderr_log: paths.stderr_log.clone(),
        runtime_class: None,
        attached_volumes: Vec::new(),
        hosted_fleet_nodes: Vec::new(),
        guest_refresh_age_seconds: None,
        wedged_since_unix_s: None,
        wedge_class: None,
        recovery_attempts: RecoveryAttemptCounters::default(),
        last_recovery_action: None,
        recovery_state: RecoveryState::default(),
        detail,
    }
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
        runtime_class: None,
        attached_volumes: Vec::new(),
        hosted_fleet_nodes: Vec::new(),
        guest_refresh_age_seconds: None,
        wedged_since_unix_s: None,
        wedge_class: None,
        recovery_attempts: RecoveryAttemptCounters::default(),
        last_recovery_action: None,
        recovery_state: RecoveryState::default(),
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

pub(crate) struct DetachedForwardLaunchRequest<'a> {
    pub machine_name: &'a str,
    pub runtime_root: &'a Path,
    pub listen: &'a str,
    pub target: &'a str,
    pub name: Option<&'a str>,
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
        runtime_class: status.runtime_class,
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
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
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

        let state = if process_is_live(manifest.pid)? {
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

pub(crate) fn start_detached_forward(
    config: &PortConfig,
    request: DetachedForwardLaunchRequest<'_>,
) -> Result<HostedDetachedForwardStatusContract> {
    let state_dir = guest_forward_state_dir(config, request.machine_name, request.runtime_root)?;
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("failed to create '{}'", state_dir.display()))?;

    let name = request
        .name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("forward-{}", detached_forward_timestamp()));
    let manifest_path = state_dir.join(format!("{name}.json"));
    let config_path = state_dir.join(format!("{name}.config.toml"));
    let stdout_log = state_dir.join(format!("{name}.stdout.log"));
    let stderr_log = state_dir.join(format!("{name}.stderr.log"));

    let mut daemon_config = config.clone();
    daemon_config.clusters.clear();
    daemon_config.k3s_clusters.clear();
    daemon_config.control_planes.clear();
    daemon_config.nodes.clear();
    daemon_config.host_groups.clear();

    fs::write(
        &config_path,
        daemon_config
            .to_toml_string()
            .context("failed to encode detached forward config")?,
    )
    .with_context(|| format!("failed to write '{}'", config_path.display()))?;

    let mut command = Command::new(detached_forward_executable()?);
    command
        .arg("--config")
        .arg(&config_path)
        .arg("internal")
        .arg("forward-daemon")
        .arg("--machine")
        .arg(request.machine_name)
        .arg("--runtime-root")
        .arg(request.runtime_root)
        .arg("--listen")
        .arg(request.listen)
        .arg("--target")
        .arg(request.target)
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--name")
        .arg(&name)
        .stdin(Stdio::null())
        .stdout(
            File::create(&stdout_log)
                .with_context(|| format!("failed to create '{}'", stdout_log.display()))?,
        )
        .stderr(
            File::create(&stderr_log)
                .with_context(|| format!("failed to create '{}'", stderr_log.display()))?,
        );
    configure_detached_session(&mut command);
    let child = command
        .spawn()
        .context("failed to start detached forward daemon")?;

    let manifest = wait_for_detached_forward_manifest(
        &manifest_path,
        DetachedForwardManifestRecord {
            name,
            machine: request.machine_name.to_string(),
            pid: child.id(),
            listen: request.listen.to_string(),
            target: request.target.to_string(),
            stdout_log,
            stderr_log,
        },
    )?;

    hosted_detached_forward_status_from_manifest(manifest, manifest_path)
}

pub(crate) fn list_detached_forwards(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &Path,
) -> Result<Vec<HostedDetachedForwardStatusContract>> {
    let runtime_root = resolve_guest_runtime_root(config, machine_name, runtime_root)?;
    let runtime_dir = RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir;
    load_detached_forward_statuses(&runtime_dir, machine_name)?
        .into_iter()
        .map(hosted_detached_forward_status_from_runtime)
        .collect()
}

pub(crate) fn stop_detached_forward(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &Path,
    forward_name: &str,
) -> Result<HostedDetachedForwardStopResult> {
    let state_dir = guest_forward_state_dir(config, machine_name, runtime_root)?;
    let manifest_path = state_dir.join(format!("{forward_name}.json"));
    let bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
    let manifest: DetachedForwardManifestRecord = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse '{}'", manifest_path.display()))?;

    if process_is_live(manifest.pid)? {
        terminate_pid(manifest.pid)?;
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
        fs::remove_file(&manifest_path)
            .with_context(|| format!("failed to remove '{}'", manifest_path.display()))?;
    }
    let config_path = state_dir.join(format!("{forward_name}.config.toml"));
    if config_path.exists() {
        fs::remove_file(&config_path)
            .with_context(|| format!("failed to remove '{}'", config_path.display()))?;
    }

    Ok(HostedDetachedForwardStopResult {
        name: manifest.name,
        state: HostedDetachedForwardState::Stopped,
        pid: Some(manifest.pid),
    })
}

fn wait_for_detached_forward_manifest(
    manifest_path: &Path,
    fallback: DetachedForwardManifestRecord,
) -> Result<DetachedForwardManifestRecord> {
    for _ in 0..100 {
        if manifest_path.exists() {
            let bytes = fs::read(manifest_path)
                .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
            let manifest: DetachedForwardManifestRecord = serde_json::from_slice(&bytes)
                .with_context(|| {
                    format!(
                        "failed to parse detached forward manifest '{}'",
                        manifest_path.display()
                    )
                })?;
            return Ok(manifest);
        }
        thread::sleep(Duration::from_millis(20));
    }

    Ok(fallback)
}

fn hosted_detached_forward_status_from_runtime(
    status: DetachedForwardStatus,
) -> Result<HostedDetachedForwardStatusContract> {
    Ok(HostedDetachedForwardStatusContract {
        name: status.name,
        state: hosted_detached_forward_state(status.state)?,
        pid: status.pid,
        listen: status.listen,
        target: status.target,
        manifest_path: status.manifest_path,
        stdout_log: status.stdout_log,
        stderr_log: status.stderr_log,
        detail: status.detail,
    })
}

fn hosted_detached_forward_status_from_manifest(
    manifest: DetachedForwardManifestRecord,
    manifest_path: PathBuf,
) -> Result<HostedDetachedForwardStatusContract> {
    Ok(HostedDetachedForwardStatusContract {
        name: manifest.name,
        state: HostedDetachedForwardState::Running,
        pid: Some(manifest.pid),
        listen: manifest.listen,
        target: manifest.target,
        manifest_path,
        stdout_log: manifest.stdout_log,
        stderr_log: manifest.stderr_log,
        detail: String::from("detached forward process is live"),
    })
}

fn hosted_detached_forward_state(state: MachineRuntimeState) -> Result<HostedDetachedForwardState> {
    match state {
        MachineRuntimeState::Running => Ok(HostedDetachedForwardState::Running),
        MachineRuntimeState::Stale => Ok(HostedDetachedForwardState::Stale),
        other => bail!("unsupported detached forward state '{other}'"),
    }
}

fn terminate_pid(pid: u32) -> Result<()> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .with_context(|| format!("failed to signal pid {pid}"))?;
    if !status.success() {
        bail!("failed to stop detached forward pid {pid}");
    }
    Ok(())
}

fn detached_forward_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn detached_forward_executable() -> Result<PathBuf> {
    if let Ok(path) = env::var("PORT_DETACHED_FORWARD_EXECUTABLE") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(exe) = env::current_exe() {
        if exe.file_name().and_then(|value| value.to_str()) == Some("port") {
            return Ok(exe);
        }
    }

    if let Ok(root) = repo_root() {
        let workspace_port = root.join("target/debug/port");
        if workspace_port.is_file() {
            return Ok(workspace_port);
        }
    }

    if let Ok(exe) = env::current_exe() {
        return Ok(exe);
    }

    bail!("failed to resolve the current port executable")
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
    backend: ServiceSecretBackend,
    materialization: ServiceSecretMaterialization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceDefinitionRecord {
    machine_name: String,
    name: String,
    kind: ServiceKind,
    desired_state: ServiceDesiredState,
    command: Vec<String>,
    secret_bindings: Vec<ServiceSecretBinding>,
    policy: ServicePolicy,
    control: MachineControlContract,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
    host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    target_host_group: Option<String>,
    scheduler: Option<HostedSchedulerPolicy>,
    created_at_unix_s: u64,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceRuntimeRecord {
    state: ServiceRuntimeState,
    #[serde(default)]
    restart_count: u32,
    pid: Option<u32>,
    exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_exit_detail: Option<String>,
    #[serde(default)]
    health_state: ServiceHealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    health_detail: Option<String>,
    stdout_path: Option<PathBuf>,
    stderr_path: Option<PathBuf>,
    detail: String,
}

#[derive(Debug, Clone)]
struct ResolvedMachineRuntime {
    status: MachineStatus,
    control_plane: Option<String>,
    node_name: Option<String>,
    host_groups: Vec<String>,
    host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    target_host_group: Option<String>,
    scheduler: Option<HostedSchedulerPolicy>,
}

fn service_state_dir(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("services")
}

fn service_secret_dir(runtime_dir: &Path) -> PathBuf {
    service_state_dir(runtime_dir).join("secrets")
}

fn service_secret_backend_dir(runtime_dir: &Path, backend: ServiceSecretBackend) -> PathBuf {
    match backend {
        ServiceSecretBackend::RuntimeFile => service_secret_dir(runtime_dir).join("runtime-file"),
    }
}

fn service_secret_metadata_path(runtime_dir: &Path, secret_name: &str) -> PathBuf {
    service_secret_dir(runtime_dir).join(format!("{secret_name}.json"))
}

fn service_secret_value_path(
    runtime_dir: &Path,
    secret_name: &str,
    backend: ServiceSecretBackend,
) -> PathBuf {
    service_secret_backend_dir(runtime_dir, backend).join(secret_name)
}

fn service_runtime_dir(runtime_dir: &Path) -> PathBuf {
    service_state_dir(runtime_dir).join("runtime")
}

fn service_definition_dir(runtime_dir: &Path) -> PathBuf {
    service_state_dir(runtime_dir).join("definitions")
}

fn service_runtime_record_path(runtime_dir: &Path, service_name: &str) -> PathBuf {
    service_runtime_dir(runtime_dir).join(format!("{service_name}.json"))
}

fn default_service_runtime_observation(
    runtime_dir: &Path,
    service_name: &str,
) -> ServiceRuntimeObservation {
    ServiceRuntimeObservation {
        state: ServiceRuntimeState::Stored,
        record_path: service_runtime_record_path(runtime_dir, service_name),
        restart_count: 0,
        pid: None,
        exit_code: None,
        last_exit_code: None,
        last_exit_detail: None,
        health_state: ServiceHealthState::Unknown,
        health_detail: None,
        stdout_path: None,
        stderr_path: None,
    }
}

fn read_service_runtime_record(
    runtime_dir: &Path,
    service_name: &str,
) -> Result<Option<ServiceRuntimeRecord>> {
    let path = service_runtime_record_path(runtime_dir, service_name);
    if !path.exists() {
        return Ok(None);
    }
    read_json_file(&path).map(Some)
}

fn write_service_runtime_record(
    runtime_dir: &Path,
    service_name: &str,
    record: &ServiceRuntimeRecord,
) -> Result<()> {
    let dir = service_runtime_dir(runtime_dir);
    fs::create_dir_all(&dir).with_context(|| {
        format!(
            "failed to create service runtime directory '{}'",
            dir.display()
        )
    })?;
    write_json_file(
        &service_runtime_record_path(runtime_dir, service_name),
        record,
    )
}

fn write_secret_value_file(path: &Path, value: &str) -> Result<()> {
    fs::write(path, value)
        .with_context(|| format!("failed to write secret backend value '{}'", path.display()))?;
    let mut permissions = fs::metadata(path)
        .with_context(|| {
            format!(
                "failed to inspect secret backend value '{}'",
                path.display()
            )
        })?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions).with_context(|| {
        format!(
            "failed to harden secret backend permissions for '{}'",
            path.display()
        )
    })
}

fn read_secret_value_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("failed to read secret backend value '{}'", path.display()))
}

fn resolve_machine_runtime(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    host_group: Option<&str>,
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
                host_group_policies: BTreeMap::new(),
                target_host_group: None,
                scheduler: None,
            }),
            HostConnection::HostedControlPlane { .. } => {
                if let Some(host_group) = host_group {
                    return resolve_targeted_hosted_service_runtime(
                        config,
                        runtime_root,
                        machine_name,
                        host_group,
                    );
                }
                let resolution = hosted_machine_resolution(config, machine_name)?;
                Ok(ResolvedMachineRuntime {
                    status: resolution.status,
                    control_plane: Some(resolution.control_plane),
                    node_name: resolution.node_name,
                    host_groups: resolution.host_groups,
                    host_group_policies: resolution.host_group_policies,
                    target_host_group: None,
                    scheduler: None,
                })
            }
            HostConnection::Ssh {
                destination,
                user,
                port,
            } => {
                bail!(
                    "machine '{}' targets ssh-managed host '{}' through {}@{}:{} but ssh-managed lifecycle is not implemented yet",
                    machine_name,
                    machine.host,
                    user,
                    destination,
                    port
                )
            }
        };
    }

    Ok(ResolvedMachineRuntime {
        status: firecracker_local_machine_status(runtime_root, machine_name)?,
        control_plane: None,
        node_name: None,
        host_groups: Vec::new(),
        host_group_policies: BTreeMap::new(),
        target_host_group: None,
        scheduler: None,
    })
}

fn resolve_service_runtime_context(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    host_group: Option<&str>,
) -> Result<ResolvedMachineRuntime> {
    let machine_is_hosted = machine_is_hosted(config, machine_name)?;
    if host_group.is_none()
        && let Some(context) =
            resolve_localized_hosted_service_runtime_context(config, runtime_root, machine_name)?
    {
        return Ok(context);
    }
    if host_group.is_none()
        && let Some(context) =
            resolve_stored_local_hosted_service_runtime_context(config, machine_name)?
    {
        return Ok(context);
    }
    let context = resolve_machine_runtime(config, runtime_root, machine_name, host_group)?;
    if context.status.state == MachineRuntimeState::Malformed {
        bail!(
            "service operations require well-formed runtime state for machine '{}': {}",
            machine_name,
            context.status.detail
        );
    }
    if !machine_is_hosted && !context.status.runtime_dir.exists() {
        bail!(
            "service operations require an existing Port runtime for machine '{}': {}",
            machine_name,
            context.status.detail
        );
    }
    Ok(context)
}

fn resolve_localized_hosted_service_runtime_context(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<Option<ResolvedMachineRuntime>> {
    if !machine_is_hosted(config, machine_name)? {
        return Ok(None);
    }

    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let hosted_identity = effective_config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let Some(summary) = effective_config.hosted_machine_summary_contract(machine_name)? else {
        return Ok(None);
    };
    let inventory = effective_config.hosted_inventory_contract()?;
    let Some((node_name, _node)) = inventory
        .nodes
        .iter()
        .find(|(_, node)| node.runtime_root == runtime_root)
    else {
        return Ok(None);
    };

    let mut status = match local_machine_status_for_runtime_root(
        &effective_config,
        runtime_root,
        machine_name,
    ) {
        Ok(status) => status,
        Err(_) => return Ok(None),
    };
    status.control = MachineControlContract::hosted_control_plane();
    status.detail = format!(
        "{} Routed through control plane '{}' and node '{}'. {}",
        status.detail, hosted_identity.control_plane, node_name, summary.placement_detail
    );

    Ok(Some(ResolvedMachineRuntime {
        status,
        control_plane: Some(hosted_identity.control_plane),
        node_name: Some(node_name.clone()),
        host_groups: summary.host_groups.clone(),
        host_group_policies: summary.host_group_policies.clone(),
        target_host_group: None,
        scheduler: None,
    }))
}

fn resolve_stored_local_hosted_service_runtime_context(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Option<ResolvedMachineRuntime>> {
    if !machine_is_hosted(config, machine_name)? {
        return Ok(None);
    }

    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let hosted_identity = effective_config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let Some(summary) = effective_config.hosted_machine_summary_contract(machine_name)? else {
        return Ok(None);
    };
    let Some(placement) = hosted_stored_machine_placement(config, machine_name)? else {
        return Ok(None);
    };
    let placement_detail = placement
        .placement_detail
        .clone()
        .unwrap_or_else(|| summary.placement_detail.clone());
    let mut status = match local_machine_status_for_runtime_root(
        &effective_config,
        &placement.runtime_root,
        machine_name,
    ) {
        Ok(status) => status,
        Err(_) => {
            let response = match hosted_control_plane_machine_status_response(config, machine_name)
            {
                Ok(response) => response,
                Err(_) => return Ok(None),
            };
            let mut status = response.result;
            status.control = MachineControlContract::hosted_control_plane();
            status.detail = match response.route.node_name.as_deref() {
                Some(node_name) => format!(
                    "{} Routed through control plane '{}' and stored node '{}'. {}",
                    status.detail, hosted_identity.control_plane, node_name, placement_detail
                ),
                None => format!(
                    "{} Routed through control plane '{}' and stored node '{}'. {}",
                    status.detail,
                    hosted_identity.control_plane,
                    placement.node_name,
                    placement_detail
                ),
            };
            return Ok(Some(ResolvedMachineRuntime {
                status,
                control_plane: Some(hosted_identity.control_plane),
                node_name: response
                    .route
                    .node_name
                    .or_else(|| Some(placement.node_name.clone())),
                host_groups: summary.host_groups.clone(),
                host_group_policies: summary.host_group_policies.clone(),
                target_host_group: None,
                scheduler: None,
            }));
        }
    };
    status.control = MachineControlContract::hosted_control_plane();
    status.detail = format!(
        "{} Routed through control plane '{}' and stored node '{}'. {}",
        status.detail, hosted_identity.control_plane, placement.node_name, placement_detail
    );

    Ok(Some(ResolvedMachineRuntime {
        status,
        control_plane: Some(hosted_identity.control_plane),
        node_name: Some(placement.node_name),
        host_groups: summary.host_groups.clone(),
        host_group_policies: summary.host_group_policies.clone(),
        target_host_group: None,
        scheduler: None,
    }))
}

fn local_machine_status_for_runtime_root(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineStatus> {
    let machine = config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{}'", machine_name))?;
    match machine.substrate {
        ExecutionSubstrate::Firecracker => {
            firecracker_local_machine_status(runtime_root, machine_name)
        }
        ExecutionSubstrate::CloudHypervisor => {
            cloud_hypervisor_local_machine_status(runtime_root, machine_name)
        }
        ExecutionSubstrate::Avf => avf_local_machine_status(runtime_root, machine_name),
    }
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

#[derive(Debug, Clone)]
struct StoredMachineSecret {
    record: MachineSecretRecord,
    path: PathBuf,
    backend_path: PathBuf,
}

fn load_machine_secret(runtime_dir: &Path, secret_name: &str) -> Result<StoredMachineSecret> {
    validate_identifier(secret_name, "secret name")?;
    let path = service_secret_metadata_path(runtime_dir, secret_name);
    let record: MachineSecretRecord = read_json_file(&path)?;
    let backend_path = service_secret_value_path(runtime_dir, &record.name, record.backend);
    if !backend_path.exists() {
        bail!(
            "secret backend value for '{}' is missing under '{}'",
            secret_name,
            backend_path.display()
        );
    }
    Ok(StoredMachineSecret {
        record,
        path,
        backend_path,
    })
}

fn machine_secret_summary_from_storage(
    machine_name: &str,
    context: ResolvedMachineRuntime,
    secret: &StoredMachineSecret,
    detail: String,
) -> MachineSecretSummary {
    MachineSecretSummary {
        machine_name: machine_name.to_string(),
        name: secret.record.name.clone(),
        backend: secret.record.backend,
        materialization: secret.record.materialization,
        control: context.status.control,
        control_plane: context.control_plane,
        node_name: context.node_name,
        host_groups: context.host_groups,
        path: secret.path.clone(),
        backend_path: secret.backend_path.clone(),
        detail,
    }
}

fn service_secret_source_from_binding(
    runtime_dir: &Path,
    binding: &ServiceSecretBinding,
) -> Result<ServiceSecretSourceStatus> {
    let secret = load_machine_secret(runtime_dir, &binding.secret)?;
    Ok(ServiceSecretSourceStatus {
        env: binding.env.clone(),
        secret: binding.secret.clone(),
        backend: secret.record.backend,
        materialization: secret.record.materialization,
        path: secret.backend_path,
        detail: String::from(
            "secret value remains under the resolved runtime owner and is materialized for process launch without being embedded in service status",
        ),
    })
}

fn project_service_secret_sources(
    runtime_dir: &Path,
    bindings: &[ServiceSecretBinding],
) -> (Vec<ServiceSecretSourceStatus>, Option<String>) {
    let mut sources = Vec::new();
    let mut errors = Vec::new();
    for binding in bindings {
        match service_secret_source_from_binding(runtime_dir, binding) {
            Ok(source) => sources.push(source),
            Err(error) => errors.push(format!("{}={}: {error}", binding.env, binding.secret)),
        }
    }
    sources.sort_by(|left, right| {
        left.env
            .cmp(&right.env)
            .then(left.secret.cmp(&right.secret))
    });
    let detail = if errors.is_empty() {
        None
    } else {
        Some(format!(
            "Secret source projection could not resolve: {}.",
            errors.join("; ")
        ))
    };
    (sources, detail)
}

fn service_status_from_record(
    record: ServiceDefinitionRecord,
    manifest_path: PathBuf,
) -> ServiceDefinitionStatus {
    let runtime_dir = manifest_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let runtime_record = read_service_runtime_record(&runtime_dir, &record.name)
        .ok()
        .flatten();
    let (secret_sources, secret_projection_detail) =
        project_service_secret_sources(&runtime_dir, &record.secret_bindings);
    let runtime = runtime_record
        .as_ref()
        .map(|runtime| ServiceRuntimeObservation {
            state: runtime.state,
            record_path: service_runtime_record_path(&runtime_dir, &record.name),
            restart_count: runtime.restart_count,
            pid: runtime.pid,
            exit_code: runtime.exit_code,
            last_exit_code: runtime.last_exit_code,
            last_exit_detail: runtime.last_exit_detail.clone(),
            health_state: runtime.health_state,
            health_detail: runtime.health_detail.clone(),
            stdout_path: runtime.stdout_path.clone(),
            stderr_path: runtime.stderr_path.clone(),
        })
        .unwrap_or_else(|| default_service_runtime_observation(&runtime_dir, &record.name));
    let mut detail = runtime_record
        .map(|runtime| runtime.detail)
        .unwrap_or(record.detail);
    if let Some(secret_projection_detail) = secret_projection_detail {
        detail = format!("{detail} {secret_projection_detail}");
    }
    ServiceDefinitionStatus {
        machine_name: record.machine_name,
        name: record.name,
        kind: record.kind,
        desired_state: record.desired_state,
        runtime,
        command: record.command,
        secret_bindings: record.secret_bindings,
        secret_sources,
        policy: record.policy,
        control: record.control,
        control_plane: record.control_plane,
        node_name: record.node_name,
        host_groups: record.host_groups,
        host_group_policies: record.host_group_policies,
        target_host_group: record.target_host_group,
        scheduler: record.scheduler,
        manifest_path,
        detail,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HostedStoredServicePlacement {
    pub status: ServiceDefinitionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct HostedMachinePlacementStateFile {
    control_plane: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    machines: BTreeMap<String, HostedMachinePlacementRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HostedMachinePlacementRecord {
    node_name: String,
    runtime_root: PathBuf,
    placed_at_unix_s: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    placement_detail: Option<String>,
}

pub(crate) fn hosted_stored_service_placements(
    config: &PortConfig,
    machine_name: &str,
    service_name: Option<&str>,
) -> Result<Vec<HostedStoredServicePlacement>> {
    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let machine = effective_config
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{}'", machine_name))?;
    let inventory = effective_config.hosted_inventory_contract()?;
    let mut placements = Vec::new();
    let mut candidate_roots = Vec::<(Option<String>, PathBuf)>::new();
    let mut seen = BTreeSet::<(Option<String>, PathBuf)>::new();
    let inventory_roots = inventory
        .nodes
        .iter()
        .filter(|(_, node)| node.host == machine.host)
        .map(|(node_name, node)| (Some(node_name.clone()), node.runtime_root.clone()))
        .collect::<Vec<_>>();

    let had_stored_machine_placement =
        if let Some(placement) = hosted_stored_machine_placement(config, machine_name)? {
            candidate_roots.push((Some(placement.node_name), placement.runtime_root));
            true
        } else {
            false
        };

    if !had_stored_machine_placement {
        candidate_roots.extend(inventory_roots.iter().cloned());
    }

    for (node_name, runtime_root) in candidate_roots {
        let dedupe_key = (node_name.clone(), runtime_root.clone());
        if !seen.insert(dedupe_key) {
            continue;
        }
        placements.extend(read_hosted_service_placements_from_runtime(
            machine_name,
            node_name.as_deref(),
            &runtime_root,
            service_name,
        )?);
    }

    if placements.is_empty() && had_stored_machine_placement {
        for (node_name, runtime_root) in inventory_roots {
            let dedupe_key = (node_name.clone(), runtime_root.clone());
            if !seen.insert(dedupe_key) {
                continue;
            }
            placements.extend(read_hosted_service_placements_from_runtime(
                machine_name,
                node_name.as_deref(),
                &runtime_root,
                service_name,
            )?);
        }
    }

    placements.sort_by(|left, right| {
        left.status
            .name
            .cmp(&right.status.name)
            .then(left.status.node_name.cmp(&right.status.node_name))
    });
    Ok(placements)
}

fn read_hosted_service_placements_from_runtime(
    machine_name: &str,
    node_name: Option<&str>,
    runtime_root: &Path,
    service_name: Option<&str>,
) -> Result<Vec<HostedStoredServicePlacement>> {
    let definitions =
        service_definition_dir(&RuntimePaths::for_machine(runtime_root, machine_name).runtime_dir);
    if !definitions.exists() {
        return Ok(Vec::new());
    }

    let mut placements = Vec::new();
    if let Some(service_name) = service_name {
        let path = definitions.join(format!("{service_name}.json"));
        if !path.exists() {
            return Ok(Vec::new());
        }
        let record: ServiceDefinitionRecord = read_json_file(&path)?;
        let mut status = service_status_from_record(record, path);
        if status.node_name.is_none() {
            status.node_name = node_name.map(ToOwned::to_owned);
        }
        placements.push(HostedStoredServicePlacement { status });
        return Ok(placements);
    }

    for entry in fs::read_dir(&definitions)
        .with_context(|| format!("failed to read '{}'", definitions.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to inspect hosted service definitions in '{}'",
                definitions.display()
            )
        })?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let path = entry.path();
        let record: ServiceDefinitionRecord = read_json_file(&path)?;
        let mut status = service_status_from_record(record, path);
        if status.node_name.is_none() {
            status.node_name = node_name.map(ToOwned::to_owned);
        }
        placements.push(HostedStoredServicePlacement { status });
    }

    Ok(placements)
}

fn hosted_stored_machine_placement(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Option<HostedMachinePlacementRecord>> {
    let hosted_identity = config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let path = hosted_placeholder_runtime_root_for_config(config, &hosted_identity.control_plane)
        .join("machine-placements.json");
    if !path.exists() {
        return Ok(None);
    }

    let placements: HostedMachinePlacementStateFile = read_json_file(&path)?;
    Ok(placements.machines.get(machine_name).cloned())
}

fn persist_local_hosted_machine_placement_from_route(
    config: &PortConfig,
    machine_name: &str,
    route: &HostedRouteContext,
    launched_at_unix_s: u64,
) -> Result<()> {
    let hosted_identity = config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let node_name = route.node_name.as_ref().with_context(|| {
        format!(
            "hosted launch for machine '{}' did not include a selected node",
            machine_name
        )
    })?;
    let runtime_root = route.runtime_root.as_ref().with_context(|| {
        format!(
            "hosted launch for machine '{}' did not include a selected runtime root",
            machine_name
        )
    })?;
    let state_path =
        hosted_placeholder_runtime_root_for_config(config, &hosted_identity.control_plane)
            .join("machine-placements.json");
    let mut placements: HostedMachinePlacementStateFile = if state_path.exists() {
        read_json_file(&state_path)?
    } else {
        HostedMachinePlacementStateFile::default()
    };
    placements.control_plane = hosted_identity.control_plane;
    placements.machines.insert(
        machine_name.to_string(),
        HostedMachinePlacementRecord {
            node_name: node_name.clone(),
            runtime_root: runtime_root.clone(),
            placed_at_unix_s: launched_at_unix_s,
            placement_detail: route.placement_detail.clone(),
        },
    );
    let parent = state_path.parent().with_context(|| {
        format!(
            "machine placement state path '{}' has no parent directory",
            state_path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create hosted placement state directory '{}'",
            parent.display()
        )
    })?;
    write_json_file(&state_path, &placements)
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
    host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    runtime_root: PathBuf,
    status: MachineStatus,
}

#[derive(Debug, Clone, Deserialize)]
struct HostedImportedInventoryStateFile {
    control_plane: String,
    nodes: BTreeMap<String, HostedImportedNodeRecord>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RegisteredNodeStateFile {
    control_plane: String,
    nodes: BTreeMap<String, port_model::HostedNodeRegistration>,
}

pub(crate) fn hosted_placeholder_runtime_root_for_config(
    config: &PortConfig,
    control_plane: &str,
) -> PathBuf {
    config
        .state_root()
        .map(|root| root.join(".port/hosted"))
        .unwrap_or_else(|| PathBuf::from(".port/hosted"))
        .join(control_plane)
}

fn hosted_placeholder_runtime_root(control_plane: &str) -> PathBuf {
    PathBuf::from(".port/hosted").join(control_plane)
}

fn hosted_imported_inventory_state_path(config: &PortConfig, control_plane: &str) -> PathBuf {
    hosted_placeholder_runtime_root_for_config(config, control_plane)
        .join("imported-inventory.json")
}

fn read_hosted_imported_inventory_state(
    config: &PortConfig,
    control_plane: &str,
) -> Result<Option<HostedImportedInventoryStateFile>> {
    let path = hosted_imported_inventory_state_path(config, control_plane);
    if !path.exists() {
        return Ok(None);
    }

    let state: HostedImportedInventoryStateFile = read_json_file(&path)?;
    if state.control_plane != control_plane {
        bail!(
            "imported inventory state '{}' belongs to control plane '{}', not '{}'",
            path.display(),
            state.control_plane,
            control_plane
        );
    }

    Ok(Some(state))
}

fn read_hosted_registered_node_state(
    config: &PortConfig,
    control_plane: &str,
) -> Result<Option<RegisteredNodeStateFile>> {
    let path = hosted_placeholder_runtime_root_for_config(config, control_plane)
        .join("registered-nodes.json");
    if !path.exists() {
        return Ok(None);
    }

    let state: RegisteredNodeStateFile = read_json_file(&path)?;
    if state.control_plane != control_plane {
        bail!(
            "registered node state '{}' belongs to control plane '{}', not '{}'",
            path.display(),
            state.control_plane,
            control_plane
        );
    }

    Ok(Some(state))
}

fn hosted_control_plane_for_node<'a>(config: &'a PortConfig, node_name: &str) -> Option<&'a str> {
    let node = config.nodes.get(node_name)?;
    let host = config.hosts.get(&node.host)?;
    match &host.connection {
        HostConnection::HostedControlPlane { control_plane } => Some(control_plane.as_str()),
        HostConnection::Local => None,
        HostConnection::Ssh { .. } => None,
    }
}

fn hosted_imported_pvm_package_attachment(
    imported: &HostedImportedNodeRecord,
    architecture: MachineArchitecture,
) -> Option<&HostedPvmHostKitPackageAttachment> {
    imported
        .pvm_host_kit_packages
        .iter()
        .find(|attachment| attachment.architecture == architecture)
}

fn hosted_pvm_preparation_hint(
    control_plane: Option<&str>,
    node_name: &str,
    architecture: MachineArchitecture,
) -> String {
    let mut command = String::from("port control-plane prepare-pvm-node");
    if let Some(control_plane) = control_plane {
        command.push_str(&format!(" --control-plane {control_plane}"));
    }
    command.push_str(&format!(
        " --node {node_name} --architecture {}",
        architecture_dir(architecture)
    ));
    format!("Prepare the node with `{command}`.")
}

fn hosted_pvm_target_machines(
    config: &PortConfig,
    node_name: &str,
    architecture: MachineArchitecture,
) -> Vec<String> {
    let Some(node) = config.nodes.get(node_name) else {
        return Vec::new();
    };

    config
        .machines
        .iter()
        .filter_map(|(machine_name, machine)| {
            if machine.host != node.host || machine.substrate != ExecutionSubstrate::Firecracker {
                return None;
            }
            let machine_architecture = resolve_machine_architecture(machine.architecture).ok()?;
            (machine_architecture == architecture && machine.protection_mode == ProtectionMode::Pvm)
                .then(|| machine_name.clone())
        })
        .collect()
}

fn hosted_pvm_preparation_guidance(
    config: &PortConfig,
    control_plane: Option<&str>,
    node_name: &str,
    architecture: MachineArchitecture,
) -> String {
    let hint = hosted_pvm_preparation_hint(control_plane, node_name, architecture);
    let machines = hosted_pvm_target_machines(config, node_name, architecture);
    if machines.is_empty() {
        hint
    } else {
        let targets = machines
            .into_iter()
            .map(|machine| format!("`{machine}`"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{hint} This imported readiness gates the hosted PVM lane for {targets}.")
    }
}

fn effective_config_with_hosted_imported_inventory(config: &PortConfig) -> Result<PortConfig> {
    let mut effective = config.clone();
    let mut imported_by_control_plane =
        BTreeMap::<String, Option<HostedImportedInventoryStateFile>>::new();

    for node_name in config.nodes.keys() {
        if let Some(node) = effective.nodes.get_mut(node_name) {
            node.capabilities = node.capabilities.without_imported_pvm_readiness();
        }
        let Some(control_plane) = hosted_control_plane_for_node(config, node_name) else {
            continue;
        };

        if !imported_by_control_plane.contains_key(control_plane) {
            imported_by_control_plane.insert(
                control_plane.to_string(),
                read_hosted_imported_inventory_state(config, control_plane).with_context(|| {
                    format!(
                        "failed to load imported hosted inventory for control plane '{}'",
                        control_plane
                    )
                })?,
            );
        }

        let Some(Some(imported_state)) = imported_by_control_plane.get(control_plane) else {
            continue;
        };
        let Some(imported) = imported_state.nodes.get(node_name) else {
            continue;
        };
        if let Some(node) = effective.nodes.get_mut(node_name) {
            node.capabilities = imported.capability_summary.clone();
        }
    }

    Ok(effective)
}

fn hosted_pvm_lane_check_without_imported_record(
    config: &PortConfig,
    node_name: &str,
    control_plane: Option<&str>,
    lane: &HostedPvmCapability,
) -> DoctorCheck {
    let name = format!(
        "pvm:{node_name}:{}:host-kit-contract",
        architecture_dir(lane.architecture)
    );

    match lane.state {
        PvmCapabilityState::ResearchOnly => DoctorCheck {
            name,
            ok: false,
            required: false,
            detail: format!(
                "Hosted node '{}' keeps the {} PVM lane research-only.",
                node_name,
                architecture_dir(lane.architecture)
            ),
        },
        PvmCapabilityState::Planned => match lane.host_kit.as_ref() {
            Some(host_kit) => {
                if let Some(detail) = pvm_host_kit_contract_issue(lane.architecture, host_kit) {
                    DoctorCheck {
                        name,
                        ok: false,
                        required: false,
                        detail,
                    }
                } else {
                    DoctorCheck {
                        name,
                        ok: false,
                        required: false,
                        detail: format!(
                            "Hosted node '{}' declares {} but imported hosted PVM readiness is missing, so the lane remains planned. {}",
                            node_name,
                            pvm_host_kit_contract_detail(host_kit),
                            hosted_pvm_preparation_guidance(
                                config,
                                control_plane,
                                node_name,
                                lane.architecture,
                            )
                        ),
                    }
                }
            }
            None => DoctorCheck {
                name,
                ok: false,
                required: false,
                detail: format!(
                    "Hosted node '{}' remains PVM-planned without a provider-backed host-kit contract. Port cannot import readiness for this lane, and the standard Firecracker lane is not a fallback.",
                    node_name
                ),
            },
        },
        PvmCapabilityState::Ready => match lane.host_kit.as_ref() {
            Some(host_kit) => {
                if let Some(detail) = pvm_host_kit_contract_issue(lane.architecture, host_kit) {
                    DoctorCheck {
                        name,
                        ok: false,
                        required: false,
                        detail,
                    }
                } else {
                    DoctorCheck {
                        name,
                        ok: false,
                        required: false,
                        detail: format!(
                            "Hosted node '{}' declares {} in configured inventory, but Port requires imported hosted PVM readiness before the lane is treated as prepared. {}",
                            node_name,
                            pvm_host_kit_contract_detail(host_kit),
                            hosted_pvm_preparation_guidance(
                                config,
                                control_plane,
                                node_name,
                                lane.architecture,
                            )
                        ),
                    }
                }
            }
            None => DoctorCheck {
                name,
                ok: false,
                required: false,
                detail: format!(
                    "Hosted node '{}' advertises a ready PVM lane without a host-kit contract. Imported readiness cannot be trusted until the host-kit contract is fixed.",
                    node_name
                ),
            },
        },
    }
}

fn hosted_pvm_lane_check_from_imported_record(
    node_name: &str,
    control_plane: Option<&str>,
    lane: &HostedPvmCapability,
    imported: &HostedImportedNodeRecord,
) -> DoctorCheck {
    let name = format!(
        "pvm:{node_name}:{}:host-kit-contract",
        architecture_dir(lane.architecture)
    );

    match lane.state {
        PvmCapabilityState::ResearchOnly => DoctorCheck {
            name,
            ok: false,
            required: false,
            detail: format!(
                "Hosted node '{}' imported from '{}' still marks the {} PVM lane as research-only. {}",
                node_name,
                imported.provenance,
                architecture_dir(lane.architecture),
                hosted_pvm_preparation_hint(control_plane, node_name, lane.architecture)
            ),
        },
        PvmCapabilityState::Planned => DoctorCheck {
            name,
            ok: false,
            required: false,
            detail: format!(
                "Hosted node '{}' remains PVM-planned in imported inventory from '{}'. {}",
                node_name,
                imported.provenance,
                hosted_pvm_preparation_hint(control_plane, node_name, lane.architecture)
            ),
        },
        PvmCapabilityState::Ready => match lane.host_kit.as_ref() {
            Some(host_kit) => {
                if let Some(detail) = pvm_host_kit_contract_issue(lane.architecture, host_kit) {
                    DoctorCheck {
                        name,
                        ok: false,
                        required: false,
                        detail: format!(
                            "Hosted node '{}' imported from '{}' advertises an invalid prepared PVM host-kit contract: {}",
                            node_name, imported.provenance, detail
                        ),
                    }
                } else {
                    match hosted_imported_pvm_package_attachment(imported, lane.architecture) {
                        Some(attachment) if attachment.package == host_kit.package => DoctorCheck {
                            name,
                            ok: true,
                            required: false,
                            detail: format!(
                                "Hosted node '{}' imported from '{}' is prepared with package {}@{} and {}",
                                node_name,
                                imported.provenance,
                                attachment.package.name,
                                attachment.package.version,
                                pvm_host_kit_contract_detail(host_kit)
                            ),
                        },
                        Some(attachment) => DoctorCheck {
                            name,
                            ok: false,
                            required: false,
                            detail: format!(
                                "Hosted node '{}' imported from '{}' advertises prepared package {}@{}, but the ready host-kit contract resolves to {}@{}.",
                                node_name,
                                imported.provenance,
                                attachment.package.name,
                                attachment.package.version,
                                host_kit.package.name,
                                host_kit.package.version
                            ),
                        },
                        None => DoctorCheck {
                            name,
                            ok: false,
                            required: false,
                            detail: format!(
                                "Hosted node '{}' imported from '{}' advertises a ready PVM lane without a matching host-kit package attachment.",
                                node_name, imported.provenance
                            ),
                        },
                    }
                }
            }
            None => DoctorCheck {
                name,
                ok: false,
                required: false,
                detail: format!(
                    "Hosted node '{}' imported from '{}' advertises a ready PVM lane without a host-kit contract.",
                    node_name, imported.provenance
                ),
            },
        },
    }
}

fn hosted_machine_resolution(
    config: &PortConfig,
    machine_name: &str,
) -> Result<HostedMachineResolution> {
    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let control = effective_config.machine_control_contract(machine_name)?;
    let hosted_identity = effective_config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let placeholder_root =
        hosted_placeholder_runtime_root_for_config(config, &hosted_identity.control_plane);

    let summary = match effective_config.hosted_machine_summary_contract(machine_name) {
        Ok(Some(summary)) => summary,
        Ok(None) => {
            return Ok(HostedMachineResolution {
                control_plane: hosted_identity.control_plane.clone(),
                node_name: None,
                host_groups: Vec::new(),
                host_group_policies: BTreeMap::new(),
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
                host_group_policies: BTreeMap::new(),
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
            host_group_policies: summary.host_group_policies.clone(),
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

    let inventory = effective_config.hosted_inventory_contract()?;
    let stored_placement = hosted_stored_machine_placement(config, machine_name)?;
    if let Ok(response) = hosted_control_plane_machine_status_response(config, machine_name) {
        let route_conflicts_with_stored_placement =
            stored_placement.as_ref().is_some_and(|placement| {
                response.route.node_name.as_deref() != Some(placement.node_name.as_str())
                    || response.route.runtime_root.as_ref() != Some(&placement.runtime_root)
            });
        if response.result.state == MachineRuntimeState::Malformed
            && route_conflicts_with_stored_placement
        {
            // A fallback candidate can still return a synthetic malformed machine status
            // when the real stored placement is currently unreachable. Keep the stored
            // placement truth in that case instead of letting the fallback candidate
            // rewrite cluster-status surfaces onto the wrong node.
        } else {
            let mut status = response.result;
            let node_name = response.route.node_name.clone().or_else(|| {
                stored_placement
                    .as_ref()
                    .map(|placement| placement.node_name.clone())
            });
            let runtime_root = response
                .route
                .runtime_root
                .clone()
                .or_else(|| {
                    stored_placement
                        .as_ref()
                        .map(|placement| placement.runtime_root.clone())
                })
                .or_else(|| runtime_root_from_machine_status(&status))
                .unwrap_or_else(|| placeholder_root.clone());
            let placement_detail = stored_placement
                .as_ref()
                .and_then(|placement| placement.placement_detail.clone())
                .unwrap_or_else(|| summary.placement_detail.clone());
            status.detail = match node_name.as_deref() {
                Some(node_name) => format!(
                    "{} Routed through control plane '{}' and node '{}'. {}",
                    status.detail, summary.control_plane, node_name, placement_detail
                ),
                None => format!(
                    "{} Routed through control plane '{}'. {}",
                    status.detail, summary.control_plane, placement_detail
                ),
            };

            return Ok(HostedMachineResolution {
                control_plane: summary.control_plane.clone(),
                node_name,
                host_groups: summary.host_groups.clone(),
                host_group_policies: summary.host_group_policies.clone(),
                runtime_root,
                status,
            });
        }
    }

    if let Some(placement) = stored_placement {
        let paths = RuntimePaths::for_machine(&placement.runtime_root, machine_name);
        let placement_detail = placement
            .placement_detail
            .clone()
            .unwrap_or_else(|| summary.placement_detail.clone());
        let mut status = if inventory.nodes.contains_key(&placement.node_name) {
            synthetic_machine_status(
                machine_name,
                &paths,
                control.clone(),
                MachineRuntimeState::Malformed,
                format!(
                    "control plane '{}' could not inspect stored placement on node '{}' through the live node-agent route",
                    summary.control_plane, placement.node_name
                ),
            )
        } else {
            synthetic_machine_status(
                machine_name,
                &paths,
                control.clone(),
                MachineRuntimeState::Malformed,
                format!(
                    "control plane '{}' resolved stored placement on unknown node '{}'",
                    summary.control_plane, placement.node_name
                ),
            )
        };
        status.detail = format!(
            "{} Routed through control plane '{}' and stored node '{}'. {}",
            status.detail, summary.control_plane, placement.node_name, placement_detail
        );

        return Ok(HostedMachineResolution {
            control_plane: summary.control_plane.clone(),
            node_name: Some(placement.node_name),
            host_groups: summary.host_groups.clone(),
            host_group_policies: summary.host_group_policies.clone(),
            runtime_root: placement.runtime_root,
            status,
        });
    }

    if let Some(node_name) = summary.candidate_nodes.first()
        && let Some(node) = inventory.nodes.get(node_name)
    {
        let runtime_root = node.runtime_root.clone();
        let mut status = synthetic_machine_status(
            machine_name,
            &RuntimePaths::for_machine(&runtime_root, machine_name),
            control,
            MachineRuntimeState::Malformed,
            format!(
                "control plane '{}' could not inspect node '{}' through a live node-agent route",
                summary.control_plane, node_name
            ),
        );
        status.detail = format!(
            "{} Routed through control plane '{}' and node '{}'. {}",
            status.detail, summary.control_plane, node_name, summary.placement_detail
        );
        return Ok(HostedMachineResolution {
            control_plane: summary.control_plane.clone(),
            node_name: Some(node_name.clone()),
            host_groups: summary.host_groups.clone(),
            host_group_policies: summary.host_group_policies.clone(),
            runtime_root,
            status,
        });
    }

    Ok(HostedMachineResolution {
        control_plane: summary.control_plane.clone(),
        node_name: None,
        host_groups: summary.host_groups.clone(),
        host_group_policies: summary.host_group_policies.clone(),
        runtime_root: placeholder_root.clone(),
        status: synthetic_machine_status(
            machine_name,
            &RuntimePaths::for_machine(&placeholder_root, machine_name),
            control,
            MachineRuntimeState::Malformed,
            format!(
                "control plane '{}' could not inspect machine '{}' through a live node-agent route. {}",
                summary.control_plane, machine_name, summary.placement_detail
            ),
        ),
    })
}

fn resolve_targeted_hosted_service_runtime(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    host_group: &str,
) -> Result<ResolvedMachineRuntime> {
    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let control = effective_config.machine_control_contract(machine_name)?;
    let hosted_identity = effective_config
        .hosted_api_identity_contract(machine_name)?
        .ok_or_else(|| {
            anyhow!("machine '{machine_name}' does not target a hosted control plane")
        })?;
    let summary = effective_config
        .hosted_machine_summary_contract(machine_name)?
        .ok_or_else(|| anyhow!("machine '{machine_name}' does not resolve to hosted inventory"))?;
    let inventory = effective_config.hosted_inventory_contract()?;
    let group = inventory.host_groups.get(host_group).ok_or_else(|| {
        anyhow!(
            "host group '{}' is not declared for hosted service placement on machine '{}'",
            host_group,
            machine_name
        )
    })?;
    if group.control_plane != hosted_identity.control_plane {
        bail!(
            "host group '{}' belongs to control plane '{}', not '{}'",
            host_group,
            group.control_plane,
            hosted_identity.control_plane
        );
    }

    let (node_name, _node) = inventory
        .nodes
        .iter()
        .find(|(_, node)| node.runtime_root == runtime_root)
        .ok_or_else(|| {
            anyhow!(
                "runtime root '{}' does not map to a hosted node for machine '{}'",
                runtime_root.display(),
                machine_name
            )
        })?;
    if !group.nodes.iter().any(|candidate| candidate == node_name) {
        bail!(
            "host group '{}' does not include selected node '{}' for machine '{}'",
            host_group,
            node_name,
            machine_name
        );
    }
    if let Some(reason) = summary.rejected_nodes.get(node_name) {
        bail!(
            "host group '{}' cannot place machine '{}' on node '{}': {}",
            host_group,
            machine_name,
            node_name,
            reason
        );
    }
    if !summary
        .candidate_nodes
        .iter()
        .any(|candidate| candidate == node_name)
    {
        bail!(
            "host group '{}' cannot place machine '{}' on node '{}'. {}",
            host_group,
            machine_name,
            node_name,
            summary.placement_detail
        );
    }

    let mut status = firecracker_local_machine_status(runtime_root, machine_name)?;
    status.control = control;
    status.detail = format!(
        "{} Routed through control plane '{}' and node '{}'.",
        status.detail, hosted_identity.control_plane, node_name
    );

    Ok(ResolvedMachineRuntime {
        status,
        control_plane: Some(hosted_identity.control_plane),
        node_name: Some(node_name.clone()),
        host_groups: summary.host_groups,
        host_group_policies: summary.host_group_policies,
        target_host_group: Some(host_group.to_string()),
        scheduler: Some(group.scheduler),
    })
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

pub fn prepare_hosted_pvm_node(
    config: &PortConfig,
    request: HostedPvmNodePrepareRequest,
) -> Result<HostedImportedNodeRecord> {
    let client = hosted_client_for_control_plane(config, &request.control_plane)?;
    let response: HostedSuccess<HostedImportedNodeRecord> = client
        .execute_json(
            client
                .inventory()
                .prepare_pvm_node(HostedPreparePvmNodeRequest {
                    control_plane: request.control_plane.clone(),
                    node_name: request.node_name.clone(),
                    architecture: request.architecture,
                    provenance: request.provenance,
                    package: request.package,
                }),
        )
        .map_err(|error| {
            anyhow!(
                "failed to prepare hosted pvm node '{}' through control plane '{}': {error}",
                request.node_name,
                request.control_plane
            )
        })?;
    Ok(response.result)
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
                    HostConnection::Ssh { .. } => None,
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
        let client = match hosted_client_for_control_plane(config, &control_plane_name) {
            Ok(client) => client,
            Err(error) if is_missing_hosted_auth_token(&error) => continue,
            Err(error) => return Err(error),
        };
        let response: HostedSuccess<Vec<MachineStatus>> = client
            .execute_json_with_timeout(client.machines().list(), HOSTED_MACHINE_LIST_TIMEOUT)
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
    let effective_config = effective_config_with_hosted_imported_inventory(config)?;
    let machine = effective_config
        .machines
        .get(request.machine_name)
        .with_context(|| format!("unknown machine '{}'", request.machine_name))?;
    effective_config
        .hosts
        .get(&machine.host)
        .with_context(|| format!("unknown host '{}'", machine.host))?;
    effective_config
        .hosted_api_identity_contract(request.machine_name)?
        .ok_or_else(|| {
            anyhow!(
                "machine '{}' does not target a hosted control plane",
                request.machine_name
            )
        })?;
    if let Some(summary) = effective_config.hosted_machine_summary_contract(request.machine_name)? {
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
    persist_local_hosted_machine_placement_from_route(
        config,
        request.machine_name,
        &response.route,
        response.result.launched_at_unix_s,
    )?;
    Ok(response.result)
}

fn hosted_control_plane_machine_status(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineStatus> {
    Ok(hosted_control_plane_machine_status_response(config, machine_name)?.result)
}

fn hosted_control_plane_machine_status_response(
    config: &PortConfig,
    machine_name: &str,
) -> Result<HostedSuccess<MachineStatus>> {
    let client = hosted_client_for_machine(config, machine_name)?;
    client
        .execute_json_with_timeout(
            client.machines().status(machine_name),
            HOSTED_MACHINE_STATUS_TIMEOUT,
        )
        .map_err(|error| {
            anyhow!(
                "failed to load machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })
}

fn runtime_root_from_machine_status(status: &MachineStatus) -> Option<PathBuf> {
    status.runtime_dir.parent().map(Path::to_path_buf)
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

fn hosted_control_plane_machine_wedge(
    config: &PortConfig,
    machine_name: &str,
) -> Result<MachineWedgeStatus> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<MachineWedgeStatus> = client
        .execute_json(client.machines().wedge(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to inspect wedge state for machine '{}' through the live hosted control-plane route: {error}",
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

fn hosted_service_bindings(bindings: &[ServiceSecretBinding]) -> Vec<ServiceSecretBinding> {
    bindings
        .iter()
        .map(|binding| ServiceSecretBinding {
            env: binding.env.clone(),
            secret: binding.secret.clone(),
        })
        .collect()
}

fn hosted_control_plane_put_machine_secret(
    config: &PortConfig,
    request: SecretPutRequest<'_>,
) -> Result<MachineSecretSummary> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<MachineSecretSummary> = client
        .execute_json(
            client
                .services()
                .secret_put(
                    request.machine_name,
                    HostedSecretPutRequest {
                        name: request.name.to_string(),
                        value: request.value.to_string(),
                    },
                )
                .context("failed to encode hosted service secret put request")?,
        )
        .map_err(|error| {
            anyhow!(
                "failed to store secret '{}' for machine '{}' through the live hosted control-plane route: {error}",
                request.name,
                request.machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_list_machine_secrets(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Vec<MachineSecretSummary>> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<Vec<MachineSecretSummary>> = client
        .execute_json(client.services().secret_list(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to list service secrets for machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_delete_machine_secret(
    config: &PortConfig,
    machine_name: &str,
    secret_name: &str,
) -> Result<MachineSecretSummary> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<MachineSecretSummary> = client
        .execute_json(client.services().secret_remove(machine_name, secret_name))
        .map_err(|error| {
            anyhow!(
                "failed to remove secret '{}' for machine '{}' through the live hosted control-plane route: {error}",
                secret_name,
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_apply_machine_service(
    config: &PortConfig,
    request: ServiceApplyRequest<'_>,
) -> Result<ServiceDefinitionStatus> {
    hosted_control_plane_apply_machine_service_with_timeout(config, request, HOSTED_HTTP_TIMEOUT)
}

fn hosted_control_plane_apply_machine_service_with_timeout(
    config: &PortConfig,
    request: ServiceApplyRequest<'_>,
    timeout: Duration,
) -> Result<ServiceDefinitionStatus> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<ServiceDefinitionStatus> = client
        .execute_json_with_timeout(
            client
                .services()
                .apply(
                    request.machine_name,
                    HostedServiceApplyRequest {
                        name: request.name.to_string(),
                        kind: request.kind,
                        host_group: request.host_group.map(str::to_string),
                        command: request.command.clone(),
                        secret_bindings: hosted_service_bindings(&request.secret_bindings),
                        policy: request.policy.clone(),
                    },
                )
                .context("failed to encode hosted service apply request")?,
            timeout,
        )
        .map_err(|error| {
            anyhow!(
                "failed to apply service '{}' for machine '{}' through the live hosted control-plane route: {error}",
                request.name,
                request.machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_list_machine_services(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Vec<ServiceDefinitionStatus>> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<Vec<ServiceDefinitionStatus>> = client
        .execute_json(client.services().list(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to list services for machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_machine_service_status(
    config: &PortConfig,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    hosted_control_plane_machine_service_status_with_timeout(
        config,
        machine_name,
        service_name,
        HOSTED_HTTP_TIMEOUT,
    )
}

fn hosted_control_plane_machine_service_status_with_timeout(
    config: &PortConfig,
    machine_name: &str,
    service_name: &str,
    timeout: Duration,
) -> Result<ServiceDefinitionStatus> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<ServiceDefinitionStatus> = client
        .execute_json_with_timeout(client.services().status(machine_name, service_name), timeout)
        .map_err(|error| {
            anyhow!(
                "failed to load service '{}' for machine '{}' through the live hosted control-plane route: {error}",
                service_name,
                machine_name
            )
        })?;
    Ok(response.result)
}

fn hosted_control_plane_stop_machine_service(
    config: &PortConfig,
    machine_name: &str,
    service_name: &str,
) -> Result<ServiceDefinitionStatus> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<ServiceDefinitionStatus> = client
        .execute_json(client.services().stop(machine_name, service_name))
        .map_err(|error| {
            anyhow!(
                "failed to stop service '{}' for machine '{}' through the live hosted control-plane route: {error}",
                service_name,
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

    resolve_routed_guest_endpoint(config, &routed_request).with_context(|| {
        format!(
            "control plane '{}' authorized guest attach for machine '{}' and routed it to node '{}'. {}",
            resolution.control_plane, request.machine_name, node_name, attach_detail
        )
    })
}

fn resolve_routed_guest_endpoint(
    config: &PortConfig,
    request: &GuestRequest<'_>,
) -> Result<GuestEndpoint> {
    let machine = config
        .machines
        .get(request.machine_name)
        .with_context(|| format!("unknown machine '{}'", request.machine_name))?;
    match machine.substrate {
        ExecutionSubstrate::Firecracker => resolve_firecracker_guest_endpoint(config, request),
        ExecutionSubstrate::CloudHypervisor => {
            resolve_cloud_hypervisor_guest_endpoint(config, request)
        }
        ExecutionSubstrate::Avf => resolve_avf_guest_endpoint(config, request),
    }
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
            HostConnection::Ssh {
                destination,
                user,
                port,
            } => {
                let control = MachineControlContract::ssh_managed_remote();
                format!(
                    "{detail} SSH-managed routing is modeled for host '{}' through {}@{}:{} with route '{}' and inventory or lifecycle owners '{}' and '{}'.",
                    host_name,
                    user,
                    destination,
                    port,
                    control.launch_route,
                    control.inventory_owner,
                    control.lifecycle_owner
                )
            }
        },
    })
}

fn ssh_connection_checks(
    host_name: &str,
    provider: HostProvider,
    connection: &HostConnection,
) -> Vec<DoctorCheck> {
    let HostConnection::Ssh {
        destination,
        user,
        port,
    } = connection
    else {
        return Vec::new();
    };

    let control = MachineControlContract::ssh_managed_remote();
    let provider = host_provider_label(provider);

    vec![
        DoctorCheck {
            name: format!("host:{host_name}:ssh-auth"),
            ok: true,
            required: false,
            detail: format!(
                "SSH-managed route '{}' targets host '{}' (provider '{}') via {}@{}:{} with lifecycle owner '{}'. Supply SSH auth material through the operator SSH environment; this lane does not use hosted control-plane bearer tokens.",
                control.launch_route,
                host_name,
                provider,
                user,
                destination,
                port,
                control.lifecycle_owner
            ),
        },
        DoctorCheck {
            name: format!("host:{host_name}:ssh-bootstrap"),
            ok: false,
            required: false,
            detail: format!(
                "SSH-managed route '{}' expects inventory owner '{}' and lifecycle owner '{}' on host '{}' (provider '{}'). Remote bootstrap must install the Linux execution prerequisites and Port runtime on the remote host; the first SSH lifecycle slice now covers launch, status, and stop, but Port still will not fall back to local runtime or hosted control-plane ownership for the remaining SSH workflows.",
                control.launch_route,
                control.inventory_owner,
                control.lifecycle_owner,
                host_name,
                provider
            ),
        },
    ]
}

fn host_provider_label(provider: HostProvider) -> &'static str {
    match provider {
        HostProvider::Local => "local",
        HostProvider::GenericLinux => "generic-linux",
        HostProvider::Aws => "aws",
        HostProvider::Gcp => "gcp",
        HostProvider::Azure => "azure",
    }
}

fn volume_backend_label(backend: MachineVolumeBackend) -> &'static str {
    match backend {
        MachineVolumeBackend::HostFile => "host-file",
    }
}

fn volume_persistence_label(persistence: MachineVolumePersistence) -> &'static str {
    match persistence {
        MachineVolumePersistence::Persistent => "persistent",
    }
}

fn attached_volume_preflight_checks(
    machine_name: &str,
    volumes: &[MachineVolumeSpec],
    control: &MachineControlContract,
) -> Vec<DoctorCheck> {
    volumes
        .iter()
        .map(|volume| {
            let exists = volume.path.exists();
            let is_file = exists
                && fs::metadata(&volume.path)
                    .map(|metadata| metadata.is_file())
                    .unwrap_or(false);
            let backend = volume_backend_label(volume.backend);
            let persistence = volume_persistence_label(volume.persistence);

            DoctorCheck {
                name: format!("machine:{machine_name}:volume:{}:host-path", volume.name),
                ok: exists && is_file,
                required: true,
                detail: if exists && is_file {
                    format!(
                        "machine '{machine_name}' volume '{}' backend '{}' persistence '{}' host path '{}' is ready for route '{}' with inventory owner '{}' and lifecycle owner '{}'.",
                        volume.name,
                        backend,
                        persistence,
                        volume.path.display(),
                        control.launch_route,
                        control.inventory_owner,
                        control.lifecycle_owner
                    )
                } else if exists {
                    format!(
                        "machine '{machine_name}' volume '{}' backend '{}' persistence '{}' host path '{}' exists but is not a regular file; route '{}' requires a regular host file with inventory owner '{}' and lifecycle owner '{}'.",
                        volume.name,
                        backend,
                        persistence,
                        volume.path.display(),
                        control.launch_route,
                        control.inventory_owner,
                        control.lifecycle_owner
                    )
                } else {
                    format!(
                        "machine '{machine_name}' volume '{}' backend '{}' persistence '{}' host path '{}' is missing; route '{}' requires a regular host file with inventory owner '{}' and lifecycle owner '{}'.",
                        volume.name,
                        backend,
                        persistence,
                        volume.path.display(),
                        control.launch_route,
                        control.inventory_owner,
                        control.lifecycle_owner
                    )
                },
            }
        })
        .collect()
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
            if !matches!(host.connection, HostConnection::Local) {
                issues.push(String::from(
                    "AVF local runtime currently requires a local host connection.",
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

fn attached_volume_lane_supported(
    host: &port_model::HostSpec,
    machine: &port_model::MachineSpec,
) -> bool {
    matches!(host.connection, HostConnection::Local)
        && host.platform == HostPlatform::Linux
        && machine.substrate == ExecutionSubstrate::Firecracker
        && machine.protection_mode == ProtectionMode::Standard
}

fn attached_volume_doctor_checks(
    machine_name: &str,
    host_name: &str,
    host: &port_model::HostSpec,
    machine: &port_model::MachineSpec,
) -> Vec<DoctorCheck> {
    let control = MachineControlContract::for_connection(&host.connection);
    let route = control.launch_route;
    let inventory_owner = control.inventory_owner;
    let lifecycle_owner = control.lifecycle_owner;
    let supported_lane = attached_volume_lane_supported(host, machine);

    machine
        .volumes
        .iter()
        .map(|volume| {
            let backend = volume_backend_label(volume.backend);
            let persistence = volume_persistence_label(volume.persistence);

            let (ok, detail) = if supported_lane {
                let exists = volume.path.exists();
                let is_file = exists
                    && fs::metadata(&volume.path)
                        .map(|metadata| metadata.is_file())
                        .unwrap_or(false);
                if exists && is_file {
                    (
                        true,
                        format!(
                            "machine '{machine_name}' attached volume '{}' backend '{}' persistence '{}' host path '{}' is ready for launch route '{}' with inventory owner '{}' and lifecycle owner '{}'.",
                            volume.name,
                            backend,
                            persistence,
                            volume.path.display(),
                            route,
                            inventory_owner,
                            lifecycle_owner
                        ),
                    )
                } else if exists {
                    (
                        false,
                        format!(
                            "machine '{machine_name}' attached volume '{}' backend '{}' persistence '{}' host path '{}' exists but is not a regular file; launch route '{}' requires a regular host file with inventory owner '{}' and lifecycle owner '{}'.",
                            volume.name,
                            backend,
                            persistence,
                            volume.path.display(),
                            route,
                            inventory_owner,
                            lifecycle_owner
                        ),
                    )
                } else {
                    (
                        false,
                        format!(
                            "machine '{machine_name}' attached volume '{}' backend '{}' persistence '{}' host path '{}' is missing; launch route '{}' requires a regular host file with inventory owner '{}' and lifecycle owner '{}'.",
                            volume.name,
                            backend,
                            persistence,
                            volume.path.display(),
                            route,
                            inventory_owner,
                            lifecycle_owner
                        ),
                    )
                }
            } else {
                (
                    false,
                    format!(
                        "machine '{machine_name}' attached volume '{}' backend '{}' persistence '{}' host path '{}' targets host '{}' through launch route '{}' with inventory owner '{}' and lifecycle owner '{}', but attached volumes are only supported on the local Firecracker standard lane in this slice.",
                        volume.name,
                        backend,
                        persistence,
                        volume.path.display(),
                        host_name,
                        route,
                        inventory_owner,
                        lifecycle_owner
                    ),
                )
            };

            DoctorCheck {
                name: format!("machine:{machine_name}:volume:{}:attached-volume", volume.name),
                ok,
                required: false,
                detail,
            }
        })
        .collect()
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

fn observed_host_architecture(facts: &DoctorHostFacts) -> Option<MachineArchitecture> {
    match facts.host_architecture.as_str() {
        "x86_64" | "amd64" => Some(MachineArchitecture::X86_64),
        "aarch64" | "arm64" => Some(MachineArchitecture::Aarch64),
        _ => None,
    }
}

fn avf_machine_checks(
    machine_name: &str,
    host: &port_model::HostSpec,
    machine: &port_model::MachineSpec,
    facts: &DoctorHostFacts,
) -> Vec<DoctorCheck> {
    if machine.substrate != ExecutionSubstrate::Avf {
        return Vec::new();
    }

    let contract = AvfExecutionContract::linux_guest();
    let platform_ok = host.platform == contract.host_platform
        && matches!(host.connection, HostConnection::Local)
        && facts.host_os == "macos";
    let supported_architectures = contract
        .supported_host_architectures
        .iter()
        .map(|architecture| architecture_dir(*architecture))
        .collect::<Vec<_>>();
    let architecture_ok = observed_host_architecture(facts)
        .map(|architecture| {
            contract
                .supported_host_architectures
                .contains(&architecture)
        })
        .unwrap_or(false);
    let availability_ok = platform_ok;
    let helper_boundary =
        "Set PORT_AVF_LAUNCHER to an external launcher helper for local AVF workflows.";
    let packaging_boundary = "Port does not ship a bundled macOS-only launcher workflow or a silent fallback to another substrate in this slice.";

    vec![
        DoctorCheck {
            name: format!("avf:{machine_name}:host-platform"),
            ok: platform_ok,
            required: false,
            detail: if platform_ok {
                String::from("Host OS is macOS and matches the local AVF lane requirement.")
            } else if !matches!(host.connection, HostConnection::Local) {
                String::from(
                    "AVF machine targets a non-local host connection. AVF local runtime currently requires a local host connection on macOS.",
                )
            } else {
                format!(
                    "AVF machine requires a macOS local host. Detected host OS '{}'; Firecracker and Firecracker/PVM remain separate Linux lanes.",
                    facts.host_os
                )
            },
        },
        DoctorCheck {
            name: format!("avf:{machine_name}:host-architecture"),
            ok: architecture_ok,
            required: false,
            detail: if architecture_ok {
                format!(
                    "Detected host architecture '{}' is supported by the AVF lane (supported: {}).",
                    facts.host_architecture,
                    supported_architectures.join(", ")
                )
            } else {
                format!(
                    "Detected host architecture '{}' is not in the AVF support set (supported: {}).",
                    facts.host_architecture,
                    supported_architectures.join(", ")
                )
            },
        },
        DoctorCheck {
            name: format!("avf:{machine_name}:runtime-availability"),
            ok: availability_ok,
            required: false,
            detail: if availability_ok {
                format!(
                    "AVF runtime can target Apple's Virtualization framework on this host. {helper_boundary} {} {packaging_boundary}",
                    contract.operator_prerequisites[1]
                )
            } else {
                format!(
                    "AVF runtime availability is bounded to local macOS hosts with Apple's Virtualization framework. {helper_boundary} {} {packaging_boundary}",
                    contract.operator_prerequisites[1]
                )
            },
        },
    ]
}

fn cloud_hypervisor_machine_checks(
    machine_name: &str,
    host: &port_model::HostSpec,
    machine: &port_model::MachineSpec,
    facts: &DoctorHostFacts,
) -> Vec<DoctorCheck> {
    if machine.substrate != ExecutionSubstrate::CloudHypervisor {
        return Vec::new();
    }

    let supported_architectures = [MachineArchitecture::X86_64, MachineArchitecture::Aarch64];
    let supported_labels = supported_architectures
        .iter()
        .map(|architecture| architecture_dir(*architecture))
        .collect::<Vec<_>>();
    let platform_ok = host.platform == HostPlatform::Linux
        && matches!(host.connection, HostConnection::Local)
        && facts.host_os == "linux";
    let architecture_ok = observed_host_architecture(facts)
        .map(|architecture| supported_architectures.contains(&architecture))
        .unwrap_or(false);
    let protection_mode_ok = machine.protection_mode == ProtectionMode::Standard;
    let binary = if platform_ok {
        binary_check(
            &format!("cloud-hypervisor:{machine_name}:binary"),
            "cloud-hypervisor",
            false,
        )
    } else {
        DoctorCheck {
            name: format!("cloud-hypervisor:{machine_name}:binary"),
            ok: false,
            required: false,
            detail: String::from(
                "Cloud Hypervisor binary readiness is only meaningful on a local Linux host; Port will not fall back to Firecracker for this machine.",
            ),
        }
    };

    vec![
        DoctorCheck {
            name: format!("cloud-hypervisor:{machine_name}:host-platform"),
            ok: platform_ok,
            required: false,
            detail: if platform_ok {
                String::from(
                    "Host OS is Linux and matches the local Cloud Hypervisor lane requirement.",
                )
            } else if !matches!(host.connection, HostConnection::Local) {
                String::from(
                    "Cloud Hypervisor machine targets a non-local host connection. This lane currently expects a local Linux host, and Port will not fall back to Firecracker.",
                )
            } else {
                format!(
                    "Cloud Hypervisor machine requires a local Linux host. Detected host OS '{}'; Port will not fall back to Firecracker for this machine.",
                    facts.host_os
                )
            },
        },
        DoctorCheck {
            name: format!("cloud-hypervisor:{machine_name}:host-architecture"),
            ok: architecture_ok,
            required: false,
            detail: if architecture_ok {
                format!(
                    "Detected host architecture '{}' is supported by the Cloud Hypervisor lane (supported: {}).",
                    facts.host_architecture,
                    supported_labels.join(", ")
                )
            } else {
                format!(
                    "Detected host architecture '{}' is not in the Cloud Hypervisor support set (supported: {}).",
                    facts.host_architecture,
                    supported_labels.join(", ")
                )
            },
        },
        DoctorCheck {
            name: format!("cloud-hypervisor:{machine_name}:protection-mode"),
            ok: protection_mode_ok,
            required: false,
            detail: if protection_mode_ok {
                String::from(
                    "Cloud Hypervisor currently defines the standard protection lane only.",
                )
            } else {
                String::from(
                    "Port does not currently define a Cloud Hypervisor PVM lane; the selected machine must stay on protection_mode = \"standard\".",
                )
            },
        },
        binary,
    ]
}

fn hosted_pvm_lane_checks(config: &PortConfig) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    for (node_name, node) in &config.nodes {
        let hosted_control_plane = hosted_control_plane_for_node(config, node_name);
        let imported_record = if let Some(control_plane) = hosted_control_plane {
            match read_hosted_imported_inventory_state(config, control_plane) {
                Ok(Some(state)) => state.nodes.get(node_name).cloned(),
                Ok(None) => None,
                Err(error) => {
                    for lane in &node.capabilities.pvm_lanes {
                        if lane.state == PvmCapabilityState::ResearchOnly {
                            continue;
                        }
                        checks.push(DoctorCheck {
                            name: format!(
                                "pvm:{node_name}:{}:host-kit-contract",
                                architecture_dir(lane.architecture)
                            ),
                            ok: false,
                            required: false,
                            detail: format!(
                                "Hosted control plane '{}' could not load imported inventory state for node '{}': {error}",
                                control_plane, node_name
                            ),
                        });
                    }
                    continue;
                }
            }
        } else {
            None
        };

        for lane in &node.capabilities.pvm_lanes {
            if lane.state == PvmCapabilityState::ResearchOnly {
                continue;
            }

            if let Some(imported) = imported_record.as_ref() {
                if let Some(imported_lane) =
                    imported.capability_summary.pvm_lane_for(lane.architecture)
                {
                    checks.push(hosted_pvm_lane_check_from_imported_record(
                        node_name,
                        hosted_control_plane,
                        imported_lane,
                        imported,
                    ));
                    continue;
                }
            }
            checks.push(hosted_pvm_lane_check_without_imported_record(
                config,
                node_name,
                hosted_control_plane,
                lane,
            ));
        }
    }

    checks
}

fn pvm_host_kit_contract_issue(
    expected_architecture: MachineArchitecture,
    host_kit: &PvmHostKit,
) -> Option<String> {
    if host_kit.package.name.trim().is_empty() {
        return Some(String::from(
            "host-kit package must declare a non-empty package name.",
        ));
    }
    if host_kit.package.version.trim().is_empty() {
        return Some(String::from(
            "host-kit package must declare a non-empty package version.",
        ));
    }
    if host_kit.package.host_kernel_release.trim().is_empty() {
        return Some(String::from(
            "host-kit package must declare a non-empty host-kernel release.",
        ));
    }
    if host_kit.package.firecracker_build.trim().is_empty() {
        return Some(String::from(
            "host-kit package must declare a non-empty Firecracker build.",
        ));
    }
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
        "host-kit package {}@{} requires host kernel {}, Firecracker build {}, Linux/{}, boot args [{}], and the patched Firecracker binary {}.",
        host_kit.package.name,
        host_kit.package.version,
        host_kit.package.host_kernel_release,
        host_kit.package.firecracker_build,
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
    connection: &HostConnection,
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

    let detail = match connection {
        HostConnection::Local => format!(
            "machine '{machine_name}' targets host '{host_name}' through a remote connection, but provider 'local' is reserved for direct local Linux launch"
        ),
        HostConnection::HostedControlPlane { .. } => match provider {
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
        },
        HostConnection::Ssh {
            destination,
            user,
            port,
        } => match provider {
            HostProvider::Local => format!(
                "machine '{machine_name}' targets ssh-managed host '{host_name}' through {}@{}:{} but provider 'local' is reserved for direct local Linux launch",
                user, destination, port
            ),
            HostProvider::GenericLinux => format!(
                "machine '{machine_name}' targets ssh-managed Linux host '{host_name}' through {}@{}:{}; the SSH ownership lane is now modeled but lifecycle execution is not implemented yet.",
                user, destination, port
            ),
            HostProvider::Aws => format!(
                "machine '{machine_name}' targets ssh-managed AWS host '{host_name}' through {}@{}:{}; AWS remains a justified remote lane, but SSH-managed lifecycle execution is not implemented yet.",
                user, destination, port
            ),
            HostProvider::Gcp => format!(
                "machine '{machine_name}' targets ssh-managed GCP host '{host_name}' through {}@{}:{}; GCP remains a justified remote lane, but SSH-managed lifecycle execution is not implemented yet.",
                user, destination, port
            ),
            HostProvider::Azure => format!(
                "machine '{machine_name}' targets ssh-managed Azure host '{host_name}' through {}@{}:{}; Azure remains unsupported for the Firecracker MVP.",
                user, destination, port
            ),
        },
    };

    format!("{detail}{hosted_route}")
}

#[derive(Debug, Clone)]
struct SshMachineTarget {
    machine_name: String,
    host_name: String,
    provider: HostProvider,
    destination: String,
    user: String,
    port: u16,
    control: MachineControlContract,
}

impl SshMachineTarget {
    fn ssh_endpoint(&self) -> String {
        format!("{}@{}:{}", self.user, self.destination, self.port)
    }

    fn route_context(&self, route: port_model::MachineCommandRoute) -> String {
        format!(
            "machine '{}' targets ssh-managed host '{}' (provider '{}') through {} with route '{}' and inventory or lifecycle owners '{}' and '{}'",
            self.machine_name,
            self.host_name,
            host_provider_label(self.provider),
            self.ssh_endpoint(),
            route,
            self.control.inventory_owner,
            self.control.lifecycle_owner
        )
    }
}

fn ssh_machine_target(config: &PortConfig, machine_name: &str) -> Result<SshMachineTarget> {
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
        bail!(
            "machine '{}' does not target an ssh-managed host",
            machine_name
        );
    };

    let target = SshMachineTarget {
        machine_name: machine_name.to_string(),
        host_name: machine.host.clone(),
        provider: host.provider,
        destination: destination.clone(),
        user: user.clone(),
        port: *port,
        control: MachineControlContract::ssh_managed_remote(),
    };

    if host.platform != HostPlatform::Linux {
        bail!(
            "{}; the first SSH lifecycle slice only supports remote Linux hosts",
            target.route_context(target.control.launch_route)
        );
    }

    match host.provider {
        HostProvider::Local => bail!(
            "{}; provider 'local' is reserved for direct local Linux launch",
            target.route_context(target.control.launch_route)
        ),
        HostProvider::Azure => bail!(
            "{}; Azure remains unsupported for the Firecracker MVP",
            target.route_context(target.control.launch_route)
        ),
        HostProvider::GenericLinux | HostProvider::Aws | HostProvider::Gcp => {}
    }

    Ok(target)
}

fn ssh_machine_config_for_remote_execution(
    config: &PortConfig,
    machine_name: &str,
) -> Result<PortConfig> {
    let mut remote = config.clone();
    let host_name = remote
        .machines
        .get(machine_name)
        .with_context(|| format!("unknown machine '{machine_name}'"))?
        .host
        .clone();
    let host = remote
        .hosts
        .get_mut(&host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;
    if !matches!(host.connection, HostConnection::Ssh { .. }) {
        bail!(
            "machine '{}' does not target an ssh-managed host",
            machine_name
        );
    }
    host.connection = HostConnection::Local;
    Ok(remote)
}

fn annotate_ssh_status(target: &SshMachineTarget, mut status: MachineStatus) -> MachineStatus {
    status.control = target.control.clone();
    status.detail = format!(
        "{}; {}",
        status.detail,
        target.route_context(target.control.status_route)
    );
    status
}

fn annotate_ssh_stop_result(target: &SshMachineTarget, mut result: StopResult) -> StopResult {
    result.control = target.control.clone();
    result.detail = format!(
        "{}; {}",
        result.detail,
        target.route_context(target.control.stop_route)
    );
    result
}

fn ssh_command_binary() -> Result<PathBuf> {
    find_binary("ssh").context(
        "ssh binary was not found on PATH; install OpenSSH client support to use ssh-managed remote lifecycle routing",
    )
}

fn run_ssh_lifecycle_command<T: DeserializeOwned>(
    config: &PortConfig,
    target: &SshMachineTarget,
    command_name: &str,
    args: &[String],
) -> Result<T> {
    let ssh_binary = ssh_command_binary()?;
    let mut command = Command::new(&ssh_binary);
    command
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-p")
        .arg(target.port.to_string())
        .arg(format!("{}@{}", target.user, target.destination));
    for arg in args {
        command.arg(arg);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to start ssh-managed lifecycle command '{}' for {} via '{}'",
            command_name,
            target.route_context(target.control.launch_route),
            ssh_binary.display()
        )
    })?;
    let encoded = serde_json::to_vec(config)
        .context("failed to serialize Port config for remote ssh command")?;
    child
        .stdin
        .take()
        .context("ssh lifecycle command did not expose a stdin pipe")?
        .write_all(&encoded)
        .with_context(|| {
            format!(
                "failed to stream Port config to remote ssh command '{}' for {}",
                command_name,
                target.route_context(target.control.launch_route)
            )
        })?;
    let output = child.wait_with_output().with_context(|| {
        format!(
            "failed to wait for remote ssh command '{}' for {}",
            command_name,
            target.route_context(target.control.launch_route)
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            format!("remote ssh command exited with status {}", output.status)
        } else {
            stderr
        };
        bail!(
            "ssh-managed lifecycle command '{}' failed for {}: {}",
            command_name,
            target.route_context(target.control.launch_route),
            detail
        );
    }

    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "failed to decode remote ssh command '{}' response for {}",
            command_name,
            target.route_context(target.control.launch_route)
        )
    })
}

pub fn ssh_internal_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    let _ = ssh_machine_target(config, request.machine_name)?;
    let remote = ssh_machine_config_for_remote_execution(config, request.machine_name)?;
    launch_local_machine(&remote, request)
}

pub fn ssh_internal_machine_status(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineStatus> {
    let target = ssh_machine_target(config, machine_name)?;
    let remote = ssh_machine_config_for_remote_execution(config, machine_name)?;
    let status = machine_status(&remote, runtime_root, machine_name)?;
    Ok(annotate_ssh_status(&target, status))
}

pub fn ssh_internal_stop_machine(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let target = ssh_machine_target(config, machine_name)?;
    let remote = ssh_machine_config_for_remote_execution(config, machine_name)?;
    let result = stop_machine(&remote, runtime_root, machine_name, timeout)?;
    Ok(annotate_ssh_stop_result(&target, result))
}

fn ssh_managed_launch_machine(
    config: &PortConfig,
    request: &LaunchRequest<'_>,
) -> Result<LaunchMetadata> {
    let target = ssh_machine_target(config, request.machine_name)?;
    run_ssh_lifecycle_command(
        config,
        &target,
        "launch",
        &[
            String::from("port"),
            String::from("internal"),
            String::from("ssh-machine-launch"),
            String::from("--machine"),
            request.machine_name.to_string(),
            String::from("--runtime-root"),
            request.runtime_root.display().to_string(),
            String::from("--boot-wait-secs"),
            request.boot_wait.as_secs().to_string(),
        ],
    )
}

fn ssh_managed_machine_status(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
) -> Result<MachineStatus> {
    let target = ssh_machine_target(config, machine_name)?;
    run_ssh_lifecycle_command(
        config,
        &target,
        "status",
        &[
            String::from("port"),
            String::from("internal"),
            String::from("ssh-machine-status"),
            String::from("--machine"),
            machine_name.to_string(),
            String::from("--runtime-root"),
            runtime_root.display().to_string(),
        ],
    )
}

fn ssh_managed_stop_machine(
    config: &PortConfig,
    runtime_root: &Path,
    machine_name: &str,
    timeout: Duration,
) -> Result<StopResult> {
    let target = ssh_machine_target(config, machine_name)?;
    run_ssh_lifecycle_command(
        config,
        &target,
        "stop",
        &[
            String::from("port"),
            String::from("internal"),
            String::from("ssh-machine-stop"),
            String::from("--machine"),
            machine_name.to_string(),
            String::from("--runtime-root"),
            runtime_root.display().to_string(),
            String::from("--wait-secs"),
            timeout.as_secs().to_string(),
        ],
    )
}

fn launch_preflight_checks(
    machine: &port_model::MachineSpec,
    kernel_path: &Path,
    guest_image_path: &Path,
) -> Vec<DoctorCheck> {
    let iptables_binary = iptables_binary();
    let mut checks = vec![
        versioned_binary_check("iproute2", "ip", &["-V"], "iproute2", true),
        versioned_binary_check(
            "iptables",
            &iptables_binary,
            &["--version"],
            "iptables",
            true,
        ),
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

    if launch_requires_kvm_device(machine) {
        checks.insert(
            0,
            path_check(
                "kvm-device",
                Path::new("/dev/kvm"),
                true,
                "Found /dev/kvm for KVM acceleration.",
                "Missing /dev/kvm.",
            ),
        );
    }

    if machine.substrate == ExecutionSubstrate::Firecracker
        && machine.protection_mode == ProtectionMode::Standard
    {
        checks.push(binary_check("firecracker-binary", "firecracker", true));
    }
    if machine.rootfs_overlay.is_some() {
        checks.push(binary_check("mkfs-ext4", "mkfs.ext4", true));
        let initrd_path = firecracker_initrd_path_for_rootfs(guest_image_path)
            .unwrap_or_else(|| guest_image_path.with_file_name("initrd.cpio.gz"));
        checks.push(path_check(
            "rootfs-overlay-initrd",
            &initrd_path,
            true,
            &format!(
                "Found sibling initrd '{}' required for rootfs overlay boot.",
                initrd_path.display()
            ),
            &format!(
                "Missing sibling initrd '{}' required for rootfs overlay boot.",
                initrd_path.display()
            ),
        ));
    }

    checks
}

fn launch_requires_kvm_device(machine: &port_model::MachineSpec) -> bool {
    match machine.substrate {
        ExecutionSubstrate::Firecracker => machine.protection_mode == ProtectionMode::Standard,
        ExecutionSubstrate::CloudHypervisor => true,
        ExecutionSubstrate::Avf => false,
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

fn binary_candidates(binary: &str) -> Vec<PathBuf> {
    let candidate = Path::new(binary);
    if candidate.components().count() > 1 {
        return candidate
            .is_file()
            .then(|| candidate.to_path_buf())
            .into_iter()
            .collect();
    }

    let Some(path) = env::var_os("PATH") else {
        return Vec::new();
    };

    env::split_paths(&path)
        .map(|entry| entry.join(binary))
        .filter(|candidate| candidate.is_file())
        .collect()
}

fn binary_output_contains(path: &Path, args: &[&str], needle: &str) -> bool {
    Command::new(path)
        .args(args)
        .output()
        .map(|output| {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            combined.contains(needle)
        })
        .unwrap_or(false)
}

fn find_versioned_binary(binary: &str, args: &[&str], needle: &str) -> Option<PathBuf> {
    binary_candidates(binary)
        .into_iter()
        .find(|candidate| binary_output_contains(candidate, args, needle))
}

fn versioned_binary_check(
    name: &str,
    binary: &str,
    args: &[&str],
    needle: &str,
    required: bool,
) -> DoctorCheck {
    match find_versioned_binary(binary, args, needle).or_else(|| find_binary(binary)) {
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

fn oci_registry_dependency_check(
    artifact_name: &str,
    direction: &str,
    transport: OciRegistryTransport,
) -> DoctorCheck {
    let mut check = binary_check(
        &format!("artifact-store:{artifact_name}:{direction}:oras"),
        "oras",
        true,
    );
    if check.ok {
        check.detail = format!(
            "{} OCI registry backend will use {} transport.",
            check.detail,
            transport.describe()
        );
    } else {
        check.detail = format!(
            "{} Required for artifact '{}' {} OCI registry transport over {}.",
            check.detail,
            artifact_name,
            direction,
            transport.describe()
        );
    }
    check
}

fn oci_registry_auth_check(
    artifact_name: &str,
    direction: &str,
    auth: &OciRegistryAuth,
) -> DoctorCheck {
    match auth {
        OciRegistryAuth::Anonymous => DoctorCheck {
            name: format!("artifact-store:{artifact_name}:{direction}:auth"),
            ok: true,
            required: true,
            detail: format!(
                "Artifact '{}' {} OCI registry backend uses anonymous auth.",
                artifact_name, direction
            ),
        },
        OciRegistryAuth::BasicEnv {
            username_variable,
            password_variable,
        } => {
            let username_present = env::var_os(username_variable).is_some();
            let password_present = env::var_os(password_variable).is_some();
            DoctorCheck {
                name: format!("artifact-store:{artifact_name}:{direction}:auth"),
                ok: username_present && password_present,
                required: true,
                detail: if username_present && password_present {
                    format!(
                        "Artifact '{}' {} OCI registry backend will source credentials from env:{} and env:{}.",
                        artifact_name, direction, username_variable, password_variable
                    )
                } else {
                    format!(
                        "Artifact '{}' {} OCI registry backend requires env:{} and env:{} for basic-env auth.",
                        artifact_name, direction, username_variable, password_variable
                    )
                },
            }
        }
    }
}

fn find_binary(binary: &str) -> Option<PathBuf> {
    binary_candidates(binary).into_iter().next()
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
    run_artifact_pipeline_with_io(config, request, action, ArtifactPipelineIo::Inherit)
}

fn run_artifact_pipeline_quiet(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
    action: ArtifactAction,
) -> Result<ArtifactMetadata> {
    run_artifact_pipeline_with_io(config, request, action, ArtifactPipelineIo::Capture)
}

fn run_artifact_pipeline_with_io(
    config: &PortConfig,
    request: ArtifactRequest<'_>,
    action: ArtifactAction,
    io: ArtifactPipelineIo,
) -> Result<ArtifactMetadata> {
    ensure_native_build_lane(request.architecture)?;
    let artifact = resolve_artifact_metadata(config, request)?;
    let kind = artifact.kind;
    let script = artifact_script(kind, action)?;

    let mut command = Command::new(&script);
    command.arg(&artifact.path).stdin(Stdio::null());
    if let Some(workdir) = artifact_pipeline_workdir(action)? {
        command.current_dir(workdir);
    }
    match io {
        ArtifactPipelineIo::Inherit => {
            let status = command
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| {
                    format!("failed to start artifact pipeline '{}'", script.display())
                })?;
            if !status.success() {
                bail!(
                    "artifact pipeline '{}' exited with status {status}",
                    script.display()
                );
            }
        }
        ArtifactPipelineIo::Capture => {
            let output = command.output().with_context(|| {
                format!("failed to start artifact pipeline '{}'", script.display())
            })?;
            if !output.status.success() {
                bail!(
                    "artifact pipeline '{}' exited with status {} (stdout: {}; stderr: {})",
                    script.display(),
                    output.status,
                    summarize_process_output(&output.stdout),
                    summarize_process_output(&output.stderr)
                );
            }
        }
    }

    Ok(artifact)
}

fn artifact_pipeline_workdir(action: ArtifactAction) -> Result<Option<PathBuf>> {
    match action {
        ArtifactAction::Build => Ok(None),
        ArtifactAction::Validate => Ok(None),
    }
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

fn artifact_inventory_record(
    name: &str,
    kind: ArtifactKind,
    spec: &port_model::ArtifactSpec,
) -> ArtifactInventoryRecord {
    let variants = spec
        .variants
        .iter()
        .map(|variant| {
            let cache_path = cache_path_for(spec, variant);
            let local_present = variant.path.exists();
            let cache_present = cache_path.exists();
            ArtifactVariantInventory {
                selector: variant.selector,
                path: variant.path.clone(),
                local_present,
                cache_path,
                cache_present,
                availability: artifact_availability_state(local_present, cache_present),
            }
        })
        .collect();

    ArtifactInventoryRecord {
        name: name.to_string(),
        kind,
        reference: spec.reference.clone(),
        build_command: spec.build.clone(),
        validate_command: spec.validate.clone(),
        variants,
    }
}

fn artifact_availability_state(
    local_present: bool,
    cache_present: bool,
) -> ArtifactAvailabilityState {
    match (local_present, cache_present) {
        (true, true) => ArtifactAvailabilityState::LocalAndCache,
        (true, false) => ArtifactAvailabilityState::Local,
        (false, true) => ArtifactAvailabilityState::CacheOnly,
        (false, false) => ArtifactAvailabilityState::Missing,
    }
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

fn resolve_artifact_store_contract(
    config: &PortConfig,
    store: &ArtifactStore,
    artifact: &ArtifactMetadata,
) -> Result<ArtifactStoreContract> {
    match store {
        ArtifactStore::FileSystem { root } => Ok(ArtifactStoreContract::FileSystem {
            store_path: root
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
                ),
        }),
        ArtifactStore::OciRegistry { transport, auth } => {
            resolve_oci_registry_contract(artifact, *transport, auth)
        }
        ArtifactStore::HostedApi { endpoint } => {
            let identity = config
                .hosted_artifact_identity_contract(endpoint)
                .map_err(|error| {
                    anyhow!("invalid hosted artifact backend '{}': {error}", endpoint)
                })?;
            let filename = artifact
                .path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("artifact"))
                .to_string_lossy()
                .into_owned();
            let store_path = port_model::hosted_artifact_store_path(
                &identity.control_plane,
                &artifact.reference,
                artifact.selector,
                Path::new(&filename),
            );
            Ok(ArtifactStoreContract::HostedApi {
                identity,
                transfer: HostedArtifactTransferRequest {
                    artifact_name: artifact.name.clone(),
                    reference: artifact.reference.clone(),
                    selector: artifact.selector,
                    filename,
                    store_path,
                },
            })
        }
    }
}

fn resolve_oci_registry_contract(
    artifact: &ArtifactMetadata,
    transport: OciRegistryTransport,
    auth: &OciRegistryAuth,
) -> Result<ArtifactStoreContract> {
    let remote_reference = artifact.reference.oci_remote_reference(artifact.selector);
    let oras_binary = find_binary("oras").with_context(|| {
        format!(
            "OCI registry backend for artifact '{}' requires 'oras' on PATH to reach '{}' over {} transport",
            artifact.name,
            remote_reference,
            transport.describe()
        )
    })?;
    validate_oci_registry_auth_env(auth, artifact, &remote_reference)?;
    Ok(ArtifactStoreContract::OciRegistry {
        oras_binary,
        remote_reference: remote_reference.clone(),
        store_path: PathBuf::from(&remote_reference),
        transport,
        auth: auth.clone(),
    })
}

fn validate_oci_registry_auth_env(
    auth: &OciRegistryAuth,
    artifact: &ArtifactMetadata,
    remote_reference: &str,
) -> Result<()> {
    match auth {
        OciRegistryAuth::Anonymous => Ok(()),
        OciRegistryAuth::BasicEnv {
            username_variable,
            password_variable,
        } => {
            if env::var_os(username_variable).is_none() {
                bail!(
                    "OCI registry backend for artifact '{}' requires env:{} before accessing '{}' with {} auth",
                    artifact.name,
                    username_variable,
                    remote_reference,
                    auth.describe()
                );
            }
            if env::var_os(password_variable).is_none() {
                bail!(
                    "OCI registry backend for artifact '{}' requires env:{} before accessing '{}' with {} auth",
                    artifact.name,
                    password_variable,
                    remote_reference,
                    auth.describe()
                );
            }
            Ok(())
        }
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
    if source == destination {
        return Ok(fs::metadata(source)
            .with_context(|| format!("failed to read '{}'", source.display()))?
            .len());
    }

    let temp_path = atomic_write_temp_path(destination);
    let _ = fs::remove_file(&temp_path);
    let mut source_file =
        File::open(source).with_context(|| format!("failed to open '{}'", source.display()))?;
    let mut temp_file = File::options()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .with_context(|| format!("failed to create '{}'", temp_path.display()))?;
    let copied = std::io::copy(&mut source_file, &mut temp_file).with_context(|| {
        format!(
            "failed to copy artifact from '{}' to '{}'",
            source.display(),
            temp_path.display()
        )
    })?;
    temp_file
        .sync_all()
        .with_context(|| format!("failed to sync '{}'", temp_path.display()))?;
    drop(temp_file);

    fs::rename(&temp_path, destination).with_context(|| {
        format!(
            "failed to atomically replace '{}' with '{}'",
            destination.display(),
            temp_path.display()
        )
    })?;

    Ok(copied)
}

fn copy_reader_to_path<R: Read>(mut reader: R, destination: &Path) -> Result<u64> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    let mut file = File::create(destination)
        .with_context(|| format!("failed to create '{}'", destination.display()))?;
    std::io::copy(&mut reader, &mut file)
        .with_context(|| format!("failed to write '{}'", destination.display()))
}

fn ensure_runtime_materialized_copy(source: &Path, destination: &Path) -> Result<u64> {
    if runtime_materialized_copy_is_current(source, destination)? {
        return Ok(fs::metadata(destination)
            .with_context(|| format!("failed to read '{}'", destination.display()))?
            .len());
    }

    copy_file(source, destination)
}

fn runtime_materialized_copy_is_current(source: &Path, destination: &Path) -> Result<bool> {
    let source_meta =
        fs::metadata(source).with_context(|| format!("failed to read '{}'", source.display()))?;
    if !source_meta.is_file() {
        bail!("artifact source '{}' does not exist", source.display());
    }

    let destination_meta = match fs::metadata(destination) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read '{}'", destination.display()));
        }
    };
    if !destination_meta.is_file() || destination_meta.len() != source_meta.len() {
        return Ok(false);
    }

    match (source_meta.modified(), destination_meta.modified()) {
        (Ok(source_modified), Ok(destination_modified)) => {
            Ok(destination_modified >= source_modified)
        }
        _ => Ok(true),
    }
}

fn materialize_runtime_rootfs_overlay(
    paths: &RuntimePaths,
    overlay: &MachineRootfsOverlaySpec,
) -> Result<PathBuf> {
    let destination = paths.runtime_dir.join("rootfs-overlay.ext4");
    let expected_size = u64::from(overlay.size_mib) * 1024 * 1024;

    match fs::metadata(&destination) {
        Ok(meta) if meta.is_file() && meta.len() == expected_size => return Ok(destination),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read '{}'", destination.display()));
        }
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }

    let temp_path = atomic_write_temp_path(&destination);
    let _ = fs::remove_file(&temp_path);
    let temp_file = File::options()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .with_context(|| format!("failed to create '{}'", temp_path.display()))?;
    temp_file
        .set_len(expected_size)
        .with_context(|| format!("failed to size '{}'", temp_path.display()))?;
    temp_file
        .sync_all()
        .with_context(|| format!("failed to sync '{}'", temp_path.display()))?;
    drop(temp_file);

    let output = Command::new("mkfs.ext4")
        .args(["-q", "-F", "-L", "port-rootfs-overlay"])
        .arg(&temp_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to run mkfs.ext4 for '{}'", temp_path.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "mkfs.ext4 failed for '{}': {}",
            temp_path.display(),
            stderr.trim()
        );
    }

    fs::rename(&temp_path, &destination).with_context(|| {
        format!(
            "failed to atomically install rootfs overlay '{}' from '{}'",
            destination.display(),
            temp_path.display()
        )
    })?;

    Ok(destination)
}

fn atomic_write_temp_path(destination: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    destination.with_file_name(format!(
        ".{}.{}.tmp",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("port-tmp"),
        stamp
    ))
}

fn artifact_oci_layer_media_type(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Kernel => "application/vnd.port.kernel.v1+binary",
        ArtifactKind::GuestImage => "application/vnd.port.guest-image.v1+ext4",
    }
}

fn oci_pull_scratch_dir(artifact: &ArtifactMetadata) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    artifact
        .cache_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".port-oci-pull-{}-{stamp}", std::process::id()))
}

fn locate_oci_pulled_artifact(scratch_dir: &Path, artifact_path: &Path) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(relative_path) = staged_relative_artifact_path(artifact_path) {
        candidates.push(scratch_dir.join(relative_path));
    }
    if let Some(file_name) = artifact_path.file_name() {
        let by_name = scratch_dir.join(file_name);
        if !candidates.iter().any(|candidate| candidate == &by_name) {
            candidates.push(by_name);
        }
    }

    for candidate in &candidates {
        if candidate.is_file() {
            return Ok(candidate.clone());
        }
    }

    let file_name = artifact_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("artifact"));
    let mut matches = Vec::new();
    let mut pending = vec![scratch_dir.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).with_context(|| {
            format!(
                "failed to enumerate OCI pull staging directory '{}'",
                directory.display()
            )
        })? {
            let path = entry
                .with_context(|| {
                    format!(
                        "failed to inspect OCI pull staging entry under '{}'",
                        directory.display()
                    )
                })?
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name() == Some(file_name) {
                matches.push(path);
            }
        }
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => bail!(
            "no staged artifact named '{}' was found beneath '{}'",
            file_name.to_string_lossy(),
            scratch_dir.display()
        ),
        _ => bail!(
            "multiple staged artifacts named '{}' were found beneath '{}': {}",
            file_name.to_string_lossy(),
            scratch_dir.display(),
            matches
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn staged_relative_artifact_path(artifact_path: &Path) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for component in artifact_path.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            Component::ParentDir => relative.push(".."),
        }
    }
    if relative.as_os_str().is_empty() {
        None
    } else {
        Some(relative)
    }
}

fn format_artifact_selector(selector: ArtifactSelector) -> String {
    format!(
        "{}/{}/{}",
        architecture_dir(selector.architecture),
        substrate_dir(selector.substrate),
        protection_mode_dir(selector.protection_mode)
    )
}

fn summarize_process_output(output: &[u8]) -> String {
    let rendered = String::from_utf8_lossy(output).trim().to_string();
    if rendered.is_empty() {
        String::from("<empty>")
    } else {
        rendered
    }
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
    resolve_artifact_script_path(script_name, artifact_script_candidates(script_name))
}

fn artifact_script_candidates(script_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(configured) = env::var_os("PORT_SHARE_ROOT") {
        push_artifact_script_candidate(
            &mut candidates,
            PathBuf::from(configured.clone())
                .join("scripts/artifacts")
                .join(script_name),
        );
        push_artifact_script_candidate(
            &mut candidates,
            PathBuf::from(configured)
                .join("artifacts")
                .join(script_name),
        );
    }

    if let Some(configured) = env::var_os("PORT_REPO_ROOT") {
        push_artifact_script_candidate(
            &mut candidates,
            PathBuf::from(configured)
                .join("scripts/artifacts")
                .join(script_name),
        );
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(prefix_root) = current_exe.parent().and_then(Path::parent) {
            push_artifact_script_candidate(
                &mut candidates,
                prefix_root
                    .join("share/port/scripts/artifacts")
                    .join(script_name),
            );
            push_artifact_script_candidate(
                &mut candidates,
                prefix_root.join("scripts/artifacts").join(script_name),
            );
            push_artifact_script_candidate(
                &mut candidates,
                prefix_root.join("artifacts").join(script_name),
            );
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        for candidate in current_dir.ancestors() {
            push_artifact_script_candidate(
                &mut candidates,
                candidate.join("scripts/artifacts").join(script_name),
            );
        }
    }

    if cfg!(debug_assertions) {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if let Some(candidate) = manifest_dir.parent().and_then(Path::parent) {
            push_artifact_script_candidate(
                &mut candidates,
                candidate.join("scripts/artifacts").join(script_name),
            );
        }
    }

    candidates
}

fn push_artifact_script_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn resolve_artifact_script_path(
    script_name: &str,
    candidates: impl IntoIterator<Item = PathBuf>,
) -> Result<PathBuf> {
    let candidates: Vec<_> = candidates.into_iter().collect();
    if let Some(path) = candidates.iter().find(|candidate| candidate.is_file()) {
        return Ok(path.clone());
    }

    let searched = candidates
        .iter()
        .map(|candidate| format!("'{}'", candidate.display()))
        .collect::<Vec<_>>()
        .join(", ");
    bail!(
        "artifact pipeline script '{}' is missing; searched {}",
        script_name,
        searched
    )
}

fn is_packaged() -> bool {
    if let Ok(current_exe) = env::current_exe() {
        if let Some(prefix_root) = current_exe.parent().and_then(Path::parent) {
            return prefix_root.join("share/port/scripts/artifacts").is_dir()
                || prefix_root.join("scripts/artifacts").is_dir()
                || prefix_root.join("artifacts").is_dir();
        }
    }
    false
}

fn repo_root() -> Result<PathBuf> {
    if let Some(configured) = env::var_os("PORT_REPO_ROOT") {
        let candidate = PathBuf::from(configured);
        if candidate.join("scripts/artifacts").is_dir() {
            return Ok(candidate);
        }
    }

    if !is_packaged() {
        if let Ok(current_dir) = env::current_dir() {
            for candidate in current_dir.ancestors() {
                if candidate.join("scripts/artifacts").is_dir() {
                    return Ok(candidate.to_path_buf());
                }
            }
        }
    }

    if cfg!(debug_assertions) {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if let Some(candidate) = manifest_dir
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
        {
            if candidate.join("scripts/artifacts").is_dir() {
                return Ok(candidate);
            }
        }
    }

    bail!(
        "failed to resolve the Port repository root; search was restricted because a packaged installation was detected. Set PORT_REPO_ROOT explicitly for development overrides."
    )
}

pub fn execute_guest_operation(
    config: &PortConfig,
    request: GuestRequest<'_>,
) -> Result<OperationResult> {
    let hosted = machine_is_hosted(config, request.machine_name)?;
    if matches!(&request.operation, GuestOperation::Copy(_))
        || (matches!(&request.operation, GuestOperation::Forward(_)) && !hosted)
    {
        bail!("copy and some forward operations use dedicated runtime flows");
    }

    if hosted {
        return hosted_control_plane_guest_operation(config, request);
    }

    let operation = request.operation.clone();
    let (mut reader, mut writer, response) = open_guest_operation_transport(config, &request)?;

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
        ResponseEnvelope::Accepted { stream, .. } => {
            collect_streamed_guest_operation(&operation, stream, &mut writer, &mut reader)
        }
    }
}

pub fn stream_guest_pty<R, W, E>(
    config: &PortConfig,
    request: GuestRequest<'_>,
    input: R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<PtyResult>
where
    R: Read + Send + 'static,
    W: Write,
    E: Write,
{
    if machine_is_hosted(config, request.machine_name)? {
        return match execute_guest_operation(config, request)? {
            OperationResult::Pty(result) => {
                stdout
                    .write_all(result.transcript.as_bytes())
                    .context("failed to write PTY transcript")?;
                Ok(result)
            }
            other => bail!("unexpected guest pty result: {other:?}"),
        };
    }

    let operation = request.operation.clone();
    let (mut reader, writer, response) = open_guest_operation_transport(config, &request)?;
    match response {
        ResponseEnvelope::Accepted {
            stream: port_agent_protocol::StreamKind::Pty,
            ..
        } => consume_streamed_pty(operation, writer, &mut reader, input, stdout, stderr),
        ResponseEnvelope::Completed {
            exit_code: 0,
            result: OperationResult::Pty(result),
            ..
        } => {
            stdout
                .write_all(result.transcript.as_bytes())
                .context("failed to write PTY transcript")?;
            Ok(result)
        }
        ResponseEnvelope::Completed {
            exit_code, result, ..
        } => bail!("guest PTY failed with exit code {exit_code}: {result:?}"),
        ResponseEnvelope::Failed { message, .. } => {
            bail!("guest agent returned an error: {message}")
        }
        ResponseEnvelope::Accepted { stream, .. } => {
            bail!("unexpected streamed PTY handshake: {stream:?}")
        }
    }
}

pub fn stream_guest_logs<W>(
    config: &PortConfig,
    request: GuestRequest<'_>,
    output: &mut W,
) -> Result<LogsResult>
where
    W: Write,
{
    if machine_is_hosted(config, request.machine_name)? {
        return match execute_guest_operation(config, request)? {
            OperationResult::Logs(result) => {
                output
                    .write_all(result.contents.as_bytes())
                    .context("failed to write guest logs")?;
                Ok(result)
            }
            other => bail!("unexpected guest logs result: {other:?}"),
        };
    }

    let (mut reader, _writer, response) = open_guest_operation_transport(config, &request)?;
    match response {
        ResponseEnvelope::Accepted {
            stream: port_agent_protocol::StreamKind::Logs,
            ..
        } => consume_streamed_logs(&mut reader, output),
        ResponseEnvelope::Completed {
            exit_code: 0,
            result: OperationResult::Logs(result),
            ..
        } => {
            output
                .write_all(result.contents.as_bytes())
                .context("failed to write guest logs")?;
            Ok(result)
        }
        ResponseEnvelope::Completed {
            exit_code, result, ..
        } => bail!("guest logs failed with exit code {exit_code}: {result:?}"),
        ResponseEnvelope::Failed { message, .. } => {
            bail!("guest agent returned an error: {message}")
        }
        ResponseEnvelope::Accepted { stream, .. } => {
            bail!("unexpected streamed logs handshake: {stream:?}")
        }
    }
}

fn open_guest_operation_transport(
    config: &PortConfig,
    request: &GuestRequest<'_>,
) -> Result<(
    BufReader<UnixStream>,
    BufWriter<UnixStream>,
    ResponseEnvelope,
)> {
    let driver = driver_for_machine(config, request.machine_name)?;
    let endpoint = driver.guest_endpoint(config, request)?;
    let stream = connect_guest_endpoint(&endpoint)?;
    configure_guest_operation_stream(&stream)?;
    let writer_stream = stream
        .try_clone()
        .context("failed to clone guest agent socket")?;
    let mut writer = BufWriter::new(writer_stream);
    let mut reader = BufReader::new(stream);

    write_frame(
        &mut writer,
        &RequestEnvelope {
            id: 1,
            operation: request.operation.clone(),
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;

    let response: ResponseEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;
    Ok((reader, writer, response))
}

fn collect_streamed_guest_operation(
    operation: &GuestOperation,
    stream: port_agent_protocol::StreamKind,
    writer: &mut BufWriter<UnixStream>,
    reader: &mut BufReader<UnixStream>,
) -> Result<OperationResult> {
    match (operation, stream) {
        (GuestOperation::Pty(_), port_agent_protocol::StreamKind::Pty) => {
            write_frame(writer, &StreamRequestFrame::Close)
                .map_err(|error| anyhow!("protocol error: {error}"))?;
            consume_streamed_pty_frames(reader).map(OperationResult::Pty)
        }
        (GuestOperation::Logs(_), port_agent_protocol::StreamKind::Logs) => {
            consume_streamed_logs_frames(reader).map(OperationResult::Logs)
        }
        _ => bail!("unexpected streamed guest operation handshake: {stream:?}"),
    }
}

fn consume_streamed_pty<R, W, E>(
    operation: GuestOperation,
    writer: BufWriter<UnixStream>,
    reader: &mut BufReader<UnixStream>,
    input: R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<PtyResult>
where
    R: Read + Send + 'static,
    W: Write,
    E: Write,
{
    if !matches!(operation, GuestOperation::Pty(_)) {
        bail!("streamed PTY requested for a non-PTY operation");
    }

    let input_stream = writer
        .get_ref()
        .try_clone()
        .context("failed to clone guest PTY stream for input")?;
    thread::spawn(move || {
        let _ = forward_stream_input(input, input_stream);
    });

    let mut transcript = String::new();
    loop {
        let frame: StreamResponseFrame =
            read_frame(reader).map_err(|error| anyhow!("protocol error: {error}"))?;
        match frame {
            StreamResponseFrame::Data { channel, data } => {
                transcript.push_str(&data);
                match channel {
                    port_agent_protocol::StreamOutputChannel::Stderr => {
                        stderr
                            .write_all(data.as_bytes())
                            .context("failed to write PTY stderr")?;
                        stderr.flush().context("failed to flush PTY stderr")?;
                    }
                    _ => {
                        stdout
                            .write_all(data.as_bytes())
                            .context("failed to write PTY stdout")?;
                        stdout.flush().context("failed to flush PTY stdout")?;
                    }
                }
            }
            StreamResponseFrame::Exit { exit_code: 0 } => {
                return Ok(PtyResult { transcript });
            }
            StreamResponseFrame::Exit { exit_code } => {
                bail!("guest PTY failed with exit code {exit_code}");
            }
            StreamResponseFrame::Error { message } => {
                bail!("guest PTY stream failed: {message}");
            }
            StreamResponseFrame::Eof => {
                bail!("guest PTY stream ended without an exit frame");
            }
        }
    }
}

fn consume_streamed_logs<W>(
    reader: &mut BufReader<UnixStream>,
    output: &mut W,
) -> Result<LogsResult>
where
    W: Write,
{
    let mut contents = String::new();
    loop {
        let frame: StreamResponseFrame =
            read_frame(reader).map_err(|error| anyhow!("protocol error: {error}"))?;
        match frame {
            StreamResponseFrame::Data { data, .. } => {
                contents.push_str(&data);
                output
                    .write_all(data.as_bytes())
                    .context("failed to write guest logs")?;
                output.flush().context("failed to flush guest logs")?;
            }
            StreamResponseFrame::Eof => return Ok(LogsResult { contents }),
            StreamResponseFrame::Error { message } => {
                bail!("guest log stream failed: {message}");
            }
            StreamResponseFrame::Exit { exit_code } => {
                bail!("guest log stream ended with unexpected exit code {exit_code}");
            }
        }
    }
}

fn consume_streamed_pty_frames(reader: &mut BufReader<UnixStream>) -> Result<PtyResult> {
    let mut transcript = String::new();
    loop {
        let frame: StreamResponseFrame =
            read_frame(reader).map_err(|error| anyhow!("protocol error: {error}"))?;
        match frame {
            StreamResponseFrame::Data { data, .. } => transcript.push_str(&data),
            StreamResponseFrame::Exit { exit_code: 0 } => {
                return Ok(PtyResult { transcript });
            }
            StreamResponseFrame::Exit { exit_code } => {
                bail!("guest PTY failed with exit code {exit_code}");
            }
            StreamResponseFrame::Error { message } => {
                bail!("guest PTY stream failed: {message}");
            }
            StreamResponseFrame::Eof => {
                bail!("guest PTY stream ended without an exit frame");
            }
        }
    }
}

fn consume_streamed_logs_frames(reader: &mut BufReader<UnixStream>) -> Result<LogsResult> {
    let mut contents = String::new();
    loop {
        let frame: StreamResponseFrame =
            read_frame(reader).map_err(|error| anyhow!("protocol error: {error}"))?;
        match frame {
            StreamResponseFrame::Data { data, .. } => contents.push_str(&data),
            StreamResponseFrame::Eof => return Ok(LogsResult { contents }),
            StreamResponseFrame::Error { message } => {
                bail!("guest log stream failed: {message}");
            }
            StreamResponseFrame::Exit { exit_code } => {
                bail!("guest log stream ended with unexpected exit code {exit_code}");
            }
        }
    }
}

fn forward_stream_input<R>(mut input: R, stream: UnixStream) -> Result<()>
where
    R: Read,
{
    let mut writer = BufWriter::new(stream);
    let mut buffer = [0_u8; 4096];
    loop {
        let bytes_read = input
            .read(&mut buffer)
            .context("failed to read PTY input")?;
        if bytes_read == 0 {
            break;
        }
        write_frame(
            &mut writer,
            &StreamRequestFrame::Input {
                data: String::from_utf8_lossy(&buffer[..bytes_read]).into_owned(),
            },
        )
        .map_err(|error| anyhow!("protocol error: {error}"))?;
    }
    write_frame(&mut writer, &StreamRequestFrame::Close)
        .map_err(|error| anyhow!("protocol error: {error}"))?;
    Ok(())
}

fn hosted_control_plane_guest_operation(
    config: &PortConfig,
    request: GuestRequest<'_>,
) -> Result<OperationResult> {
    hosted_control_plane_guest_operation_with_timeout(config, request, HOSTED_HTTP_TIMEOUT)
}

fn hosted_control_plane_guest_operation_with_timeout(
    config: &PortConfig,
    request: GuestRequest<'_>,
    timeout: Duration,
) -> Result<OperationResult> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let response: HostedSuccess<OperationResult> = match request.operation {
        GuestOperation::Exec(exec) => client.execute_json_with_timeout(
            client
                .guest()
                .exec(request.machine_name, exec)
                .context("failed to encode hosted guest exec request")?,
            timeout,
        ),
        GuestOperation::Pty(pty) => client.execute_json_with_timeout(
            client
                .guest()
                .pty(request.machine_name, pty)
                .context("failed to encode hosted guest pty request")?,
            timeout,
        ),
        GuestOperation::Logs(logs) => client.execute_json_with_timeout(
            client
                .guest()
                .logs(request.machine_name, logs)
                .context("failed to encode hosted guest logs request")?,
            timeout,
        ),
        GuestOperation::Forward(forward) => client.execute_json_with_timeout(
            client
                .guest()
                .forward(request.machine_name, forward)
                .context("failed to encode hosted guest forward request")?,
            timeout,
        ),
        GuestOperation::ManagedService(_) => {
            bail!("managed service uses the canonical service control path")
        }
        GuestOperation::Copy(_) => bail!("copy uses a dedicated runtime flow"),
        GuestOperation::Ping => {
            bail!("ping uses the node-agent guest heartbeat probe loop, not the hosted guest RPC")
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

pub fn start_hosted_detached_forward(
    config: &PortConfig,
    machine_name: &str,
    listen: &str,
    target: &str,
    name: Option<&str>,
) -> Result<HostedDetachedForwardStatusContract> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<HostedDetachedForwardStatusContract> = client
        .execute_json(
            client
                .guest()
                .forward_detached_start(
                    machine_name,
                    HostedDetachedForwardStartRequest {
                        listen: listen.to_string(),
                        target: target.to_string(),
                        name: name.map(ToOwned::to_owned),
                    },
                )
                .context("failed to encode hosted detached forward start request")?,
        )
        .map_err(|error| {
            anyhow!(
                "failed to start detached forward for machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

pub fn list_hosted_detached_forwards(
    config: &PortConfig,
    machine_name: &str,
) -> Result<Vec<HostedDetachedForwardStatusContract>> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<Vec<HostedDetachedForwardStatusContract>> = client
        .execute_json(client.guest().forward_detached_list(machine_name))
        .map_err(|error| {
            anyhow!(
                "failed to list detached forwards for machine '{}' through the live hosted control-plane route: {error}",
                machine_name
            )
        })?;
    Ok(response.result)
}

pub fn stop_hosted_detached_forward(
    config: &PortConfig,
    machine_name: &str,
    forward_name: &str,
) -> Result<HostedDetachedForwardStopResult> {
    let client = hosted_client_for_machine(config, machine_name)?;
    let response: HostedSuccess<HostedDetachedForwardStopResult> = client
        .execute_json(client.guest().forward_detached_stop(machine_name, forward_name))
        .map_err(|error| {
            anyhow!(
                "failed to stop detached forward '{}' for machine '{}' through the live hosted control-plane route: {error}",
                forward_name, machine_name
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

    let copy_request = copy_protocol_request(&request)?;
    match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            let mut source = File::open(request.source)
                .with_context(|| format!("failed to open '{}'", request.source.display()))?;
            copy_guest_via_endpoint(
                config,
                request.machine_name,
                request.runtime_root,
                copy_request,
                Some(&mut source),
                None,
            )
        }
        port_agent_protocol::CopyDirection::GuestToHost => {
            if let Some(parent) = request.destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            let mut destination = File::create(request.destination)
                .with_context(|| format!("failed to create '{}'", request.destination.display()))?;
            copy_guest_via_endpoint(
                config,
                request.machine_name,
                request.runtime_root,
                copy_request,
                None,
                Some(&mut destination),
            )
        }
    }
}

fn hosted_control_plane_copy_guest_file(
    config: &PortConfig,
    request: GuestCopyRequest<'_>,
) -> Result<port_agent_protocol::CopyResult> {
    let client = hosted_client_for_machine(config, request.machine_name)?;
    let copy_request = copy_protocol_request(&request)?;
    let stream_request = client
        .guest()
        .copy_stream(request.machine_name, copy_request.clone())
        .context("failed to encode hosted guest copy request")?;
    let response = match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            let source = File::open(request.source)
                .with_context(|| format!("failed to open '{}'", request.source.display()))?;
            let prefix = encode_copy_request_envelope(&copy_request)?;
            execute_hosted_stream_request(
                stream_request,
                reqwest::blocking::Body::new(PrefixedReader::new(prefix, source)),
            )
        }
        port_agent_protocol::CopyDirection::GuestToHost => {
            let prefix = encode_copy_request_envelope(&copy_request)?;
            execute_hosted_stream_request(
                stream_request,
                reqwest::blocking::Body::new(Cursor::new(prefix)),
            )
        }
    }
    .map_err(|error| {
        anyhow!(
            "failed to copy files for machine '{}' through the live hosted control-plane stream route: {error}",
            request.machine_name
        )
    })?;

    let mut reader = BufReader::new(response);
    match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            match read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))? {
                ResponseEnvelope::Accepted {
                    stream: port_agent_protocol::StreamKind::Bytes,
                    ..
                } => parse_copy_completion_response(
                    read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?,
                ),
                response => parse_copy_completion_response(response),
            }
        }
        port_agent_protocol::CopyDirection::GuestToHost => {
            let response =
                read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;
            let size_bytes = match response {
                ResponseEnvelope::Accepted {
                    stream: port_agent_protocol::StreamKind::Bytes,
                    size_bytes: Some(size_bytes),
                    ..
                } => size_bytes,
                other => return parse_copy_completion_response(other),
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

            parse_copy_completion_response(
                read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?,
            )
        }
    }
}

fn copy_protocol_request(
    request: &GuestCopyRequest<'_>,
) -> Result<port_agent_protocol::CopyRequest> {
    let size_bytes = match request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => Some(
            fs::metadata(request.source)
                .with_context(|| format!("failed to stat '{}'", request.source.display()))?
                .len(),
        ),
        port_agent_protocol::CopyDirection::GuestToHost => None,
    };

    Ok(port_agent_protocol::CopyRequest {
        source: request.source.display().to_string(),
        destination: request.destination.display().to_string(),
        direction: request.direction,
        size_bytes,
    })
}

fn copy_guest_via_endpoint(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &Path,
    copy_request: port_agent_protocol::CopyRequest,
    upload: Option<&mut dyn Read>,
    download: Option<&mut dyn Write>,
) -> Result<port_agent_protocol::CopyResult> {
    let driver = driver_for_machine(config, machine_name)?;
    let endpoint = driver.guest_endpoint(
        config,
        &GuestRequest {
            machine_name,
            runtime_root,
            operation: GuestOperation::Copy(copy_request.clone()),
        },
    )?;
    let stream = connect_guest_endpoint(&endpoint)?;
    configure_guest_operation_stream(&stream)?;
    let writer_stream = stream
        .try_clone()
        .context("failed to clone guest agent socket")?;
    let mut writer = BufWriter::new(writer_stream);
    let mut reader = BufReader::new(stream);

    write_frame(
        &mut writer,
        &RequestEnvelope {
            id: 1,
            operation: GuestOperation::Copy(copy_request.clone()),
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;

    match copy_request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            match read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))? {
                ResponseEnvelope::Accepted {
                    stream: port_agent_protocol::StreamKind::Bytes,
                    ..
                } => {}
                response => return parse_copy_completion_response(response),
            }

            let size_bytes = copy_request
                .size_bytes
                .context("host-to-guest copy requires size_bytes")?;
            let upload = upload.context("host-to-guest copy requires an upload reader")?;
            let mut limited = upload.take(size_bytes);
            let bytes_copied = std::io::copy(&mut limited, &mut writer)
                .context("failed to stream host-to-guest copy bytes")?;
            if bytes_copied != size_bytes {
                bail!("expected {size_bytes} bytes for guest copy, sent {bytes_copied}");
            }
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
                response => return parse_copy_completion_response(response),
            };

            let download = download.context("guest-to-host copy requires a destination writer")?;
            let mut limited = reader.by_ref().take(size_bytes);
            let bytes_copied = std::io::copy(&mut limited, download)
                .context("failed to stream guest-to-host copy bytes")?;
            if bytes_copied != size_bytes {
                bail!("expected {size_bytes} bytes from guest copy, received {bytes_copied}");
            }
        }
    }

    parse_copy_completion_response(
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?,
    )
}

fn parse_copy_completion_response(
    response: ResponseEnvelope,
) -> Result<port_agent_protocol::CopyResult> {
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

fn encode_copy_request_envelope(request: &port_agent_protocol::CopyRequest) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();
    write_frame(
        &mut encoded,
        &RequestEnvelope {
            id: 1,
            operation: GuestOperation::Copy(request.clone()),
        },
    )
    .map_err(|error| anyhow!("protocol error: {error}"))?;
    Ok(encoded)
}

fn execute_hosted_request(request: HostedApiRequest) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::builder()
        .timeout(HOSTED_HTTP_TIMEOUT)
        .build()
        .context("failed to build hosted HTTP client")?;
    let method = match request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
    };

    let mut builder = client.request(method.clone(), &request.url);
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    if let Some(body) = &request.body {
        builder = builder.json(body);
    }
    let response = builder.send().map_err(|error| {
        if error.is_timeout() {
            anyhow!(
                "hosted request {} {} timed out after {:?}",
                request.method,
                request.url,
                HOSTED_HTTP_TIMEOUT
            )
        } else {
            anyhow!(
                "failed to send hosted request {} {}: {error}",
                request.method,
                request.url
            )
        }
    })?;
    finalize_hosted_response(request.method, &request.url, response)
}

fn execute_hosted_stream_request(
    request: HostedApiStreamRequest,
    body: reqwest::blocking::Body,
) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::builder()
        .timeout(HOSTED_HTTP_TIMEOUT)
        .build()
        .context("failed to build hosted HTTP client")?;
    let method = match request.request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
    };

    let mut builder = client.request(method.clone(), &request.request.url);
    for (name, value) in &request.request.headers {
        if name.eq_ignore_ascii_case("content-type") {
            continue;
        }
        builder = builder.header(name, value);
    }
    builder = builder.header("content-type", "application/octet-stream");
    let response = builder.body(body).send().map_err(|error| {
        if error.is_timeout() {
            anyhow!(
                "hosted request {} {} timed out after {:?}",
                request.request.method,
                request.request.url,
                HOSTED_HTTP_TIMEOUT
            )
        } else {
            anyhow!(
                "failed to send hosted request {} {}: {error}",
                request.request.method,
                request.request.url
            )
        }
    })?;
    finalize_hosted_response(request.request.method, &request.request.url, response)
}

fn finalize_hosted_response(
    method: HttpMethod,
    url: &str,
    response: reqwest::blocking::Response,
) -> Result<reqwest::blocking::Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let bytes = response
        .bytes()
        .with_context(|| format!("failed to read hosted error body for {url}"))?;
    if let Ok(error) = serde_json::from_slice::<HostedError>(&bytes) {
        bail!(
            "hosted request {} {} failed with {}: {}{}",
            method,
            url,
            status,
            error.message,
            render_hosted_route_context(error.route.as_ref()),
        );
    }

    let fallback = status
        .canonical_reason()
        .unwrap_or("unknown error")
        .to_string();
    bail!(
        "hosted request {} {} failed with {}: {}",
        method,
        url,
        status,
        fallback,
    );
}

fn render_hosted_route_context(route: Option<&HostedRouteContext>) -> String {
    let Some(route) = route else {
        return String::new();
    };

    let mut parts = Vec::new();
    if let Some(control_plane) = &route.control_plane {
        parts.push(format!("control-plane={control_plane}"));
    }
    if let Some(machine_name) = &route.machine_name {
        parts.push(format!("machine={machine_name}"));
    }
    if let Some(forward_name) = &route.forward_name {
        parts.push(format!("forward={forward_name}"));
    }
    if let Some(service_name) = &route.service_name {
        parts.push(format!("service={service_name}"));
    }
    if let Some(node_name) = &route.node_name {
        parts.push(format!("node={node_name}"));
    }
    if !route.host_groups.is_empty() {
        parts.push(format!("host-groups={}", route.host_groups.join(",")));
    }
    if !route.candidate_nodes.is_empty() {
        parts.push(format!(
            "candidate-nodes={}",
            route.candidate_nodes.join(",")
        ));
    }
    if !route.rejected_nodes.is_empty() {
        let rejected = route
            .rejected_nodes
            .iter()
            .map(|(node_name, reason)| format!("{node_name}({reason})"))
            .collect::<Vec<_>>()
            .join(",");
        parts.push(format!("rejected-nodes={rejected}"));
    }
    if let Some(runtime_root) = &route.runtime_root {
        parts.push(format!("runtime-root={}", runtime_root.display()));
    }
    if let Some(placement_detail) = &route.placement_detail {
        parts.push(format!("placement={placement_detail}"));
    }
    if let Some(guest_session) = &route.guest_session {
        parts.push(format!("session={}", guest_session.id));
        parts.push(format!("session-scope={}", guest_session.scope));
        parts.push(format!("driver={}", guest_session.driver.id));
        parts.push(format!("driver-route={}", guest_session.driver.route));
        parts.push(format!("driver-broker={}", guest_session.driver.broker));
        parts.push(format!("driver-protocol={}", guest_session.driver.protocol));
        if !guest_session.driver.command_surface.is_empty() {
            parts.push(format!(
                "driver-commands={}",
                guest_session
                    .driver
                    .command_surface
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(" "))
    }
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
        // Track consecutive proxy failures. When the guest transport is
        // persistently dead (vsock gone, VM crashed), exit so the
        // orchestration layer can detect the broken forward and restart it
        // instead of silently black-holing TCP connections.
        const MAX_CONSECUTIVE_FAILURES: u32 = 10;
        let (result_tx, result_rx) = std::sync::mpsc::channel::<bool>();

        macro_rules! forward_accept_loop {
            ($listener:expr, $label:expr) => {{
                let mut consecutive_failures: u32 = 0;
                for inbound in $listener.incoming() {
                    let inbound = inbound.with_context(|| {
                        format!("failed to accept forwarded {} connection", $label)
                    })?;

                    // Drain results from completed proxy threads.
                    while let Ok(success) = result_rx.try_recv() {
                        if success {
                            consecutive_failures = 0;
                        } else {
                            consecutive_failures += 1;
                        }
                    }
                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        bail!(
                            "forward daemon exiting: {} consecutive proxy failures; \
                             the guest transport is likely dead",
                            consecutive_failures
                        );
                    }

                    let endpoint = self.endpoint.clone();
                    let target = self.target.clone();
                    let tx = result_tx.clone();
                    thread::spawn(move || {
                        let ok = match proxy_guest_forward_connection(endpoint, target, inbound) {
                            Ok(()) => true,
                            Err(error) => {
                                eprintln!("port guest forward connection failed: {error}");
                                false
                            }
                        };
                        let _ = tx.send(ok);
                    });
                }
            }};
        }

        match self.listener {
            ForwardListener::Tcp(ref listener) => {
                forward_accept_loop!(listener, "host");
            }
            ForwardListener::Unix {
                ref listener,
                ref socket_path,
            } => {
                forward_accept_loop!(listener, "Unix-socket");
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
        HostConnection::Ssh {
            destination,
            user,
            port,
        } => {
            bail!(
                "machine '{}' targets ssh-managed host '{}' through {}@{}:{} but guest runtime-root resolution is not implemented yet",
                machine_name,
                machine.host,
                user,
                destination,
                port
            )
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
    configure_guest_operation_stream(&stream)?;
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
    set_guest_transport_timeouts(&guest_stream, None, "guest forward transport")?;
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
    VsockTunnel {
        backend_name: &'static str,
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
        return Ok(GuestEndpoint::VsockTunnel {
            backend_name: "Firecracker",
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

fn resolve_cloud_hypervisor_guest_endpoint(
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
        return Ok(GuestEndpoint::VsockTunnel {
            backend_name: "Cloud Hypervisor",
            host_socket_path: paths.vsock_path,
            guest_port: u32::from(machine.guest.control_port),
        });
    }

    if paths.manifest_path.exists() {
        bail!(
            "launched Cloud Hypervisor machine '{}' does not expose a live guest transport socket at '{}' or '{}'; inspect the runtime logs or relaunch the VM",
            request.machine_name,
            paths.guest_agent_socket.display(),
            paths.vsock_path.display()
        );
    }

    bail!(
        "guest agent socket '{}' does not exist for machine '{}'",
        paths.guest_agent_socket.display(),
        request.machine_name
    );
}

fn resolve_avf_guest_endpoint(
    config: &PortConfig,
    request: &GuestRequest<'_>,
) -> Result<GuestEndpoint> {
    let runtime_root =
        resolve_guest_runtime_root(config, request.machine_name, request.runtime_root)?;
    let paths = RuntimePaths::for_machine(runtime_root, request.machine_name);

    if paths.guest_agent_socket.exists() {
        return Ok(GuestEndpoint::RuntimeSocket(paths.guest_agent_socket));
    }

    if paths.manifest_path.exists() {
        bail!(
            "launched AVF machine '{}' does not expose a live guest transport socket at '{}'; inspect '{}' or relaunch the VM",
            request.machine_name,
            paths.guest_agent_socket.display(),
            paths.firecracker_log.display()
        );
    }

    bail!(
        "guest agent socket '{}' does not exist for machine '{}'",
        paths.guest_agent_socket.display(),
        request.machine_name
    );
}

fn set_guest_transport_timeouts(
    stream: &UnixStream,
    timeout: Option<Duration>,
    label: &str,
) -> Result<()> {
    stream
        .set_read_timeout(timeout)
        .with_context(|| format!("failed to set {label} read timeout"))?;
    stream
        .set_write_timeout(timeout)
        .with_context(|| format!("failed to set {label} write timeout"))?;
    Ok(())
}

fn configure_guest_operation_stream(stream: &UnixStream) -> Result<()> {
    set_guest_transport_timeouts(stream, Some(GUEST_TRANSPORT_IO_TIMEOUT), "guest transport")
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
        GuestEndpoint::VsockTunnel {
            backend_name,
            host_socket_path,
            guest_port,
        } => connect_vsock_tunnel(backend_name, host_socket_path, *guest_port),
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

impl MachineDriver for CloudHypervisorLocalDriver {
    fn kind(&self) -> MachineDriverKind {
        MachineDriverKind::CloudHypervisorLocal
    }

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata> {
        cloud_hypervisor_local_launch_machine(config, request)
    }

    fn list_machines(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
    ) -> Result<Vec<MachineStatus>> {
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
            let paths = RuntimePaths::for_machine(runtime_root, &machine_name);
            if cloud_hypervisor_runtime_metadata_path(&paths).exists() {
                machines.push(cloud_hypervisor_local_machine_status(
                    runtime_root,
                    &machine_name,
                )?);
            }
        }
        machines.sort_by(|left, right| left.machine_name.cmp(&right.machine_name));
        Ok(machines)
    }

    fn machine_status(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus> {
        cloud_hypervisor_local_machine_status(runtime_root, machine_name)
    }

    fn stop_machine(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult> {
        cloud_hypervisor_local_stop_machine(runtime_root, machine_name, timeout)
    }

    fn machine_monitor(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport> {
        cloud_hypervisor_local_machine_monitor(runtime_root, machine_name)
    }

    fn machine_top(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport> {
        cloud_hypervisor_local_machine_top(runtime_root, machine_name)
    }

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint> {
        resolve_cloud_hypervisor_guest_endpoint(config, request)
    }
}

impl MachineDriver for AvfLocalDriver {
    fn kind(&self) -> MachineDriverKind {
        MachineDriverKind::AvfLocal
    }

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata> {
        avf_local_launch_machine(config, request)
    }

    fn list_machines(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
    ) -> Result<Vec<MachineStatus>> {
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
            let paths = RuntimePaths::for_machine(runtime_root, &machine_name);
            if avf_runtime_metadata_path(&paths).exists() {
                machines.push(avf_local_machine_status(runtime_root, &machine_name)?);
            }
        }
        machines.sort_by(|left, right| left.machine_name.cmp(&right.machine_name));
        Ok(machines)
    }

    fn machine_status(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus> {
        avf_local_machine_status(runtime_root, machine_name)
    }

    fn stop_machine(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult> {
        avf_local_stop_machine(runtime_root, machine_name, timeout)
    }

    fn machine_monitor(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport> {
        avf_local_machine_monitor(runtime_root, machine_name)
    }

    fn machine_top(
        &self,
        _config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport> {
        avf_local_machine_top(runtime_root, machine_name)
    }

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint> {
        resolve_avf_guest_endpoint(config, request)
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

impl MachineDriver for SshManagedDriver {
    fn kind(&self) -> MachineDriverKind {
        MachineDriverKind::SshManagedRemote
    }

    fn launch(&self, config: &PortConfig, request: &LaunchRequest<'_>) -> Result<LaunchMetadata> {
        ssh_managed_launch_machine(config, request)
    }

    fn list_machines(
        &self,
        _config: &PortConfig,
        _runtime_root: &Path,
    ) -> Result<Vec<MachineStatus>> {
        bail!(
            "ssh-managed remote inventory listing is not implemented yet; use `port machine status --machine <name>` for the first bounded lifecycle slice"
        )
    }

    fn machine_status(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineStatus> {
        ssh_managed_machine_status(config, runtime_root, machine_name)
    }

    fn stop_machine(
        &self,
        config: &PortConfig,
        runtime_root: &Path,
        machine_name: &str,
        timeout: Duration,
    ) -> Result<StopResult> {
        ssh_managed_stop_machine(config, runtime_root, machine_name, timeout)
    }

    fn machine_monitor(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineMonitorReport> {
        let target = ssh_machine_target(config, machine_name)?;
        bail!(
            "{}; machine monitor is not implemented for the first ssh-managed lifecycle slice",
            target.route_context(target.control.monitor_route)
        )
    }

    fn machine_top(
        &self,
        config: &PortConfig,
        _runtime_root: &Path,
        machine_name: &str,
    ) -> Result<MachineTopReport> {
        let target = ssh_machine_target(config, machine_name)?;
        bail!(
            "{}; machine top is not implemented for the first ssh-managed lifecycle slice",
            target.route_context(target.control.top_route)
        )
    }

    fn guest_endpoint(
        &self,
        config: &PortConfig,
        request: &GuestRequest<'_>,
    ) -> Result<GuestEndpoint> {
        let target = ssh_machine_target(config, request.machine_name)?;
        bail!(
            "{}; guest operations are not implemented for the first ssh-managed lifecycle slice",
            target.route_context(target.control.guest_route)
        )
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
            ExecutionSubstrate::CloudHypervisor => Ok(Box::new(CloudHypervisorLocalDriver)),
            ExecutionSubstrate::Avf => Ok(Box::new(AvfLocalDriver)),
        },
        HostConnection::Ssh { .. } => Ok(Box::new(SshManagedDriver)),
    }
}

fn connect_vsock_tunnel(
    backend_name: &str,
    host_socket_path: &Path,
    guest_port: u32,
) -> Result<UnixStream> {
    connect_vsock_tunnel_with_timeout(
        backend_name,
        host_socket_path,
        guest_port,
        GUEST_TRANSPORT_HANDSHAKE_TIMEOUT,
    )
}

fn connect_vsock_tunnel_with_timeout(
    backend_name: &str,
    host_socket_path: &Path,
    guest_port: u32,
    handshake_timeout: Duration,
) -> Result<UnixStream> {
    let mut stream = UnixStream::connect(host_socket_path).with_context(|| {
        format!(
            "failed to connect to {backend_name} guest transport socket '{}'",
            host_socket_path.display(),
        )
    })?;
    set_guest_transport_timeouts(
        &stream,
        Some(handshake_timeout),
        "guest transport handshake",
    )?;
    stream
        .write_all(format!("CONNECT {guest_port}\n").as_bytes())
        .with_context(|| {
            format!(
                "failed to request {backend_name} guest transport port {} via '{}'",
                guest_port,
                host_socket_path.display()
            )
        })?;
    stream
        .flush()
        .with_context(|| format!("failed to flush {backend_name} handshake"))?;

    let reader_stream = stream
        .try_clone()
        .with_context(|| format!("failed to clone {backend_name} guest transport socket"))?;
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();
    reader.read_line(&mut line).with_context(|| {
        format!(
            "failed to read {backend_name} response from '{}'",
            host_socket_path.display(),
        )
    })?;

    if !line.starts_with("OK") {
        let detail = line.trim();
        bail!(
            "{backend_name} refused to establish a guest transport tunnel to port {} via '{}': {}",
            guest_port,
            host_socket_path.display(),
            if detail.is_empty() {
                "empty response"
            } else {
                detail
            }
        );
    }

    set_guest_transport_timeouts(&stream, None, "guest transport handshake")?;
    Ok(stream)
}

#[allow(clippy::too_many_arguments)]
fn build_firecracker_config(
    kernel_image_path: PathBuf,
    rootfs_path: PathBuf,
    rootfs_overlay_path: Option<PathBuf>,
    attached_volumes: &[MachineVolumeSpec],
    vcpu_count: u8,
    mem_size_mib: u32,
    boot_args: String,
    rootfs_read_only: bool,
    guest_control_port: u16,
    guest_cid: u32,
    uds_path: PathBuf,
    machine_name: &str,
    network: Option<&port_model::MachineNetworkSpec>,
) -> FirecrackerConfig {
    let mut boot_args =
        format!("{boot_args} init=/init port.guest_control_port={guest_control_port}");
    if rootfs_overlay_path.is_some() {
        boot_args =
            format!("{boot_args} port.rootfs_overlay=1 port.rootfs_overlay_device=/dev/vdb");
    }
    let (network_interfaces, _) = match network {
        Some(net) if net.enabled => {
            boot_args = format!(
                "{boot_args} port.net_ip={} port.net_gateway={} port.net_prefix_len={}",
                net.guest_ip, net.host_ip, net.prefix_len
            );
            if !net.dns_servers.is_empty() {
                boot_args = format!("{boot_args} port.net_dns={}", net.dns_servers.join(","));
            }
            (
                vec![NetworkInterfaceConfig {
                    iface_id: String::from("eth0"),
                    host_dev_name: tap_device_name(machine_name),
                    guest_mac: net.guest_mac.clone(),
                }],
                true,
            )
        }
        _ => (Vec::new(), false),
    };
    let initrd_path = firecracker_initrd_path_for_rootfs(&rootfs_path);
    let mut drives = vec![DriveConfig {
        drive_id: String::from("rootfs"),
        path_on_host: rootfs_path,
        is_root_device: true,
        is_read_only: rootfs_read_only,
    }];
    if let Some(path_on_host) = rootfs_overlay_path {
        drives.push(DriveConfig {
            drive_id: String::from("rootfs-overlay"),
            path_on_host,
            is_root_device: false,
            is_read_only: false,
        });
    }
    drives.extend(attached_volumes.iter().map(|volume| DriveConfig {
        drive_id: volume.name.clone(),
        path_on_host: volume.path.clone(),
        is_root_device: false,
        is_read_only: false,
    }));

    FirecrackerConfig {
        boot_source: BootSourceConfig {
            kernel_image_path,
            initrd_path,
            boot_args,
        },
        drives,
        machine_config: MachineConfig {
            vcpu_count,
            mem_size_mib,
            smt: false,
        },
        vsock: VsockConfig {
            guest_cid,
            uds_path,
        },
        network_interfaces,
    }
}

fn tap_device_name(machine_name: &str) -> String {
    let base = format!("port-{machine_name}");
    if base.len() <= 15 {
        base
    } else {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        machine_name.hash(&mut hasher);
        format!("port-{:08x}", hasher.finish() as u32)
    }
}

fn network_state_path(paths: &RuntimePaths) -> PathBuf {
    paths.runtime_dir.join("network-state.json")
}

pub(crate) fn run_network_command(program: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to run '{program}' with args {args:?}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "'{program} {}' exited with {}: {stderr}",
            args.join(" "),
            output.status
        );
    }
    Ok(())
}

pub(crate) fn iptables_binary() -> String {
    env::var(PORT_IPTABLES_BINARY_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from("iptables"))
}

pub(crate) fn iproute_binary() -> String {
    find_versioned_binary("ip", &["-V"], "iproute2")
        .unwrap_or_else(|| PathBuf::from("ip"))
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn default_outbound_interface() -> Result<String> {
    let output = Command::new(iproute_binary())
        .args(["route", "show", "default"])
        .output()
        .context("failed to run 'ip route show default'")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .skip_while(|w| *w != "dev")
        .nth(1)
        .map(|s| s.to_string())
        .context("failed to determine default outbound interface from 'ip route show default'")
}

fn setup_host_networking(
    machine_name: &str,
    network: &port_model::MachineNetworkSpec,
) -> Result<()> {
    // Retrying a failed launch must start from a clean host-networking slate.
    // Prior attempts may have already created the TAP device before failing.
    teardown_host_networking(machine_name, network);

    let iptables_binary = iptables_binary();
    let iproute_binary = iproute_binary();
    let tap_name = tap_device_name(machine_name);
    let host_cidr = format!("{}/{}", network.host_ip, network.prefix_len);
    let subnet = format!("{}/{}", network.guest_ip, network.prefix_len);
    let outbound_iface = default_outbound_interface()?;

    let result: Result<()> = (|| {
        run_network_command(
            &iproute_binary,
            &["tuntap", "add", "dev", &tap_name, "mode", "tap"],
        )
        .with_context(|| format!("failed to create TAP device '{tap_name}'"))?;
        run_network_command(
            &iproute_binary,
            &["addr", "add", &host_cidr, "dev", &tap_name],
        )
        .with_context(|| format!("failed to assign address {host_cidr} to '{tap_name}'"))?;
        run_network_command(&iproute_binary, &["link", "set", &tap_name, "up"])
            .with_context(|| format!("failed to bring up '{tap_name}'"))?;

        // Enable proxy ARP so this TAP answers ARP requests for other guests
        // on the same subnet, allowing inter-VM traffic to be routed through
        // the host.
        let proxy_arp_path = format!("/proc/sys/net/ipv4/conf/{tap_name}/proxy_arp");
        fs::write(&proxy_arp_path, "1")
            .with_context(|| format!("failed to enable proxy_arp on '{tap_name}'"))?;

        // Add a host route for this guest's IP so the kernel routes traffic
        // to the correct TAP even when multiple TAPs share a subnet.
        let guest_host_route = format!("{}/32", network.guest_ip);
        let _ = run_network_command(
            &iproute_binary,
            &["route", "add", &guest_host_route, "dev", &tap_name],
        );

        fs::write("/proc/sys/net/ipv4/ip_forward", "1").context("failed to enable ip_forward")?;

        run_network_command(
            &iptables_binary,
            &[
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-s",
                &subnet,
                "-o",
                &outbound_iface,
                "-j",
                "MASQUERADE",
            ],
        )
        .context("failed to add iptables MASQUERADE rule")?;
        run_network_command(
            &iptables_binary,
            &[
                "-A",
                "FORWARD",
                "-i",
                &tap_name,
                "-o",
                &outbound_iface,
                "-j",
                "ACCEPT",
            ],
        )
        .context("failed to add iptables FORWARD accept rule")?;
        run_network_command(
            &iptables_binary,
            &[
                "-A",
                "FORWARD",
                "-i",
                &outbound_iface,
                "-o",
                &tap_name,
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )
        .context("failed to add iptables FORWARD established rule")?;

        // Allow forwarding between guest TAP devices on the same subnet so
        // multi-VM clusters (e.g. K3s server + workers) can communicate.
        let guest_subnet = format!("{}/{}", network.host_ip, network.prefix_len);
        run_network_command(
            &iptables_binary,
            &[
                "-A",
                "FORWARD",
                "-s",
                &guest_subnet,
                "-d",
                &guest_subnet,
                "-j",
                "ACCEPT",
            ],
        )
        .context("failed to add iptables FORWARD inter-vm rule")?;

        Ok(())
    })();

    if result.is_err() {
        teardown_host_networking(machine_name, network);
    }

    result
}

fn teardown_host_networking(machine_name: &str, network: &port_model::MachineNetworkSpec) {
    let iptables_binary = iptables_binary();
    let iproute_binary = iproute_binary();
    let tap_name = tap_device_name(machine_name);
    let subnet = format!("{}/{}", network.guest_ip, network.prefix_len);

    if let Ok(outbound_iface) = default_outbound_interface() {
        let _ = run_network_command(
            &iptables_binary,
            &[
                "-t",
                "nat",
                "-D",
                "POSTROUTING",
                "-s",
                &subnet,
                "-o",
                &outbound_iface,
                "-j",
                "MASQUERADE",
            ],
        );
        let _ = run_network_command(
            &iptables_binary,
            &[
                "-D",
                "FORWARD",
                "-i",
                &tap_name,
                "-o",
                &outbound_iface,
                "-j",
                "ACCEPT",
            ],
        );
        let _ = run_network_command(
            &iptables_binary,
            &[
                "-D",
                "FORWARD",
                "-i",
                &outbound_iface,
                "-o",
                &tap_name,
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        );
    }

    let guest_host_route = format!("{}/32", network.guest_ip);
    let _ = run_network_command(
        &iproute_binary,
        &["route", "del", &guest_host_route, "dev", &tap_name],
    );

    let _ = run_network_command(&iproute_binary, &["link", "del", &tap_name]);
}

fn teardown_host_networking_from_state(paths: &RuntimePaths, machine_name: &str) {
    let state_path = network_state_path(paths);
    if !state_path.exists() {
        return;
    }
    if let Ok(bytes) = fs::read(&state_path) {
        if let Ok(network) = serde_json::from_slice::<port_model::MachineNetworkSpec>(&bytes) {
            teardown_host_networking(machine_name, &network);
        }
    }
    let _ = fs::remove_file(&state_path);
}

/// Transfer ownership of a runtime directory tree to the user who invoked
/// `sudo`, identified by `SUDO_UID` / `SUDO_GID`.  No-op when not running
/// under sudo.
pub fn chown_runtime_to_sudo_caller(dir: &Path) -> Result<()> {
    let (uid, gid) = match sudo_caller_ids() {
        Some(ids) => ids,
        None => return Ok(()),
    };
    chown_recursive(dir, uid, gid)
        .with_context(|| format!("failed to chown '{}' to {uid}:{gid}", dir.display()))
}

fn sudo_caller_ids() -> Option<(u32, u32)> {
    let uid: u32 = env::var("SUDO_UID").ok()?.parse().ok()?;
    let gid: u32 = env::var("SUDO_GID").ok()?.parse().ok()?;
    Some((uid, gid))
}

fn chown_recursive(path: &Path, uid: u32, gid: u32) -> Result<()> {
    chown_path(path, uid, gid)?;
    if path.is_dir() {
        for entry in fs::read_dir(path)
            .with_context(|| format!("failed to read directory '{}'", path.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read entry in '{}'", path.display()))?;
            chown_recursive(&entry.path(), uid, gid)?;
        }
    }
    Ok(())
}

fn chown_path(path: &Path, uid: u32, gid: u32) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .context("path contains interior null byte")?;
    if unsafe { libc::chown(c_path.as_ptr(), uid, gid) } != 0 {
        bail!(
            "chown '{}' to {uid}:{gid}: {}",
            path.display(),
            std::io::Error::last_os_error()
        );
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct FirecrackerConfig {
    #[serde(rename = "boot-source")]
    boot_source: BootSourceConfig,
    drives: Vec<DriveConfig>,
    #[serde(rename = "machine-config")]
    machine_config: MachineConfig,
    vsock: VsockConfig,
    #[serde(rename = "network-interfaces", skip_serializing_if = "Vec::is_empty")]
    network_interfaces: Vec<NetworkInterfaceConfig>,
}

#[derive(Debug, Serialize)]
struct BootSourceConfig {
    kernel_image_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    initrd_path: Option<PathBuf>,
    boot_args: String,
}

fn firecracker_initrd_path_for_rootfs(rootfs_path: &Path) -> Option<PathBuf> {
    let candidate = rootfs_path.with_file_name("initrd.cpio.gz");
    candidate.is_file().then_some(candidate)
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

#[derive(Debug, Serialize)]
struct NetworkInterfaceConfig {
    iface_id: String,
    host_dev_name: String,
    guest_mac: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AvfLaunchConfig {
    machine_name: String,
    runtime_dir: PathBuf,
    kernel_path: PathBuf,
    guest_image_path: PathBuf,
    vcpu_count: u8,
    memory_mib: u32,
    kernel_args: String,
    rootfs_read_only: bool,
    guest_vsock_cid: u32,
    guest_control_port: u16,
    guest_agent_socket: PathBuf,
    guest_transport: port_model::AvfGuestTransport,
    console_transport: port_model::AvfConsoleTransport,
    console_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CloudHypervisorLaunchConfig {
    machine_name: String,
    runtime_dir: PathBuf,
    kernel_path: PathBuf,
    guest_image_path: PathBuf,
    vcpu_count: u8,
    memory_mib: u32,
    kernel_args: String,
    rootfs_read_only: bool,
    guest_vsock_cid: u32,
    guest_control_port: u16,
    vsock_path: PathBuf,
    api_socket_path: PathBuf,
    console_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CloudHypervisorRuntimeMetadata {
    machine_name: String,
    pid: u32,
    binary: PathBuf,
    config_path: PathBuf,
    metadata_path: PathBuf,
    vsock_path: PathBuf,
    guest_vsock_cid: u32,
    guest_control_port: u16,
    api_socket_path: PathBuf,
    console_log: PathBuf,
    launched_at_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AvfRuntimeMetadata {
    machine_name: String,
    pid: u32,
    launcher: PathBuf,
    config_path: PathBuf,
    metadata_path: PathBuf,
    guest_agent_socket: PathBuf,
    console_log: PathBuf,
    guest_transport: port_model::AvfGuestTransport,
    console_transport: port_model::AvfConsoleTransport,
    launched_at_unix_s: u64,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};
    use std::fs;
    use std::io::{BufRead, BufReader, Cursor, Read, Write};
    use std::net::{Shutdown, TcpListener as StdTcpListener, TcpStream};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::{Mutex, OnceLock, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    use axum::extract::{Path as AxumPath, State};
    use axum::http::StatusCode;
    use axum::routing::{get, post};
    use axum::{Json, Router};
    use serde::Serialize;
    use serde_json::json;
    use tempfile::tempdir;

    use super::{
        ArtifactAction, ArtifactAvailabilityState, ArtifactRequest, AvfRuntimeMetadata,
        CloudHypervisorRuntimeMetadata, ClusterDownRequest, ClusterReadinessState,
        ClusterStageRequest, ClusterStatusRequest, ClusterUpRequest, ControlPlaneServeRequest,
        DoctorCheck, DoctorHostFacts, GuestCopyRequest, GuestForwardRequest, GuestRequest,
        HostedNodeBinding, LaunchMetadata, LaunchRequest, MachineDriverKind, MachineRuntimeState,
        MachineStatus, NodeAgentServeRequest, RecoveryAttemptCounters, RecoveryState, RuntimePaths,
        ServiceApplyRequest, ServiceDefinitionRecord, ServiceDefinitionStatus, ServiceDesiredState,
        ServiceKind, ServicePolicy, ServiceRuntimeState, ServiceSecretBinding, StopResult,
        apply_machine_service, artifact_pipeline_workdir, artifact_script,
        avf_local_launch_machine_with_host_os, bootstrap_hosted_k3s_cluster,
        build_firecracker_config, cache_path_for, chown_recursive, chown_runtime_to_sudo_caller,
        cloud_hypervisor_api_socket_path, cloud_hypervisor_config_path,
        cloud_hypervisor_local_launch_machine, cloud_hypervisor_log_path, collect_doctor_report,
        collect_doctor_report_with_facts, copy_guest_file, delete_machine_secret,
        down_local_cluster, driver_for_machine, ensure_native_build_lane, execute_guest_operation,
        execute_hosted_k3s_managed_service_start_with_retry, hosted_k3s_api_readiness_command,
        hosted_k3s_cluster_access, hosted_k3s_cluster_kubeconfig, hosted_k3s_join_token_command,
        hosted_k3s_kubeconfig_command, hosted_k3s_machine_access, hosted_k3s_service_policy,
        hosted_k3s_visibility_command, hosted_machine_resolution, hosted_placeholder_runtime_root,
        k3s_bootstrap_command, launch_local_machine, list_artifacts, list_machine_secrets,
        list_machine_services, list_machines, local_cluster_kubeconfig, local_cluster_status,
        machine_monitor, machine_service_status, machine_status, machine_top, path_check,
        prepare_guest_forward, prepare_runtime_state, pull_artifact, push_artifact,
        put_machine_secret, read_json_file, read_pid_file, render_hosted_route_context, repo_root,
        resolve_artifact_metadata, resolve_artifact_script_path, resolve_artifact_store_contract,
        resolve_machine_architecture, select_firecracker_binary, serve_control_plane,
        serve_node_agent, service_definition_dir, service_runtime_dir, service_status_from_record,
        stage_local_cluster_bootstrap, stop_machine, stop_machine_service, sudo_caller_ids,
        up_local_cluster, uses_repo_managed_guest_image_pipeline,
    };
    use port_agent_protocol::{
        CopyDirection, ExecRequest, ExecResult, ForwardRequest, GuestOperation, LogsRequest,
        LogsResult, ManagedServiceKind, ManagedServiceOperation, ManagedServiceRequest,
        ManagedServiceResult, ManagedServiceRuntimeState, ManagedServiceStatus, OperationResult,
        PtyRequest, RequestEnvelope, ResponseEnvelope, StreamKind, StreamRequestFrame,
        StreamResponseFrame, read_frame, write_frame,
    };
    use port_guest_agent::serve as serve_guest_agent;
    use port_hosted_protocol::{
        HostedDetachedForwardStartRequest, HostedDetachedForwardState,
        HostedDetachedForwardStatus as HostedDetachedForwardStatusContract,
        HostedDetachedForwardStopResult, HostedError, HostedRouteContext, HostedSuccess,
    };
    use port_model::{
        ArtifactKind, ArtifactStore, ExecutionSubstrate, HostConnection, HostPlatform,
        HostProvider, HostedImportedNodeRecord, HostedNodeCapabilities, HostedNodeSpec,
        HostedSchedulerPolicy, MachineArchitecture, MachineVolumeBackend, MachineVolumePersistence,
        MachineVolumeSpec, OciRegistryAuth, OciRegistryTransport, PortConfig, ProtectionMode,
        PvmCapabilityState, ServiceHealthPolicy, ServiceHealthState, ServiceHealthcheck,
        ServiceRestartPolicy, ServiceSecretBackend, ServiceSecretMaterialization,
    };
    use tokio::net::TcpListener;

    fn sample_config_with_hosted_runtime_roots(root: &Path) -> PortConfig {
        let mut config = PortConfig::sample();
        config.clusters.clear();
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

    fn sample_multi_node_service_config(root: &Path) -> PortConfig {
        let mut config = sample_config_with_hosted_runtime_roots(root);
        let mut alternate = config
            .nodes
            .get("aws-linux-node")
            .expect("aws-linux-node should exist")
            .clone();
        alternate.runtime_root = root.join("hosted/aws-linux-node-b");
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
        config.host_groups.insert(
            String::from("aws-secondary"),
            port_model::HostedHostGroupSpec {
                placement: port_model::HostedPlacementPolicy::ExplicitMembership,
                scheduler: HostedSchedulerPolicy::DeterministicFirstFit,
                nodes: vec![String::from("aws-linux-node-b")],
                notes: vec![String::from(
                    "Secondary AWS builders group used for deterministic service placement tests.",
                )],
            },
        );
        config
    }

    fn sample_multi_node_machine_config(root: &Path) -> PortConfig {
        let mut config = sample_config_with_hosted_runtime_roots(root);
        let mut alternate = config
            .nodes
            .get("aws-linux-node")
            .expect("aws-linux-node should exist")
            .clone();
        alternate.runtime_root = root.join("hosted/aws-linux-node-b");
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

    fn configure_hosted_kernel_paths(
        config: &mut PortConfig,
        local_root: &Path,
        cache_root: &Path,
        endpoint: &str,
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
            let architecture = match variant.selector.architecture {
                MachineArchitecture::Native => "native",
                MachineArchitecture::X86_64 => "x86_64",
                MachineArchitecture::Aarch64 => "aarch64",
            };
            let protection_mode = match variant.selector.protection_mode {
                ProtectionMode::Standard => "standard",
                ProtectionMode::Pvm => "pvm",
            };
            variant.path = local_root
                .join(architecture)
                .join("firecracker")
                .join(protection_mode)
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
        let store_path = PathBuf::from(".port/hosted/demo/artifacts")
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

    fn install_fake_oras_script(root: &Path, body: &str) -> PathBuf {
        let script_path = root.join("oras");
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
    fn resolve_artifact_store_contract_returns_hosted_transfer_contract() {
        let mut config = PortConfig::sample();
        {
            let kernel = config
                .artifacts
                .kernels
                .get_mut("demo-kernel")
                .expect("sample kernel should exist");
            kernel.distribution.push = ArtifactStore::HostedApi {
                endpoint: String::from("https://port.example.internal"),
            };
        }

        let artifact = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("artifact should resolve");

        let push_store = config
            .artifacts
            .kernels
            .get("demo-kernel")
            .expect("sample kernel should exist")
            .distribution
            .push
            .clone();
        let contract = resolve_artifact_store_contract(&config, &push_store, &artifact)
            .expect("hosted artifact contract should resolve");

        match contract {
            super::ArtifactStoreContract::HostedApi { identity, transfer } => {
                assert_eq!(identity.control_plane, "demo");
                assert_eq!(identity.endpoint, "https://port.example.internal");
                assert_eq!(transfer.artifact_name, "demo-kernel");
                assert_eq!(
                    transfer.store_path,
                    PathBuf::from(
                        ".port/hosted/demo/artifacts/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux"
                    )
                );
            }
            other => panic!("expected hosted artifact contract, got {other:?}"),
        }
    }

    #[test]
    fn oci_registry_backend_requires_oras_binary_with_explicit_detail() {
        let _path = ScopedPathEnv::replace(Path::new("/definitely-missing-port-oci-path"));
        let mut config = PortConfig::sample();
        {
            let kernel = config
                .artifacts
                .kernels
                .get_mut("demo-kernel")
                .expect("sample kernel should exist");
            kernel.reference.registry = String::from("registry.port.test:5000");
            kernel.reference.repository = String::from("artifacts/demo-kernel");
            kernel.distribution.push = ArtifactStore::OciRegistry {
                transport: OciRegistryTransport::PlainHttp,
                auth: OciRegistryAuth::Anonymous,
            };
        }

        let artifact = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("artifact should resolve");

        let push_store = config
            .artifacts
            .kernels
            .get("demo-kernel")
            .expect("sample kernel should exist")
            .distribution
            .push
            .clone();
        let error = resolve_artifact_store_contract(&config, &push_store, &artifact)
            .expect_err("missing oras should fail fast");

        assert!(
            error.to_string().contains("oras"),
            "unexpected error: {error}"
        );
        assert!(
            error.to_string().contains(
                "registry.port.test:5000/artifacts/demo-kernel:v1-x86_64-firecracker-standard"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn oci_registry_backend_requires_basic_auth_environment_variables() {
        let tempdir = tempdir().expect("tempdir should exist");
        let _path = ScopedPathEnv::prepend(tempdir.path());
        fs::write(tempdir.path().join("oras"), "#!/usr/bin/env bash\nexit 0\n")
            .expect("fake oras should write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = tempdir.path().join("oras");
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }

        let mut config = PortConfig::sample();
        {
            let kernel = config
                .artifacts
                .kernels
                .get_mut("demo-kernel")
                .expect("sample kernel should exist");
            kernel.reference.registry = String::from("registry.port.test:5000");
            kernel.reference.repository = String::from("artifacts/demo-kernel");
            kernel.distribution.push = ArtifactStore::OciRegistry {
                transport: OciRegistryTransport::PlainHttp,
                auth: OciRegistryAuth::BasicEnv {
                    username_variable: String::from("PORT_OCI_USER"),
                    password_variable: String::from("PORT_OCI_PASSWORD"),
                },
            };
        }
        unsafe {
            std::env::remove_var("PORT_OCI_USER");
            std::env::remove_var("PORT_OCI_PASSWORD");
        }

        let artifact = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("artifact should resolve");
        let push_store = config
            .artifacts
            .kernels
            .get("demo-kernel")
            .expect("sample kernel should exist")
            .distribution
            .push
            .clone();
        let error = resolve_artifact_store_contract(&config, &push_store, &artifact)
            .expect_err("missing auth env should fail fast");

        assert!(
            error.to_string().contains("PORT_OCI_USER"),
            "unexpected error: {error}"
        );
        assert!(
            error.to_string().contains("basic-env"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn doctor_report_surfaces_oci_registry_backend_dependency_and_auth_checks() {
        let _path = ScopedPathEnv::replace(Path::new("/definitely-missing-port-oci-path"));
        let mut config = PortConfig::sample();
        {
            let kernel = config
                .artifacts
                .kernels
                .get_mut("demo-kernel")
                .expect("sample kernel should exist");
            kernel.distribution.push = ArtifactStore::OciRegistry {
                transport: OciRegistryTransport::PlainHttp,
                auth: OciRegistryAuth::BasicEnv {
                    username_variable: String::from("PORT_OCI_USER"),
                    password_variable: String::from("PORT_OCI_PASSWORD"),
                },
            };
        }
        unsafe {
            std::env::remove_var("PORT_OCI_USER");
            std::env::remove_var("PORT_OCI_PASSWORD");
        }

        let report = collect_doctor_report(Some(&config));
        let oras_check = report
            .checks
            .iter()
            .find(|check| check.name == "artifact-store:demo-kernel:push:oras")
            .expect("oras check should exist");
        let auth_check = report
            .checks
            .iter()
            .find(|check| check.name == "artifact-store:demo-kernel:push:auth")
            .expect("auth check should exist");

        assert!(!oras_check.ok, "unexpected check state: {oras_check:?}");
        assert!(
            oras_check.detail.contains("plain-http"),
            "unexpected detail: {}",
            oras_check.detail
        );
        assert!(!auth_check.ok, "unexpected check state: {auth_check:?}");
        assert!(
            auth_check.detail.contains("PORT_OCI_USER"),
            "unexpected detail: {}",
            auth_check.detail
        );
    }

    #[test]
    fn oci_registry_push_executes_oras_and_materializes_cache() {
        let tempdir = tempdir().expect("tempdir should exist");
        let args_log = tempdir.path().join("oras-args.log");
        install_fake_oras_script(
            tempdir.path(),
            r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "${PORT_TEST_ORAS_ARGS:?}"
"#,
        );
        let _path = ScopedPathEnv::prepend(tempdir.path());
        unsafe {
            std::env::set_var("PORT_TEST_ORAS_ARGS", &args_log);
        }

        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let mut config = PortConfig::sample();
        let kernel = config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        kernel.distribution.cache_root = cache_root.clone();
        kernel.distribution.push = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::Anonymous,
        };
        for variant in &mut kernel.variants {
            variant.path = local_root
                .join(match variant.selector.architecture {
                    MachineArchitecture::Native => "native",
                    MachineArchitecture::X86_64 => "x86_64",
                    MachineArchitecture::Aarch64 => "aarch64",
                })
                .join("firecracker")
                .join(match variant.selector.protection_mode {
                    ProtectionMode::Standard => "standard",
                    ProtectionMode::Pvm => "pvm",
                })
                .join("vmlinux");
        }

        let artifact_path = local_root
            .join("x86_64")
            .join("firecracker")
            .join("standard")
            .join("vmlinux");
        fs::create_dir_all(artifact_path.parent().expect("parent should exist"))
            .expect("artifact parent should exist");
        fs::write(&artifact_path, b"demo-oci-kernel-bytes").expect("artifact should write");

        let transfer = push_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("OCI push should succeed");

        assert_eq!(
            transfer.store_path,
            PathBuf::from("demo-fs/port/demo-kernel:v1-x86_64-firecracker-standard")
        );
        assert_eq!(transfer.bytes_copied, 21);
        assert!(
            transfer
                .backend_detail
                .contains("oci-registry plain-http anonymous"),
            "unexpected backend detail: {}",
            transfer.backend_detail
        );
        assert_eq!(
            fs::read(&transfer.artifact.cache_path).expect("cache path should exist"),
            b"demo-oci-kernel-bytes"
        );

        let args = fs::read_to_string(&args_log).expect("args log should exist");
        assert!(args.contains("push"), "unexpected args: {args}");
        assert!(args.contains("--plain-http"), "unexpected args: {args}");
        assert!(
            args.contains("demo-fs/port/demo-kernel:v1-x86_64-firecracker-standard"),
            "unexpected args: {args}"
        );
        assert!(
            args.contains("vmlinux:application/vnd.port.kernel.v1+binary"),
            "unexpected args: {args}"
        );
    }

    #[test]
    fn oci_registry_push_failure_surfaces_remote_and_path_context() {
        let tempdir = tempdir().expect("tempdir should exist");
        install_fake_oras_script(
            tempdir.path(),
            r#"#!/usr/bin/env bash
set -euo pipefail
echo "simulated oras push failure" >&2
exit 19
"#,
        );
        let _path = ScopedPathEnv::prepend(tempdir.path());

        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let mut config = PortConfig::sample();
        let kernel = config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        kernel.distribution.cache_root = cache_root.clone();
        kernel.distribution.push = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::Anonymous,
        };
        for variant in &mut kernel.variants {
            variant.path = local_root
                .join(match variant.selector.architecture {
                    MachineArchitecture::Native => "native",
                    MachineArchitecture::X86_64 => "x86_64",
                    MachineArchitecture::Aarch64 => "aarch64",
                })
                .join("firecracker")
                .join(match variant.selector.protection_mode {
                    ProtectionMode::Standard => "standard",
                    ProtectionMode::Pvm => "pvm",
                })
                .join("vmlinux");
        }

        let artifact_path = local_root
            .join("x86_64")
            .join("firecracker")
            .join("standard")
            .join("vmlinux");
        fs::create_dir_all(artifact_path.parent().expect("parent should exist"))
            .expect("artifact parent should exist");
        fs::write(&artifact_path, b"demo-oci-kernel-bytes").expect("artifact should write");

        let error = push_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect_err("OCI push should fail");

        let rendered = error.to_string();
        assert!(
            rendered.contains("demo-fs/port/demo-kernel:v1"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("x86_64/firecracker/standard"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("demo-fs/port/demo-kernel:v1-x86_64-firecracker-standard"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains(&artifact_path.display().to_string()),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("artifact-cache"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("simulated oras push failure"),
            "unexpected error: {rendered}"
        );
    }

    #[test]
    fn oci_registry_pull_fetches_into_cache_and_local_paths() {
        let tempdir = tempdir().expect("tempdir should exist");
        let args_log = tempdir.path().join("oras-args.log");
        install_fake_oras_script(
            tempdir.path(),
            r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "${PORT_TEST_ORAS_ARGS:?}"
output_dir=""
while (($#)); do
  case "$1" in
    --output)
      output_dir="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
mkdir -p "${output_dir:?}"
printf '%s' "demo-oci-kernel-bytes" > "${output_dir}/vmlinux"
"#,
        );
        let _path = ScopedPathEnv::prepend(tempdir.path());
        unsafe {
            std::env::set_var("PORT_TEST_ORAS_ARGS", &args_log);
        }

        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let mut config = PortConfig::sample();
        let kernel = config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        kernel.distribution.cache_root = cache_root.clone();
        kernel.distribution.pull = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::Anonymous,
        };
        for variant in &mut kernel.variants {
            variant.path = local_root
                .join(match variant.selector.architecture {
                    MachineArchitecture::Native => "native",
                    MachineArchitecture::X86_64 => "x86_64",
                    MachineArchitecture::Aarch64 => "aarch64",
                })
                .join("firecracker")
                .join(match variant.selector.protection_mode {
                    ProtectionMode::Standard => "standard",
                    ProtectionMode::Pvm => "pvm",
                })
                .join("vmlinux");
        }

        let transfer = pull_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("OCI pull should succeed");

        assert_eq!(
            transfer.store_path,
            PathBuf::from("demo-fs/port/demo-kernel:v1-x86_64-firecracker-standard")
        );
        assert_eq!(transfer.bytes_copied, 21);
        assert!(
            transfer
                .backend_detail
                .contains("oci-registry plain-http anonymous"),
            "unexpected backend detail: {}",
            transfer.backend_detail
        );
        assert_eq!(
            fs::read(&transfer.artifact.path).expect("local path should exist"),
            b"demo-oci-kernel-bytes"
        );
        assert_eq!(
            fs::read(&transfer.artifact.cache_path).expect("cache path should exist"),
            b"demo-oci-kernel-bytes"
        );

        let args = fs::read_to_string(&args_log).expect("args log should exist");
        assert!(args.contains("pull"), "unexpected args: {args}");
        assert!(args.contains("--plain-http"), "unexpected args: {args}");
        assert!(args.contains("--output"), "unexpected args: {args}");
        assert!(
            args.contains("demo-fs/port/demo-kernel:v1-x86_64-firecracker-standard"),
            "unexpected args: {args}"
        );
    }

    #[test]
    fn locate_oci_pulled_artifact_prefers_preserved_relative_path() {
        let tempdir = tempdir().expect("tempdir should exist");
        let scratch_dir = tempdir.path().join("scratch");
        let staged_path = scratch_dir
            .join("artifacts")
            .join("kernel")
            .join("demo")
            .join("x86_64")
            .join("firecracker")
            .join("standard")
            .join("vmlinux");
        fs::create_dir_all(staged_path.parent().expect("staged parent should exist"))
            .expect("staged directories should create");
        fs::write(&staged_path, b"demo-oci-kernel-bytes").expect("staged artifact should write");

        let resolved = crate::locate_oci_pulled_artifact(
            &scratch_dir,
            Path::new("artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux"),
        )
        .expect("nested staged artifact should resolve");

        assert_eq!(resolved, staged_path);
    }

    #[test]
    fn oci_registry_pull_failure_surfaces_remote_and_path_context() {
        let tempdir = tempdir().expect("tempdir should exist");
        install_fake_oras_script(
            tempdir.path(),
            r#"#!/usr/bin/env bash
set -euo pipefail
echo "simulated oras pull failure" >&2
exit 23
"#,
        );
        let _path = ScopedPathEnv::prepend(tempdir.path());

        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let mut config = PortConfig::sample();
        let kernel = config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        kernel.distribution.cache_root = cache_root.clone();
        kernel.distribution.pull = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::Anonymous,
        };
        for variant in &mut kernel.variants {
            variant.path = local_root
                .join(match variant.selector.architecture {
                    MachineArchitecture::Native => "native",
                    MachineArchitecture::X86_64 => "x86_64",
                    MachineArchitecture::Aarch64 => "aarch64",
                })
                .join("firecracker")
                .join(match variant.selector.protection_mode {
                    ProtectionMode::Standard => "standard",
                    ProtectionMode::Pvm => "pvm",
                })
                .join("vmlinux");
        }

        let error = pull_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect_err("OCI pull should fail");

        let rendered = error.to_string();
        assert!(
            rendered.contains("demo-fs/port/demo-kernel:v1"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("x86_64/firecracker/standard"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("plain-http"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("anonymous auth"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("artifact-cache"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("local-artifacts"),
            "unexpected error: {rendered}"
        );
        assert!(
            rendered.contains("simulated oras pull failure"),
            "unexpected error: {rendered}"
        );
    }

    #[test]
    fn oci_registry_cache_path_is_deterministic_across_pull_backends() {
        let cache_root = tempdir().expect("tempdir should exist");
        let mut filesystem = PortConfig::sample();
        let filesystem_kernel = filesystem
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        filesystem_kernel.distribution.cache_root = cache_root.path().to_path_buf();

        let mut oci = filesystem.clone();
        let oci_kernel = oci
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("sample kernel should exist");
        oci_kernel.distribution.pull = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::Anonymous,
        };

        let filesystem_artifact = resolve_artifact_metadata(
            &filesystem,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Pvm,
            },
        )
        .expect("filesystem metadata should resolve");
        let oci_artifact = resolve_artifact_metadata(
            &oci,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Pvm,
            },
        )
        .expect("OCI metadata should resolve");

        assert_eq!(oci_artifact.path, filesystem_artifact.path);
        assert_eq!(oci_artifact.cache_path, filesystem_artifact.cache_path);
    }

    #[test]
    fn list_artifacts_reports_local_and_cached_variant_presence() {
        let tempdir = tempdir().expect("tempdir should exist");
        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let kernel_path = local_root.join("kernel-x86_64-standard");
        let guest_path = local_root.join("guest-x86_64-standard");

        let mut config = PortConfig::sample();
        {
            let kernel = config
                .artifacts
                .kernels
                .get_mut("demo-kernel")
                .expect("sample kernel should exist");
            kernel.distribution.cache_root = cache_root.clone();
            kernel
                .variants
                .iter_mut()
                .find(|variant| {
                    variant.selector.architecture == MachineArchitecture::X86_64
                        && variant.selector.substrate == ExecutionSubstrate::Firecracker
                        && variant.selector.protection_mode == ProtectionMode::Standard
                })
                .expect("standard kernel variant should exist")
                .path = kernel_path.clone();
        }
        {
            let guest = config
                .artifacts
                .guest_images
                .get_mut("demo-guest")
                .expect("sample guest image should exist");
            guest.distribution.cache_root = cache_root.clone();
            guest
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

        fs::create_dir_all(kernel_path.parent().expect("kernel parent should exist"))
            .expect("kernel parent should exist");
        fs::write(&kernel_path, b"demo-kernel-bytes").expect("kernel artifact should write");

        let guest_cache_path = {
            let guest = config
                .artifacts
                .guest_images
                .get("demo-guest")
                .expect("sample guest image should exist");
            let variant = guest
                .variant(
                    MachineArchitecture::X86_64,
                    ExecutionSubstrate::Firecracker,
                    ProtectionMode::Standard,
                )
                .expect("standard guest variant should exist");
            cache_path_for(guest, variant)
        };
        fs::create_dir_all(
            guest_cache_path
                .parent()
                .expect("guest cache parent should exist"),
        )
        .expect("guest cache parent should exist");
        fs::write(&guest_cache_path, b"demo-guest-cache").expect("guest cache should write");

        let inventory = list_artifacts(&config);
        let kernel = inventory
            .iter()
            .find(|record| record.name == "demo-kernel")
            .expect("kernel record should exist");
        let kernel_variant = kernel
            .variants
            .iter()
            .find(|variant| {
                variant.selector.architecture == MachineArchitecture::X86_64
                    && variant.selector.substrate == ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == ProtectionMode::Standard
            })
            .expect("kernel standard variant should exist");
        assert!(kernel_variant.local_present);
        assert!(!kernel_variant.cache_present);
        assert_eq!(
            kernel_variant.availability,
            ArtifactAvailabilityState::Local
        );

        let guest = inventory
            .iter()
            .find(|record| record.name == "demo-guest")
            .expect("guest record should exist");
        let guest_variant = guest
            .variants
            .iter()
            .find(|variant| {
                variant.selector.architecture == MachineArchitecture::X86_64
                    && variant.selector.substrate == ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == ProtectionMode::Standard
            })
            .expect("guest standard variant should exist");
        assert!(!guest_variant.local_present);
        assert!(guest_variant.cache_present);
        assert_eq!(
            guest_variant.availability,
            ArtifactAvailabilityState::CacheOnly
        );
        assert_eq!(guest_variant.cache_path, guest_cache_path);
    }

    #[test]
    fn push_and_pull_artifact_round_trip_through_live_hosted_backend() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        let _ = fs::remove_dir_all(".port/hosted/demo");

        let tempdir = tempdir().expect("tempdir should exist");
        let local_root = tempdir.path().join("local-artifacts");
        let cache_root = tempdir.path().join("artifact-cache");
        let mut config = PortConfig::sample();
        let control_plane_addr =
            start_live_control_plane(&config, None).expect("control plane should start");
        let endpoint = format!("http://{control_plane_addr}");
        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = endpoint.clone();
        let (local_path, cache_path, store_path) =
            configure_hosted_kernel_paths(&mut config, &local_root, &cache_root, &endpoint);

        fs::create_dir_all(local_path.parent().expect("local parent"))
            .expect("local parent should exist");
        fs::write(&local_path, "demo-kernel-hosted-bytes").expect("local artifact should write");

        let push = push_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("push should route through hosted backend");
        assert_eq!(push.store_path, store_path);
        assert_eq!(
            fs::read_to_string(&store_path).expect("store path should exist"),
            "demo-kernel-hosted-bytes"
        );
        assert_eq!(
            fs::read_to_string(&cache_path).expect("cache path should exist"),
            "demo-kernel-hosted-bytes"
        );
        assert!(
            push.backend_detail.contains("hosted-api"),
            "{}",
            push.backend_detail
        );

        fs::remove_file(&local_path).expect("local artifact should be removable");
        fs::remove_file(&cache_path).expect("cache artifact should be removable");

        let pull = pull_artifact(
            &config,
            ArtifactRequest {
                name: "demo-kernel",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
        )
        .expect("pull should route through hosted backend");
        assert_eq!(pull.store_path, store_path);
        assert_eq!(
            fs::read_to_string(&local_path).expect("local path should be restored"),
            "demo-kernel-hosted-bytes"
        );
        assert_eq!(
            fs::read_to_string(&cache_path).expect("cache path should be restored"),
            "demo-kernel-hosted-bytes"
        );
        assert!(
            pull.backend_detail.contains("hosted-api"),
            "{}",
            pull.backend_detail
        );

        let _ = fs::remove_dir_all(".port/hosted/demo");
    }

    fn write_machine_placement_state(
        control_plane: &str,
        machine_name: &str,
        node_name: &str,
        runtime_root: &Path,
        placement_detail: &str,
    ) {
        let state_path =
            hosted_placeholder_runtime_root(control_plane).join("machine-placements.json");
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

    #[derive(Debug, Serialize)]
    struct ImportedInventoryStateFile {
        control_plane: String,
        nodes: BTreeMap<String, port_model::HostedImportedNodeRecord>,
    }

    #[derive(Debug, Serialize)]
    struct RegisteredNodeStateFile {
        control_plane: String,
        nodes: BTreeMap<String, port_model::HostedNodeRegistration>,
    }

    fn write_imported_inventory_state_at(
        root: &Path,
        control_plane: &str,
        nodes: BTreeMap<String, port_model::HostedImportedNodeRecord>,
    ) {
        let state_path = root
            .join(".port/hosted")
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
            serde_json::to_vec_pretty(&ImportedInventoryStateFile {
                control_plane: control_plane.to_string(),
                nodes,
            })
            .expect("imported inventory state should encode"),
        )
        .expect("imported inventory state should write");
    }

    fn write_imported_inventory_state(
        control_plane: &str,
        nodes: BTreeMap<String, port_model::HostedImportedNodeRecord>,
    ) {
        write_imported_inventory_state_at(Path::new("."), control_plane, nodes);
    }

    fn write_registered_node_state(
        control_plane: &str,
        nodes: BTreeMap<String, port_model::HostedNodeRegistration>,
    ) {
        let state_path =
            hosted_placeholder_runtime_root(control_plane).join("registered-nodes.json");
        fs::create_dir_all(
            state_path
                .parent()
                .expect("registered node state path should have parent"),
        )
        .expect("registered node state dir should exist");
        fs::write(
            &state_path,
            serde_json::to_vec_pretty(&RegisteredNodeStateFile {
                control_plane: control_plane.to_string(),
                nodes,
            })
            .expect("registered node state should encode"),
        )
        .expect("registered node state should write");
    }

    fn sample_avf_config() -> PortConfig {
        let mut config = PortConfig::sample();
        config.clusters.clear();
        let machine = config
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.host = String::from("mac-local");
        machine.substrate = ExecutionSubstrate::Avf;
        machine.architecture = MachineArchitecture::X86_64;
        machine.protection_mode = ProtectionMode::Standard;

        config
    }

    fn sample_hosted_k3s_config(root: &Path) -> PortConfig {
        let mut config = sample_config_with_hosted_runtime_roots(root);
        config.set_state_root(root);
        let mut worker = config
            .machines
            .get("cloud-aws")
            .expect("cloud-aws should exist")
            .clone();
        worker.guest.vsock_cid = 63;
        worker.guest.control_port = 7002;
        worker.guest.console_log = PathBuf::from("runtime/cloud-aws-worker/console.log");
        config
            .machines
            .insert(String::from("cloud-aws-worker"), worker);
        config.k3s_clusters.insert(
            String::from("demo"),
            port_model::K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("aws-builders"),
                server_machines: vec![String::from("cloud-aws")],
                worker_machines: vec![String::from("cloud-aws-worker")],
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: port_model::HostedSchedulerPolicy::DeterministicFirstFit,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: vec![String::from("--node-label=role=worker")],
            },
        );
        config
    }

    #[test]
    fn hosted_k3s_effective_args_use_guest_underlay_ip_for_external_identity() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        config
            .machines
            .get_mut("cloud-aws-worker")
            .expect("worker should exist")
            .network = Some(port_model::MachineNetworkSpec {
            guest_ip: String::from("172.16.23.2"),
            host_ip: String::from("172.16.23.1"),
            prefix_len: 24,
            ..port_model::MachineNetworkSpec::default()
        });
        let args = super::hosted_k3s_effective_args(
            &config,
            "agent",
            "cloud-aws-worker",
            &[String::from("--node-label=role=worker")],
        )
        .expect("effective args should resolve");
        let worker_guest_ip = config
            .machines
            .get("cloud-aws-worker")
            .and_then(|machine| machine.network.as_ref())
            .map(|network| network.guest_ip.clone())
            .expect("worker guest IP should exist");
        assert!(
            args.windows(2).any(|window| {
                window[0] == "--node-external-ip" && window[1] == worker_guest_ip
            }),
            "{args:?}"
        );
        assert!(
            !args.iter().any(|arg| arg == "--flannel-external-ip"),
            "{args:?}"
        );
    }

    #[test]
    fn hosted_k3s_machine_external_ip_falls_back_to_imported_node_provenance_when_guest_network_is_missing()
     {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        config
            .machines
            .get_mut("cloud-aws-worker")
            .expect("worker should exist")
            .network = None;
        let state_root = super::hosted_placeholder_runtime_root_for_config(&config, "demo");
        fs::create_dir_all(&state_root).expect("state root should exist");
        super::persist_local_hosted_machine_placement_from_route(
            &config,
            "cloud-aws-worker",
            &super::HostedRouteContext {
                control_plane: Some(String::from("demo")),
                node_name: Some(String::from("aws-linux-node")),
                runtime_root: Some(PathBuf::from("/tmp/aws-linux-node")),
                placement_detail: Some(String::from("placed on aws-linux-node")),
                ..super::HostedRouteContext::default()
            },
            1,
        )
        .expect("placement state should sync");
        fs::write(
            state_root.join("imported-inventory.json"),
            serde_json::to_vec_pretty(&ImportedInventoryStateFile {
                control_plane: String::from("demo"),
                nodes: BTreeMap::from([(
                    String::from("aws-linux-node"),
                    port_model::HostedImportedNodeRecord {
                        provider: port_model::HostProvider::Aws,
                        provenance: String::from("10.0.1.24"),
                        imported_at: 1,
                        capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                        pvm_host_kit_packages: Vec::new(),
                    },
                )]),
            })
            .expect("imported inventory state should encode"),
        )
        .expect("imported inventory state should write");

        let imported_ip =
            super::hosted_imported_node_external_ip(&config, "demo", "aws-linux-node")
                .expect("imported node ip should resolve");
        assert_eq!(
            imported_ip,
            Some("10.0.1.24".parse().expect("ip should parse"))
        );
        let placement = super::hosted_stored_machine_placement(&config, "cloud-aws-worker")
            .expect("placement lookup should succeed");
        assert!(placement.is_some(), "stored placement should exist");
        let machine_ip = super::hosted_k3s_machine_external_ip(&config, "cloud-aws-worker")
            .expect("machine ip lookup should succeed");
        assert_eq!(
            machine_ip,
            Some("10.0.1.24".parse().expect("ip should parse"))
        );
    }

    fn launch_sample_avf_machine(
        runtime_root: &Path,
    ) -> (PortConfig, RuntimePaths, LaunchMetadata) {
        let mut config = sample_avf_config();
        let launcher = write_fake_avf_launcher_binary(runtime_root, "port-avf-launcher");
        let kernel_path = runtime_root.join("avf-vmlinux");
        let guest_path = runtime_root.join("avf-rootfs.ext4");
        fs::write(&kernel_path, b"fake-avf-kernel").expect("kernel variant should write");
        fs::write(&guest_path, b"fake-avf-rootfs").expect("guest variant should write");
        config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("demo-kernel should exist")
            .variants
            .iter_mut()
            .find(|variant| {
                variant.selector.architecture == MachineArchitecture::X86_64
                    && variant.selector.substrate == ExecutionSubstrate::Avf
                    && variant.selector.protection_mode == ProtectionMode::Standard
            })
            .expect("avf kernel variant should exist")
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
                    && variant.selector.substrate == ExecutionSubstrate::Avf
                    && variant.selector.protection_mode == ProtectionMode::Standard
            })
            .expect("avf guest variant should exist")
            .path = guest_path;

        let metadata = avf_local_launch_machine_with_host_os(
            &config,
            &LaunchRequest {
                machine_name: "demo",
                runtime_root,
                boot_wait: Duration::from_millis(250),
            },
            "macos",
            Some(launcher),
        )
        .expect("avf launch should succeed");
        let paths = RuntimePaths::for_machine(runtime_root, "demo");
        (config, paths, metadata)
    }

    fn hosted_server_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn current_dir_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl CurrentDirGuard {
        fn change(path: &Path) -> Self {
            let original = std::env::current_dir().expect("current dir should resolve");
            std::env::set_current_dir(path).expect("current dir should change");
            Self { original }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).expect("current dir should restore");
        }
    }

    fn with_current_dir<T>(path: &Path, f: impl FnOnce() -> T) -> T {
        let _lock = current_dir_lock().lock().expect("cwd lock should work");
        let _guard = CurrentDirGuard::change(path);
        f()
    }

    fn reserve_addr() -> String {
        let listener = StdTcpListener::bind("127.0.0.1:0").expect("port should bind");
        let addr = listener.local_addr().expect("addr should exist");
        drop(listener);
        addr.to_string()
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

    fn write_log_asserting_firecracker_binary(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        fs::write(
            &path,
            r#"#!/usr/bin/env bash
set -euo pipefail
log_path=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --log-path)
      log_path="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
test -n "$log_path"
test -f "$log_path"
sleep 30
"#,
        )
        .expect("log-asserting firecracker should write");
        let mut permissions = fs::metadata(&path)
            .expect("log-asserting firecracker metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .expect("log-asserting firecracker permissions should update");
        path
    }

    fn write_fake_network_binaries(root: &Path) {
        for (name, version_output) in [
            ("ip", "ip utility, iproute2-6.12.0"),
            ("iptables", "iptables v1.8.11"),
        ] {
            let path = root.join(name);
            fs::write(
                &path,
                format!(
                    "#!/usr/bin/env bash\nif [[ \"${{1:-}}\" == \"-V\" || \"${{1:-}}\" == \"--version\" ]]; then\n  echo '{version_output}'\n  exit 0\nfi\nexit 0\n"
                ),
            )
            .expect("fake network tool should write");
            let mut permissions = fs::metadata(&path)
                .expect("fake network tool metadata should exist")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions)
                .expect("fake network tool permissions should update");
        }
    }

    fn write_fake_ip_binary(root: &Path, script_name: &str, script: &str) -> PathBuf {
        let path = root.join(script_name);
        fs::write(&path, script).expect("fake ip binary should write");
        let mut permissions = fs::metadata(&path)
            .expect("fake ip binary metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("fake ip binary permissions should update");
        path
    }

    fn serve_vsock_guest_agent_proxy(vsock_path: &Path, backend_socket: &Path) {
        let listener = UnixListener::bind(vsock_path).expect("vsock proxy should bind");
        for stream in listener.incoming() {
            let mut frontend = stream.expect("vsock proxy should accept");
            let reader_stream = frontend
                .try_clone()
                .expect("vsock proxy frontend should clone");
            let mut handshake_reader = BufReader::new(reader_stream);
            let mut handshake = String::new();
            handshake_reader
                .read_line(&mut handshake)
                .expect("vsock proxy handshake should read");
            assert_eq!(handshake, "CONNECT 7000\n");
            frontend
                .write_all(b"OK\n")
                .expect("vsock proxy should acknowledge handshake");
            frontend
                .flush()
                .expect("vsock proxy handshake should flush");

            let mut frontend_reader = handshake_reader.into_inner();
            let mut frontend_writer = frontend;
            let mut backend_reader = std::os::unix::net::UnixStream::connect(backend_socket)
                .expect("vsock proxy backend should connect");
            let mut backend_writer = backend_reader
                .try_clone()
                .expect("vsock proxy backend should clone");

            let frontend_to_backend = thread::spawn(move || {
                let _ = std::io::copy(&mut frontend_reader, &mut backend_writer);
                let _ = backend_writer.shutdown(Shutdown::Write);
            });
            let backend_to_frontend = thread::spawn(move || {
                let _ = std::io::copy(&mut backend_reader, &mut frontend_writer);
                let _ = frontend_writer.shutdown(Shutdown::Write);
            });

            frontend_to_backend
                .join()
                .expect("vsock proxy upload thread should complete");
            backend_to_frontend
                .join()
                .expect("vsock proxy download thread should complete");
        }
    }

    fn write_fake_standard_firecracker_artifacts(config: &mut PortConfig, root: &Path) {
        let kernel_path = root.join("standard-vmlinux");
        let guest_path = root.join("standard-rootfs.ext4");
        fs::write(&kernel_path, b"fake-standard-kernel")
            .expect("standard kernel variant should write");
        fs::write(&guest_path, b"fake-standard-rootfs")
            .expect("standard guest variant should write");

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

    fn write_fake_cloud_hypervisor_artifacts(config: &mut PortConfig, root: &Path) {
        let kernel_path = root.join("cloud-hypervisor-vmlinux");
        let guest_path = root.join("cloud-hypervisor-rootfs.ext4");
        fs::write(&kernel_path, b"fake-cloud-hypervisor-kernel")
            .expect("cloud-hypervisor kernel variant should write");
        fs::write(&guest_path, b"fake-cloud-hypervisor-rootfs")
            .expect("cloud-hypervisor guest variant should write");

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
            .expect("cloud-hypervisor kernel variant should exist")
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
            .expect("cloud-hypervisor guest variant should exist")
            .path = guest_path;
    }

    struct ScopedPathEnv {
        original: Option<std::ffi::OsString>,
    }

    impl ScopedPathEnv {
        fn prepend(path: &Path) -> Self {
            let original = std::env::var_os("PATH");
            let mut entries = vec![path.to_path_buf()];
            if let Some(existing) = &original {
                entries.extend(std::env::split_paths(existing));
            }
            let joined = std::env::join_paths(entries).expect("PATH should join");
            unsafe {
                std::env::set_var("PATH", joined);
            }
            Self { original }
        }

        fn from_paths<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Self {
            let original = std::env::var_os("PATH");
            let joined = std::env::join_paths(paths.into_iter().map(Path::to_path_buf))
                .expect("PATH should join");
            unsafe {
                std::env::set_var("PATH", joined);
            }
            Self { original }
        }

        fn replace(path: &Path) -> Self {
            let original = std::env::var_os("PATH");
            unsafe {
                std::env::set_var("PATH", path);
            }
            Self { original }
        }
    }

    impl Drop for ScopedPathEnv {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe {
                    std::env::set_var("PATH", value);
                },
                None => unsafe {
                    std::env::remove_var("PATH");
                },
            }
        }
    }

    struct ScopedEnvVar {
        name: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl ScopedEnvVar {
        fn set(name: &'static str, value: &Path) -> Self {
            let original = std::env::var_os(name);
            unsafe {
                std::env::set_var(name, value);
            }
            Self { name, original }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe {
                    std::env::set_var(self.name, value);
                },
                None => unsafe {
                    std::env::remove_var(self.name);
                },
            }
        }
    }

    fn write_fake_cluster_bootstrap_assets(root: &Path) {
        let bootstrap_root = root.join("examples/bootstrap/demo-k3s");
        fs::create_dir_all(&bootstrap_root).expect("bootstrap root should exist");
        fs::write(
            bootstrap_root.join("install-k3s-offline.sh"),
            r#"#!/bin/sh
set -eu

role="${1:-server}"
if [ "$#" -gt 0 ]; then
  shift
fi

stage_root=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
binary="${stage_root}/k3s"
target_dir="${PORT_K3S_BIN_DIR:-${stage_root}/bin}"
kubeconfig_path="${PORT_K3S_KUBECONFIG_PATH:-etc/rancher/k3s/k3s.yaml}"
server_log_path="${PORT_K3S_LOG_PATH:-var/log/port-k3s.log}"
server_pid_path="${target_dir}/k3s-server.pid"
server_ready_path="${target_dir}/k3s-server.ready"
node_name="${PORT_K3S_NODE_NAME:-demo}"

install -d \
  "${target_dir}" \
  "$(dirname "${kubeconfig_path}")" \
  "$(dirname "${server_log_path}")" \
  etc/rancher/k3s \
  opt/cni/bin \
  run \
  sys/fs/cgroup \
  var/lib/cni \
  var/lib/kubelet \
  var/lib/rancher/k3s \
  var/log
install -m 0755 "${binary}" "${target_dir}/k3s"
ln -sf "k3s" "${target_dir}/kubectl"
"${target_dir}/k3s" "${role}" --write-kubeconfig "${kubeconfig_path}" >/dev/null
printf 'k3s-server:%s pid-file=%s log=%s\n' \
  'started' "${server_pid_path}" "${server_log_path}"
printf 'offline-install-ok role=%s args=%s bin-dir=%s kubeconfig=%s\n' \
  "${role}" "$*" "${target_dir}" "${kubeconfig_path}"
"#,
        )
        .expect("fake install script should write");
        fs::write(
            bootstrap_root.join("k3s"),
            r#"#!/bin/sh
set -eu

exec usr/bin/k3s "$@"
"#,
        )
        .expect("fake k3s binary should write");
        for path in [
            bootstrap_root.join("install-k3s-offline.sh"),
            bootstrap_root.join("k3s"),
        ] {
            let mut permissions = fs::metadata(&path)
                .expect("bootstrap asset metadata should exist")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).expect("bootstrap asset should be executable");
        }
    }

    fn write_fake_guest_k3s_runtime(guest_root: &Path) {
        let usr_bin = guest_root.join("usr/bin");
        let state_root = guest_root.join("var/lib/rancher/k3s");
        fs::create_dir_all(&usr_bin).expect("guest usr/bin should exist");
        fs::create_dir_all(&state_root).expect("guest state root should exist");
        let k3s = usr_bin.join("k3s");
        fs::write(
            &k3s,
            r#"#!/bin/sh
set -eu

state_root="var/lib/rancher/k3s"
state_file="${state_root}/server.started"
node_name="demo"
version="v1.35.2+k3s1"
write_kubeconfig="etc/rancher/k3s/k3s.yaml"

if [ "$#" -gt 0 ] && [ "$1" = "server" ]; then
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --write-kubeconfig)
        shift
        write_kubeconfig="$1"
        ;;
      --node-name)
        shift
        node_name="$1"
        ;;
    esac
    shift
  done
  mkdir -p "$(dirname "${write_kubeconfig}")" "${state_root}"
  cat >"${write_kubeconfig}" <<EOF
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://127.0.0.1:6443
  name: demo
contexts:
- context:
    cluster: demo
    user: demo
  name: demo
current-context: demo
users:
- name: demo
  user:
    token: demo-token
EOF
  printf '%s\n' "${node_name}" >"${state_file}"
  exit 0
fi

if [ "$#" -ge 4 ] && [ "$1" = "kubectl" ] && [ "$2" = "get" ] && [ "$3" = "nodes" ]; then
  if [ ! -f "${state_file}" ]; then
    echo "control plane not ready" >&2
    exit 1
  fi
  read -r stored_name <"${state_file}"
  cat <<EOF
NAME          STATUS   ROLES                  AGE   VERSION
${stored_name}   Ready    control-plane,master   1m    ${version}
EOF
  exit 0
fi

if [ "$#" -ge 4 ] && [ "$1" = "kubectl" ] && [ "$2" = "api-resources" ] && [ "$3" = "-o" ] && [ "$4" = "name" ]; then
  cat <<EOF
configmaps
namespaces
secrets
serviceaccounts
deployments.apps
customresourcedefinitions.apiextensions.k8s.io
EOF
  exit 0
fi

if [ "$#" -gt 0 ] && [ "$1" = "--version" ]; then
  printf 'k3s version %s (fake)\n' "${version}"
  exit 0
fi

printf 'fake-k3s %s\n' "$*"
"#,
        )
        .expect("fake guest k3s should write");
        let mut permissions = fs::metadata(&k3s)
            .expect("fake guest k3s metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&k3s, permissions).expect("fake guest k3s should become executable");

        #[cfg(unix)]
        std::os::unix::fs::symlink("k3s", usr_bin.join("kubectl"))
            .expect("fake guest kubectl symlink should exist");
    }

    fn write_fake_avf_launcher_binary(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        fs::write(
            &path,
            r#"#!/usr/bin/env bash
set -euo pipefail
console_log=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --console-log)
      console_log="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [[ -n "$console_log" ]]; then
  printf 'avf-launcher booted\n' >>"$console_log"
fi
exec sleep 30
"#,
        )
        .expect("fake avf launcher should write");
        let mut permissions = fs::metadata(&path)
            .expect("fake avf launcher metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .expect("fake avf launcher permissions should update");
        path
    }

    fn wait_for_http_or_server_error(
        url: &str,
        headers: &[(&str, &str)],
        expected_status: Option<u16>,
        error_rx: &mpsc::Receiver<anyhow::Result<()>>,
        name: &str,
    ) -> anyhow::Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(200))
            .build()
            .expect("blocking client should build");
        for _ in 0..500 {
            let mut request = client.get(url);
            for (header, value) in headers {
                request = request.header(*header, *value);
            }
            if let Ok(response) = request.send() {
                if expected_status.is_none_or(|status| response.status().as_u16() == status) {
                    return Ok(());
                }
            }
            if let Ok(result) = error_rx.try_recv() {
                return match result {
                    Ok(()) => Err(anyhow::anyhow!(
                        "{name} exited before becoming ready at '{url}'"
                    )),
                    Err(error) => Err(error),
                };
            }
            thread::sleep(Duration::from_millis(20));
        }

        Err(anyhow::anyhow!(
            "timed out waiting for {name} listener at '{url}'{}",
            expected_status
                .map(|status| format!(" expecting status {status}"))
                .unwrap_or_default()
        ))
    }

    #[cfg(target_os = "linux")]
    fn wait_for_process_state(pid: u32, expected: char) {
        for _ in 0..500 {
            if matches!(super::process_state_code(pid), Ok(Some(state)) if state == expected) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("process {pid} did not reach state {expected} in time");
    }

    fn start_live_hosted_servers_inner(
        config: &PortConfig,
        bind_node: bool,
    ) -> anyhow::Result<PortConfig> {
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        let _ = std::fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));

        let mut client_config = config.clone();
        let control_plane_addr = start_live_control_plane(&client_config, None)?;
        client_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{control_plane_addr}");
        if bind_node {
            start_live_node_agent(&client_config)?;
        }

        Ok(client_config)
    }

    fn start_live_hosted_servers(
        config: &PortConfig,
        bind_node: bool,
    ) -> anyhow::Result<PortConfig> {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        start_live_hosted_servers_inner(config, bind_node)
    }

    fn start_named_live_hosted_servers_inner(
        config: &PortConfig,
        node_names: &[&str],
    ) -> anyhow::Result<PortConfig> {
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        let _ = std::fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));

        let mut client_config = config.clone();
        let control_plane_addr =
            start_live_control_plane_with_bindings(&client_config, Vec::new())?;
        client_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{control_plane_addr}");
        for node_name in node_names {
            start_live_named_node_agent(&client_config, node_name)?;
        }

        Ok(client_config)
    }

    fn start_named_live_hosted_servers(
        config: &PortConfig,
        node_names: &[&str],
    ) -> anyhow::Result<PortConfig> {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        start_named_live_hosted_servers_inner(config, node_names)
    }

    fn start_live_node_agent(config: &PortConfig) -> anyhow::Result<String> {
        for _ in 0..10 {
            let node_addr = reserve_addr();
            let node_config = config.clone();
            let bind = node_addr.clone();
            let (node_tx, node_rx) = mpsc::channel();
            thread::spawn(move || {
                let result = serve_node_agent(
                    node_config,
                    NodeAgentServeRequest {
                        node_name: String::from("aws-linux-node"),
                        bind,
                        token: String::from("node-secret"),
                    },
                )
                .map(|_| ());
                let _ = node_tx.send(result);
            });
            let url = format!("http://{node_addr}/v1/node/machines/cloud-aws");
            match wait_for_http_or_server_error(
                &url,
                &[("x-port-node-agent-token", "node-secret")],
                None,
                &node_rx,
                "node-agent",
            ) {
                Ok(()) => return Ok(node_addr),
                Err(error) if error.to_string().contains("failed to bind") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(anyhow::anyhow!(
            "failed to bind a node-agent test server after repeated attempts"
        ))
    }

    fn start_live_named_node_agent(config: &PortConfig, node_name: &str) -> anyhow::Result<String> {
        for _ in 0..10 {
            let node_addr = reserve_addr();
            let node_config = config.clone();
            let bind = node_addr.clone();
            let node_name = node_name.to_string();
            let probe_name = node_name.clone();
            let (node_tx, node_rx) = mpsc::channel();
            thread::spawn(move || {
                let result = serve_node_agent(
                    node_config,
                    NodeAgentServeRequest {
                        node_name,
                        bind,
                        token: String::from("node-secret"),
                    },
                )
                .map(|_| ());
                let _ = node_tx.send(result);
            });
            let url = format!("http://{node_addr}/v1/node/machines/cloud-aws");
            match wait_for_http_or_server_error(
                &url,
                &[("x-port-node-agent-token", "node-secret")],
                None,
                &node_rx,
                &format!("node-agent-{probe_name}"),
            ) {
                Ok(()) => return Ok(node_addr),
                Err(error) if error.to_string().contains("failed to bind") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(anyhow::anyhow!(
            "failed to bind a node-agent test server for '{node_name}' after repeated attempts"
        ))
    }

    fn start_live_control_plane(
        config: &PortConfig,
        node_addr: Option<&str>,
    ) -> anyhow::Result<String> {
        for _ in 0..10 {
            let control_plane_addr = reserve_addr();
            let mut control_config = config.clone();
            control_config
                .control_planes
                .get_mut("demo")
                .expect("demo control plane should exist")
                .endpoint = format!("http://{control_plane_addr}");
            let bind = control_plane_addr.clone();
            let node_bindings = if let Some(node_addr) = node_addr {
                vec![HostedNodeBinding {
                    node_name: String::from("aws-linux-node"),
                    endpoint: format!("http://{node_addr}"),
                    token: String::from("node-secret"),
                }]
            } else {
                Vec::new()
            };
            let (control_tx, control_rx) = mpsc::channel();
            thread::spawn(move || {
                let result = serve_control_plane(
                    control_config,
                    ControlPlaneServeRequest {
                        control_plane: String::from("demo"),
                        bind,
                        node_bindings,
                    },
                )
                .map(|_| ());
                let _ = control_tx.send(result);
            });
            let url = format!("http://{control_plane_addr}/v1/machines");
            match wait_for_http_or_server_error(
                &url,
                &[("authorization", "Bearer demo-token")],
                Some(200),
                &control_rx,
                "control plane",
            ) {
                Ok(()) => return Ok(control_plane_addr),
                Err(error) if error.to_string().contains("failed to bind") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(anyhow::anyhow!(
            "failed to bind a control-plane test server after repeated attempts"
        ))
    }

    fn start_live_control_plane_with_bindings(
        config: &PortConfig,
        node_bindings: Vec<HostedNodeBinding>,
    ) -> anyhow::Result<String> {
        for _ in 0..10 {
            let control_plane_addr = reserve_addr();
            let mut control_config = config.clone();
            control_config
                .control_planes
                .get_mut("demo")
                .expect("demo control plane should exist")
                .endpoint = format!("http://{control_plane_addr}");
            let bind = control_plane_addr.clone();
            let node_bindings = node_bindings.clone();
            let (control_tx, control_rx) = mpsc::channel();
            thread::spawn(move || {
                let result = serve_control_plane(
                    control_config,
                    ControlPlaneServeRequest {
                        control_plane: String::from("demo"),
                        bind,
                        node_bindings,
                    },
                )
                .map(|_| ());
                let _ = control_tx.send(result);
            });
            let url = format!("http://{control_plane_addr}/v1/machines");
            match wait_for_http_or_server_error(
                &url,
                &[("authorization", "Bearer demo-token")],
                Some(200),
                &control_rx,
                "control plane",
            ) {
                Ok(()) => return Ok(control_plane_addr),
                Err(error) if error.to_string().contains("failed to bind") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(anyhow::anyhow!(
            "failed to bind a control-plane test server after repeated attempts"
        ))
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
            runtime_class: None,
            attached_volumes: Vec::new(),
        };
        fs::write(
            &paths.manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");
    }

    #[derive(Debug)]
    enum HostedGuestExpectedOperation {
        Exec {
            command: Vec<String>,
            stdout: String,
        },
        ExecFailure {
            command: Vec<String>,
            stderr: String,
            exit_code: i32,
        },
        ManagedServiceStart {
            name: String,
            command: Vec<String>,
            policy: ServicePolicy,
        },
    }

    fn running_managed_service_status(name: &str) -> ManagedServiceStatus {
        ManagedServiceStatus {
            name: name.to_string(),
            kind: ManagedServiceKind::Service,
            state: ManagedServiceRuntimeState::Running,
            restart_count: 0,
            pid: Some(4242),
            exit_code: None,
            last_exit_code: None,
            last_exit_detail: None,
            health_state: ServiceHealthState::Unknown,
            health_detail: None,
            stdout_path: Some(format!("/run/port/services/{name}.stdout.log")),
            stderr_path: Some(format!("/run/port/services/{name}.stderr.log")),
            detail: String::from("managed process is running"),
        }
    }

    fn hosted_demo_server_start() -> HostedGuestExpectedOperation {
        HostedGuestExpectedOperation::ManagedServiceStart {
            name: String::from("k3s-server"),
            command: k3s_bootstrap_command(
                "server",
                &[
                    String::from("--disable=traefik"),
                    String::from("--node-name"),
                    String::from("cloud-aws"),
                    String::from("--node-external-ip"),
                    String::from("127.0.0.1"),
                    String::from("--flannel-external-ip"),
                ],
                Some("--cluster-init"),
                None,
                None,
            ),
            policy: hosted_k3s_service_policy("server", "cloud-aws"),
        }
    }

    fn hosted_demo_worker_start() -> HostedGuestExpectedOperation {
        HostedGuestExpectedOperation::ManagedServiceStart {
            name: String::from("k3s-agent"),
            command: k3s_bootstrap_command(
                "agent",
                &[
                    String::from("--node-label=role=worker"),
                    String::from("--node-name"),
                    String::from("cloud-aws-worker"),
                    String::from("--node-external-ip"),
                    String::from("127.0.0.1"),
                    String::from("--flannel-external-ip"),
                ],
                None,
                Some("https://demo-k3s.internal:6443"),
                Some("demo-join-token"),
            ),
            policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
        }
    }

    fn spawn_hosted_guest_sequence_server(
        paths: RuntimePaths,
        expected: Vec<HostedGuestExpectedOperation>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut expected = VecDeque::from(expected);
            let manifest_wait_started = Instant::now();
            while !paths.manifest_path.exists() {
                assert!(
                    manifest_wait_started.elapsed() < Duration::from_secs(120),
                    "machine manifest should exist before binding guest transport at {}",
                    paths.manifest_path.display()
                );
                thread::sleep(Duration::from_millis(10));
            }
            fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
            let listener =
                UnixListener::bind(&paths.vsock_path).expect("guest transport socket should bind");
            listener
                .set_nonblocking(true)
                .expect("guest transport listener should become nonblocking");
            let mut last_request_at = Instant::now();
            let mut requests_seen = false;

            while !expected.is_empty() {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => {
                        requests_seen = true;
                        last_request_at = Instant::now();
                        connection
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if expected.iter().all(|operation| {
                            matches!(
                                operation,
                                HostedGuestExpectedOperation::ManagedServiceStart { .. }
                            )
                        }) && last_request_at.elapsed() >= Duration::from_secs(1)
                            && (requests_seen
                                || manifest_wait_started.elapsed() >= Duration::from_secs(1))
                        {
                            break;
                        }
                        assert!(
                            last_request_at.elapsed() < Duration::from_secs(120),
                            "timed out waiting for hosted guest operation; remaining expected operations: {:?}",
                            expected
                        );
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => panic!("should accept hosted guest transport: {error}"),
                };
                let reader_stream = stream.try_clone().expect("stream should clone");
                let mut reader = BufReader::new(reader_stream);
                let mut handshake = String::new();
                reader
                    .read_line(&mut handshake)
                    .expect("handshake should decode");
                assert!(
                    handshake.starts_with("CONNECT "),
                    "unexpected guest transport handshake: {handshake:?}"
                );
                stream.write_all(b"OK\n").expect("handshake should ack");
                stream.flush().expect("handshake should flush");
                let request: RequestEnvelope =
                    read_frame(&mut reader).expect("request should decode");
                let request_id = request.id;
                if let GuestOperation::ManagedService(ManagedServiceRequest {
                    operation: ManagedServiceOperation::Status { name },
                }) = &request.operation
                {
                    let status = running_managed_service_status(name);
                    write_frame(
                        &mut stream,
                        &ResponseEnvelope::Completed {
                            id: request_id,
                            exit_code: 0,
                            result: OperationResult::ManagedService(ManagedServiceResult::Status(
                                status,
                            )),
                        },
                    )
                    .expect("response should encode");
                    continue;
                }
                let expected_operation = match &request.operation {
                    GuestOperation::Exec(exec_request) => {
                        let index = expected
                            .iter()
                            .position(|operation| match operation {
                                HostedGuestExpectedOperation::Exec { command, .. }
                                | HostedGuestExpectedOperation::ExecFailure { command, .. } => {
                                    command == &exec_request.command
                                }
                                _ => false,
                            })
                            .unwrap_or_else(|| {
                                panic!("unexpected hosted guest operation: {:?}", request.operation)
                            });
                        expected.remove(index).expect("expected exec should exist")
                    }
                    GuestOperation::ManagedService(ManagedServiceRequest {
                        operation: ManagedServiceOperation::Start { name, command, .. },
                    }) => {
                        let index = expected
                            .iter()
                            .position(|operation| match operation {
                                HostedGuestExpectedOperation::ManagedServiceStart {
                                    name: expected_name,
                                    command: expected_command,
                                    ..
                                } => expected_name == name && expected_command == command,
                                _ => false,
                            })
                            .unwrap_or_else(|| {
                                panic!("unexpected hosted guest operation: {:?}", request.operation)
                            });
                        expected
                            .remove(index)
                            .expect("expected service start should exist")
                    }
                    _ => expected
                        .pop_front()
                        .expect("expected operation should exist"),
                };
                match (expected_operation, request.operation) {
                    (
                        HostedGuestExpectedOperation::Exec { command, stdout },
                        GuestOperation::Exec(exec_request),
                    ) => {
                        assert_eq!(exec_request.command, command);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code: 0,
                                result: OperationResult::Exec(ExecResult {
                                    stdout,
                                    stderr: String::new(),
                                }),
                            },
                        )
                        .expect("response should encode");
                    }
                    (
                        HostedGuestExpectedOperation::ExecFailure {
                            command,
                            stderr,
                            exit_code,
                        },
                        GuestOperation::Exec(exec_request),
                    ) => {
                        assert_eq!(exec_request.command, command);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code,
                                result: OperationResult::Exec(ExecResult {
                                    stdout: String::new(),
                                    stderr,
                                }),
                            },
                        )
                        .expect("response should encode");
                    }
                    (
                        HostedGuestExpectedOperation::ManagedServiceStart {
                            name,
                            command,
                            policy,
                        },
                        GuestOperation::ManagedService(ManagedServiceRequest {
                            operation:
                                ManagedServiceOperation::Start {
                                    name: request_name,
                                    kind,
                                    command: request_command,
                                    env,
                                    cwd,
                                    policy: request_policy,
                                },
                        }),
                    ) => {
                        assert_eq!(request_name, name);
                        assert_eq!(kind, ManagedServiceKind::Service);
                        assert_eq!(request_command, command);
                        assert!(env.is_empty());
                        assert_eq!(cwd, None);
                        assert_eq!(request_policy, policy);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code: 0,
                                result: OperationResult::ManagedService(
                                    ManagedServiceResult::Status(ManagedServiceStatus {
                                        name: request_name.clone(),
                                        kind,
                                        state: ManagedServiceRuntimeState::Running,
                                        restart_count: 0,
                                        pid: Some(4242),
                                        exit_code: None,
                                        last_exit_code: None,
                                        last_exit_detail: None,
                                        health_state: ServiceHealthState::Unknown,
                                        health_detail: None,
                                        stdout_path: Some(format!(
                                            "/run/port/services/{request_name}.stdout.log"
                                        )),
                                        stderr_path: Some(format!(
                                            "/run/port/services/{request_name}.stderr.log"
                                        )),
                                        detail: String::from("managed process is running"),
                                    }),
                                ),
                            },
                        )
                        .expect("response should encode");
                    }
                    (_, other) => panic!("unexpected hosted guest operation: {other:?}"),
                }
            }
        })
    }

    fn spawn_hosted_guest_exec_server_with_optional_service_start(
        paths: RuntimePaths,
        optional_service_name: &str,
        optional_service_command: Vec<String>,
        optional_service_policy: ServicePolicy,
        required_exec_command: Vec<String>,
        required_exec_stdout: String,
    ) -> thread::JoinHandle<()> {
        let optional_service_name = optional_service_name.to_string();
        thread::spawn(move || {
            let manifest_wait_started = Instant::now();
            while !paths.manifest_path.exists() {
                assert!(
                    manifest_wait_started.elapsed() < Duration::from_secs(120),
                    "machine manifest should exist before binding guest transport at {}",
                    paths.manifest_path.display()
                );
                thread::sleep(Duration::from_millis(10));
            }
            fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
            let listener =
                UnixListener::bind(&paths.vsock_path).expect("guest transport socket should bind");
            listener
                .set_nonblocking(true)
                .expect("guest transport listener should become nonblocking");

            let started = Instant::now();
            loop {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            started.elapsed() < Duration::from_secs(120),
                            "timed out waiting for hosted guest exec {:?} on {}",
                            required_exec_command,
                            paths.vsock_path.display()
                        );
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => panic!("should accept hosted guest transport: {error}"),
                };
                let reader_stream = stream.try_clone().expect("stream should clone");
                let mut reader = BufReader::new(reader_stream);
                let mut handshake = String::new();
                reader
                    .read_line(&mut handshake)
                    .expect("handshake should decode");
                assert!(
                    handshake.starts_with("CONNECT "),
                    "unexpected guest transport handshake: {handshake:?}"
                );
                stream.write_all(b"OK\n").expect("handshake should ack");
                stream.flush().expect("handshake should flush");
                let request: RequestEnvelope =
                    read_frame(&mut reader).expect("request should decode");
                let request_id = request.id;
                match request.operation {
                    GuestOperation::ManagedService(ManagedServiceRequest {
                        operation: ManagedServiceOperation::Status { name },
                    }) => {
                        let status = running_managed_service_status(&name);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code: 0,
                                result: OperationResult::ManagedService(
                                    ManagedServiceResult::Status(status),
                                ),
                            },
                        )
                        .expect("response should encode");
                    }
                    GuestOperation::ManagedService(ManagedServiceRequest {
                        operation:
                            ManagedServiceOperation::Start {
                                name,
                                kind,
                                command,
                                env,
                                cwd,
                                policy,
                            },
                    }) => {
                        assert_eq!(name, optional_service_name);
                        assert_eq!(kind, ManagedServiceKind::Service);
                        assert_eq!(command, optional_service_command);
                        assert!(env.is_empty());
                        assert_eq!(cwd, None);
                        assert_eq!(policy, optional_service_policy);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code: 0,
                                result: OperationResult::ManagedService(
                                    ManagedServiceResult::Status(ManagedServiceStatus {
                                        name: name.clone(),
                                        kind,
                                        state: ManagedServiceRuntimeState::Running,
                                        restart_count: 0,
                                        pid: Some(4242),
                                        exit_code: None,
                                        last_exit_code: None,
                                        last_exit_detail: None,
                                        health_state: ServiceHealthState::Unknown,
                                        health_detail: None,
                                        stdout_path: Some(format!(
                                            "/run/port/services/{name}.stdout.log"
                                        )),
                                        stderr_path: Some(format!(
                                            "/run/port/services/{name}.stderr.log"
                                        )),
                                        detail: String::from("managed process is running"),
                                    }),
                                ),
                            },
                        )
                        .expect("response should encode");
                    }
                    GuestOperation::Exec(exec_request) => {
                        assert_eq!(exec_request.command, required_exec_command);
                        write_frame(
                            &mut stream,
                            &ResponseEnvelope::Completed {
                                id: request_id,
                                exit_code: 0,
                                result: OperationResult::Exec(ExecResult {
                                    stdout: required_exec_stdout.clone(),
                                    stderr: String::new(),
                                }),
                            },
                        )
                        .expect("response should encode");
                        break;
                    }
                    other => panic!("unexpected hosted guest operation: {other:?}"),
                }
            }
        })
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
    fn driver_selection_routes_avf_machine_to_local_driver() {
        let config = sample_avf_config();
        let driver = driver_for_machine(&config, "demo").expect("driver should resolve");

        assert_eq!(driver.kind(), MachineDriverKind::AvfLocal);
    }

    #[test]
    fn driver_selection_routes_cloud_hypervisor_machine_to_local_driver() {
        let config = PortConfig::sample();
        let driver = driver_for_machine(&config, "demo-ch").expect("driver should resolve");

        assert_eq!(driver.kind(), MachineDriverKind::CloudHypervisorLocal);
    }

    #[test]
    fn driver_selection_routes_ssh_machine_to_ssh_driver() {
        let config = sample_ssh_doctor_config();
        let driver = driver_for_machine(&config, "cloud-generic").expect("driver should resolve");

        assert_eq!(driver.kind(), MachineDriverKind::SshManagedRemote);
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
    fn runtime_guest_materialization_reuses_current_copy_without_recopy() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let source = tempdir.path().join("rootfs.ext4");
        fs::write(&source, b"rootfs-v1").expect("source rootfs should write");
        fs::write(tempdir.path().join("initrd.cpio.gz"), b"initrd").expect("initrd should write");

        let first = super::materialize_runtime_guest_storage(&paths, &source, false)
            .expect("first materialization should succeed");
        let first_inode = fs::metadata(&first.rootfs_path)
            .expect("materialized rootfs should exist")
            .ino();

        let second = super::materialize_runtime_guest_storage(&paths, &source, false)
            .expect("second materialization should succeed");
        let second_inode = fs::metadata(&second.rootfs_path)
            .expect("materialized rootfs should exist")
            .ino();

        assert_eq!(first.rootfs_path, second.rootfs_path);
        assert_eq!(first_inode, second_inode);
        assert!(
            second
                .rootfs_path
                .with_file_name("initrd.cpio.gz")
                .is_file(),
            "copied initrd should remain present next to the materialized rootfs"
        );
    }

    #[test]
    fn runtime_guest_materialization_refreshes_stale_copy() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let source = tempdir.path().join("rootfs.ext4");
        fs::write(&source, b"rootfs-v1").expect("source rootfs should write");

        let first = super::materialize_runtime_guest_storage(&paths, &source, false)
            .expect("first materialization should succeed");
        let first_inode = fs::metadata(&first.rootfs_path)
            .expect("materialized rootfs should exist")
            .ino();

        fs::write(&source, b"rootfs-v2-with-new-bytes").expect("source rootfs should update");

        let refreshed = super::materialize_runtime_guest_storage(&paths, &source, false)
            .expect("refresh should succeed");
        let refreshed_inode = fs::metadata(&refreshed.rootfs_path)
            .expect("refreshed rootfs should exist")
            .ino();

        assert_ne!(first_inode, refreshed_inode);
        assert_eq!(
            fs::read(&refreshed.rootfs_path).expect("refreshed rootfs should read"),
            b"rootfs-v2-with-new-bytes"
        );
    }

    #[test]
    fn runtime_guest_overlay_materialization_is_idempotent() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let source = tempdir.path().join("rootfs.ext4");
        fs::write(&source, b"rootfs-v1").expect("source rootfs should write");
        fs::write(tempdir.path().join("initrd.cpio.gz"), b"initrd").expect("initrd should write");

        let overlay = port_model::MachineRootfsOverlaySpec { size_mib: 64 };
        let first = super::materialize_runtime_guest_storage_with_overlay(
            &paths,
            &source,
            true,
            Some(&overlay),
        )
        .expect("overlay materialization should succeed");
        let overlay_path = first
            .rootfs_overlay_path
            .clone()
            .expect("overlay path should exist");
        let first_inode = fs::metadata(&overlay_path)
            .expect("overlay file should exist")
            .ino();

        let second = super::materialize_runtime_guest_storage_with_overlay(
            &paths,
            &source,
            true,
            Some(&overlay),
        )
        .expect("overlay materialization should remain idempotent");
        let second_overlay_path = second
            .rootfs_overlay_path
            .expect("overlay path should exist");
        let second_inode = fs::metadata(&second_overlay_path)
            .expect("overlay file should exist")
            .ino();

        assert_eq!(first.rootfs_path, source);
        assert_eq!(overlay_path, second_overlay_path);
        assert_eq!(first_inode, second_inode);
        assert_eq!(
            fs::metadata(&overlay_path)
                .expect("overlay file should exist")
                .len(),
            64 * 1024 * 1024
        );
    }

    #[test]
    fn firecracker_config_contains_kernel_rootfs_and_vsock() {
        let config = build_firecracker_config(
            "/tmp/vmlinux".into(),
            "/tmp/rootfs.ext4".into(),
            None,
            &[],
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            false,
            7000,
            52,
            "/tmp/guest.vsock".into(),
            "demo",
            None,
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(json.contains("\"boot-source\""));
        assert!(json.contains("\"/tmp/vmlinux\""));
        assert!(json.contains("\"rootfs\""));
        assert!(json.contains("\"guest_cid\": 52"));
        assert!(json.contains("init=/init"));
        assert!(json.contains("port.guest_control_port=7000"));
        assert!(!json.contains("\"initrd_path\""));
        assert!(!json.contains("\"network-interfaces\""));
    }

    #[test]
    fn firecracker_config_uses_sibling_initrd_when_present() {
        let tempdir = tempdir().expect("tempdir should exist");
        let kernel = tempdir.path().join("vmlinux");
        let rootfs = tempdir.path().join("rootfs.ext4");
        let initrd = tempdir.path().join("initrd.cpio.gz");
        fs::write(&kernel, "kernel").expect("kernel should write");
        fs::write(&rootfs, "rootfs").expect("rootfs should write");
        fs::write(&initrd, "initrd").expect("initrd should write");

        let config = build_firecracker_config(
            kernel,
            rootfs,
            None,
            &[],
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            false,
            7000,
            52,
            tempdir.path().join("guest.vsock"),
            "demo",
            None,
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(json.contains("\"initrd_path\""));
        assert!(json.contains("initrd.cpio.gz"));
    }

    #[test]
    fn firecracker_config_attaches_rootfs_overlay_drive_and_boot_args() {
        let config = build_firecracker_config(
            "/tmp/vmlinux".into(),
            "/tmp/rootfs.ext4".into(),
            Some("/tmp/rootfs-overlay.ext4".into()),
            &[],
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            true,
            7000,
            52,
            "/tmp/guest.vsock".into(),
            "demo",
            None,
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(json.contains("\"rootfs-overlay\""));
        assert!(json.contains("/tmp/rootfs-overlay.ext4"));
        assert!(json.contains("port.rootfs_overlay=1"));
        assert!(json.contains("port.rootfs_overlay_device=/dev/vdb"));
    }

    #[test]
    fn firecracker_config_includes_network_interface_with_default_network_spec() {
        let net = port_model::MachineNetworkSpec::default();
        assert!(
            net.enabled,
            "default MachineNetworkSpec should have enabled=true"
        );
        assert_eq!(net.guest_ip, "172.16.0.2");
        assert_eq!(net.host_ip, "172.16.0.1");
        assert_eq!(net.dns_servers, vec!["8.8.8.8", "8.8.4.4"]);

        let config = build_firecracker_config(
            "/tmp/vmlinux".into(),
            "/tmp/rootfs.ext4".into(),
            None,
            &[],
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            false,
            7000,
            52,
            "/tmp/guest.vsock".into(),
            "demo",
            Some(&net),
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(
            json.contains("\"network-interfaces\""),
            "config JSON must contain network-interfaces key but got:\n{json}"
        );
        assert!(
            json.contains("\"host_dev_name\": \"port-demo\""),
            "config JSON must contain TAP device name"
        );
        assert!(
            json.contains("\"guest_mac\": \"AA:FC:00:00:00:01\""),
            "config JSON must contain guest MAC"
        );
        assert!(
            json.contains("port.net_ip=172.16.0.2"),
            "boot_args must contain guest IP"
        );
        assert!(
            json.contains("port.net_gateway=172.16.0.1"),
            "boot_args must contain gateway"
        );
        assert!(
            json.contains("port.net_dns=8.8.8.8,8.8.4.4"),
            "boot_args must contain DNS servers"
        );
    }

    #[test]
    fn firecracker_config_omits_network_when_none_is_explicitly_disabled() {
        let net = port_model::MachineNetworkSpec {
            enabled: false,
            ..Default::default()
        };
        let config = build_firecracker_config(
            "/tmp/vmlinux".into(),
            "/tmp/rootfs.ext4".into(),
            None,
            &[],
            2,
            512,
            String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
            false,
            7000,
            52,
            "/tmp/guest.vsock".into(),
            "demo",
            Some(&net),
        );
        let json = serde_json::to_string_pretty(&config).expect("config should encode");

        assert!(
            !json.contains("\"network-interfaces\""),
            "config JSON must NOT contain network-interfaces when disabled"
        );
        assert!(
            !json.contains("port.net_ip"),
            "boot_args must NOT contain network params when disabled"
        );
    }

    #[test]
    fn default_network_activates_via_unwrap_or_default() {
        let effective = port_model::MachineNetworkSpec::default();
        assert!(
            effective.enabled,
            "unwrap_or_default on None should produce enabled=true"
        );
        assert_eq!(effective.guest_ip, "172.16.0.2");
        assert_eq!(effective.host_ip, "172.16.0.1");
        assert_eq!(effective.dns_servers, vec!["8.8.8.8", "8.8.4.4"]);
    }

    #[test]
    fn sudo_caller_ids_returns_none_without_env() {
        // Clear env vars if they happen to be set in the test runner.
        unsafe { std::env::remove_var("SUDO_UID") };
        unsafe { std::env::remove_var("SUDO_GID") };
        assert!(sudo_caller_ids().is_none());
    }

    #[test]
    fn chown_runtime_is_noop_without_sudo() {
        unsafe { std::env::remove_var("SUDO_UID") };
        unsafe { std::env::remove_var("SUDO_GID") };
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("artifact.json");
        fs::write(&file, "{}").expect("write");
        // Should be a no-op when not running under sudo.
        chown_runtime_to_sudo_caller(dir.path()).expect("chown_runtime_to_sudo_caller");
    }

    #[test]
    fn chown_recursive_changes_ownership_of_tree() {
        use std::os::unix::fs::MetadataExt;
        let dir = tempdir().expect("tempdir");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).expect("mkdir");
        fs::write(sub.join("file.txt"), "data").expect("write");

        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        // Re-chown to our own uid/gid — this always succeeds for the owner.
        chown_recursive(dir.path(), uid, gid).expect("chown_recursive");

        let meta = fs::metadata(dir.path()).expect("stat root");
        assert_eq!(meta.uid(), uid);
        assert_eq!(meta.gid(), gid);
        let meta = fs::metadata(sub.join("file.txt")).expect("stat file");
        assert_eq!(meta.uid(), uid);
        assert_eq!(meta.gid(), gid);
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
    fn artifact_scripts_resolve_from_packaged_share_root_candidates() {
        let tempdir = tempdir().expect("tempdir should exist");
        let packaged_script = tempdir
            .path()
            .join("share/port/scripts/artifacts/validate-guest-image.sh");
        fs::create_dir_all(
            packaged_script
                .parent()
                .expect("packaged script parent should exist"),
        )
        .expect("packaged script parent should exist");
        fs::write(&packaged_script, "#!/bin/sh\nexit 0\n").expect("packaged script should write");
        let mut permissions = fs::metadata(&packaged_script)
            .expect("packaged script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&packaged_script, permissions)
            .expect("packaged script should be executable");

        let root = repo_root().expect("repo root should resolve");
        let resolved = resolve_artifact_script_path(
            "validate-guest-image.sh",
            [
                packaged_script.clone(),
                root.join("scripts/artifacts/validate-guest-image.sh"),
            ],
        )
        .expect("packaged validate script should resolve");

        assert_eq!(resolved, packaged_script);
    }

    #[test]
    fn artifact_scripts_resolve_from_packaged_root_candidates() {
        let tempdir = tempdir().expect("tempdir should exist");
        let packaged_script = tempdir.path().join("artifacts/validate-guest-image.sh");
        fs::create_dir_all(
            packaged_script
                .parent()
                .expect("packaged script parent should exist"),
        )
        .expect("packaged script parent should exist");
        fs::write(&packaged_script, "#!/bin/sh\nexit 0\n").expect("packaged script should write");
        let mut permissions = fs::metadata(&packaged_script)
            .expect("packaged script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&packaged_script, permissions)
            .expect("packaged script should be executable");

        let root = repo_root().expect("repo root should resolve");
        let resolved = resolve_artifact_script_path(
            "validate-guest-image.sh",
            [
                packaged_script.clone(),
                root.join("scripts/artifacts/validate-guest-image.sh"),
            ],
        )
        .expect("packaged validate script should resolve");

        assert_eq!(resolved, packaged_script);
    }

    #[test]
    fn artifact_validate_pipeline_does_not_require_a_repo_workdir() {
        assert_eq!(
            artifact_pipeline_workdir(ArtifactAction::Validate)
                .expect("validate workdir should resolve"),
            None
        );
        assert_eq!(
            artifact_pipeline_workdir(ArtifactAction::Build).expect("build workdir should resolve"),
            None
        );
    }

    #[test]
    fn repo_managed_guest_image_pipeline_detection_covers_relative_and_absolute_paths() {
        let root = repo_root().expect("repo root should resolve");

        assert!(uses_repo_managed_guest_image_pipeline(Path::new(
            "artifacts/guest/demo/x86_64/firecracker/standard/rootfs.ext4"
        )));
        assert!(uses_repo_managed_guest_image_pipeline(&root.join(
            "artifacts/guest/demo/x86_64/firecracker/standard/rootfs.ext4"
        )));
        assert!(!uses_repo_managed_guest_image_pipeline(Path::new(
            "/tmp/custom-rootfs.ext4"
        )));
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
    fn resolve_guest_image_metadata_distinguishes_standard_and_pvm_paths() {
        let config = PortConfig::sample();

        let standard = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-guest",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: port_model::ProtectionMode::Standard,
            },
        )
        .expect("standard guest-image metadata should resolve");
        let pvm = resolve_artifact_metadata(
            &config,
            ArtifactRequest {
                name: "demo-guest",
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: port_model::ProtectionMode::Pvm,
            },
        )
        .expect("pvm guest-image metadata should resolve");

        assert_ne!(standard.path, pvm.path);
        assert_eq!(
            pvm.path,
            PathBuf::from("artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4")
        );
        assert_eq!(
            pvm.cache_path,
            PathBuf::from(
                ".port/cache/demo-fs/port/demo-guest/v1/x86_64/firecracker/pvm/rootfs.ext4"
            )
        );
    }

    #[test]
    fn resolve_artifact_metadata_reports_missing_selected_pvm_variant_without_fallback() {
        for artifact_name in ["demo-kernel", "demo-guest"] {
            let mut config = PortConfig::sample();
            match artifact_name {
                "demo-kernel" => config
                    .artifacts
                    .kernels
                    .get_mut(artifact_name)
                    .expect("sample kernel should exist")
                    .variants
                    .retain(|variant| {
                        variant.selector.protection_mode != port_model::ProtectionMode::Pvm
                    }),
                "demo-guest" => config
                    .artifacts
                    .guest_images
                    .get_mut(artifact_name)
                    .expect("sample guest image should exist")
                    .variants
                    .retain(|variant| {
                        variant.selector.protection_mode != port_model::ProtectionMode::Pvm
                    }),
                _ => unreachable!("unexpected artifact"),
            }

            let error = resolve_artifact_metadata(
                &config,
                ArtifactRequest {
                    name: artifact_name,
                    architecture: MachineArchitecture::X86_64,
                    substrate: ExecutionSubstrate::Firecracker,
                    protection_mode: port_model::ProtectionMode::Pvm,
                },
            )
            .expect_err("missing pvm variant should fail");

            let message = error.to_string();
            assert!(
                message.contains(&format!("artifact '{artifact_name}' has no variant")),
                "{message}"
            );
            assert!(message.contains("X86_64/Firecracker/Pvm"), "{message}");
            assert!(!message.contains("standard"), "{message}");
        }
    }

    #[test]
    fn resolve_artifact_metadata_reports_missing_selected_cloud_hypervisor_variant_without_fallback()
     {
        for artifact_name in ["demo-kernel", "demo-guest"] {
            let mut config = PortConfig::sample();
            match artifact_name {
                "demo-kernel" => config
                    .artifacts
                    .kernels
                    .get_mut(artifact_name)
                    .expect("sample kernel should exist")
                    .variants
                    .retain(|variant| {
                        variant.selector.substrate != ExecutionSubstrate::CloudHypervisor
                    }),
                "demo-guest" => config
                    .artifacts
                    .guest_images
                    .get_mut(artifact_name)
                    .expect("sample guest image should exist")
                    .variants
                    .retain(|variant| {
                        variant.selector.substrate != ExecutionSubstrate::CloudHypervisor
                    }),
                _ => unreachable!("unexpected artifact"),
            }

            let error = resolve_artifact_metadata(
                &config,
                ArtifactRequest {
                    name: artifact_name,
                    architecture: MachineArchitecture::X86_64,
                    substrate: ExecutionSubstrate::CloudHypervisor,
                    protection_mode: port_model::ProtectionMode::Standard,
                },
            )
            .expect_err("missing cloud hypervisor variant should fail");

            let message = error.to_string();
            assert!(
                message.contains(&format!("artifact '{artifact_name}' has no variant")),
                "{message}"
            );
            assert!(
                message.contains("X86_64/CloudHypervisor/Standard"),
                "{message}"
            );
            assert!(!message.contains("Firecracker"), "{message}");
        }
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

    fn sample_ssh_doctor_config() -> PortConfig {
        let mut config = PortConfig::sample();
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

    #[test]
    fn doctor_ssh_remote_guidance() {
        let report = collect_doctor_report(Some(&sample_ssh_doctor_config()));

        let auth = report
            .checks
            .iter()
            .find(|check| check.name == "host:generic-linux:ssh-auth")
            .expect("ssh auth guidance should exist");
        let bootstrap = report
            .checks
            .iter()
            .find(|check| check.name == "host:generic-linux:ssh-bootstrap")
            .expect("ssh bootstrap guidance should exist");

        assert!(auth.ok);
        assert!(auth.detail.contains("ssh-managed-remote"));
        assert!(auth.detail.contains("ubuntu@builder.example.internal:2222"));
        assert!(auth.detail.contains("SSH auth material"));
        assert!(auth.detail.contains("hosted control-plane bearer tokens"));

        assert!(!bootstrap.ok);
        assert!(bootstrap.detail.contains("bootstrap"));
        assert!(bootstrap.detail.contains("ssh-remote-runtime"));
        assert!(bootstrap.detail.contains("ssh-remote-port-runtime"));
        assert!(bootstrap.detail.contains("generic-linux"));
        assert!(
            report
                .notes
                .iter()
                .any(|note| note.contains("SSH-managed remote hosts surface separate auth"))
        );
    }

    #[test]
    fn doctor_ssh_remote_failure_guidance() {
        let mut config = sample_ssh_doctor_config();
        config
            .hosts
            .get_mut("generic-linux")
            .expect("generic-linux host should exist")
            .provider = HostProvider::Local;

        let report = collect_doctor_report(Some(&config));
        let check = report
            .checks
            .iter()
            .find(|check| check.name == "host:generic-linux")
            .expect("ssh provider failure guidance should exist");

        assert!(!check.ok);
        assert!(check.detail.contains("provider 'local'"));
        assert!(check.detail.contains("generic-linux"));
        assert!(check.detail.contains("ssh-managed-remote"));
        assert!(check.detail.contains("ssh-remote-runtime"));
        assert!(check.detail.contains("ssh-remote-port-runtime"));
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
    fn doctor_attached_volume_guidance() {
        let tempdir = tempdir().expect("tempdir should exist");
        let volume_path = tempdir.path().join("demo-data.ext4");
        fs::write(&volume_path, b"attached-volume").expect("attached volume should write");

        let mut config = PortConfig::sample();
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
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws machine should exist")
            .volumes = vec![MachineVolumeSpec {
            name: String::from("data"),
            backend: MachineVolumeBackend::HostFile,
            persistence: MachineVolumePersistence::Persistent,
            path: volume_path.clone(),
        }];

        let report = collect_doctor_report_with_facts(
            Some(&config),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: std::env::consts::ARCH.to_string(),
                proc_cmdline: None,
                pvm_firecracker_binary: None,
            },
        );

        let local = report
            .checks
            .iter()
            .find(|check| check.name == "machine:demo:volume:data:attached-volume")
            .expect("local attached-volume doctor check should exist");
        assert!(local.ok);
        assert!(local.detail.contains("machine 'demo'"), "{}", local.detail);
        assert!(
            local.detail.contains("attached volume 'data'"),
            "{}",
            local.detail
        );
        assert!(
            local.detail.contains("backend 'host-file'"),
            "{}",
            local.detail
        );
        assert!(
            local
                .detail
                .contains(volume_path.to_string_lossy().as_ref()),
            "{}",
            local.detail
        );
        assert!(
            local.detail.contains("direct-local-runtime"),
            "{}",
            local.detail
        );
        assert!(
            local.detail.contains("local-runtime-root"),
            "{}",
            local.detail
        );
        assert!(
            local.detail.contains("local-port-runtime"),
            "{}",
            local.detail
        );

        let hosted = report
            .checks
            .iter()
            .find(|check| check.name == "machine:cloud-aws:volume:data:attached-volume")
            .expect("hosted attached-volume doctor check should exist");
        assert!(!hosted.ok);
        assert!(
            hosted.detail.contains("machine 'cloud-aws'"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted.detail.contains("attached volume 'data'"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted.detail.contains("backend 'host-file'"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted.detail.contains("hosted-control-plane"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted.detail.contains("hosted-node-agent"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted
                .detail
                .contains("local Firecracker standard lane in this slice"),
            "{}",
            hosted.detail
        );
    }

    #[test]
    fn doctor_report_surfaces_avf_platform_and_boundary_checks() {
        let report = collect_doctor_report_with_facts(
            Some(&sample_avf_config()),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: String::from("x86_64"),
                proc_cmdline: None,
                pvm_firecracker_binary: None,
            },
        );

        let platform = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:host-platform")
            .expect("avf platform check should exist");
        let architecture = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:host-architecture")
            .expect("avf architecture check should exist");
        let availability = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:runtime-availability")
            .expect("avf availability check should exist");

        assert!(!platform.ok);
        assert!(platform.detail.contains("macOS"));
        assert!(architecture.ok);
        assert!(architecture.detail.contains("x86_64"));
        assert!(!availability.ok);
        assert!(availability.detail.contains("PORT_AVF_LAUNCHER"));
        assert!(availability.detail.contains("entitlement"));
        assert!(availability.detail.contains("bundled macOS-only"));
        assert!(
            report
                .notes
                .iter()
                .any(|note| note.contains("AVF lane locally"))
        );
        assert!(
            report
                .notes
                .iter()
                .any(|note| note.contains("PORT_AVF_LAUNCHER"))
        );
    }

    #[test]
    fn doctor_report_marks_avf_checks_ready_on_macos() {
        let report = collect_doctor_report_with_facts(
            Some(&sample_avf_config()),
            &DoctorHostFacts {
                host_os: String::from("macos"),
                host_architecture: String::from("aarch64"),
                proc_cmdline: None,
                pvm_firecracker_binary: None,
            },
        );

        let platform = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:host-platform")
            .expect("avf platform check should exist");
        let architecture = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:host-architecture")
            .expect("avf architecture check should exist");
        let availability = report
            .checks
            .iter()
            .find(|check| check.name == "avf:demo:runtime-availability")
            .expect("avf availability check should exist");

        assert!(platform.ok);
        assert!(architecture.ok);
        assert!(architecture.detail.contains("aarch64"));
        assert!(availability.ok);
        assert!(availability.detail.contains("Virtualization framework"));
        assert!(availability.detail.contains("PORT_AVF_LAUNCHER"));
        assert!(availability.detail.contains("entitlement"));
        assert!(availability.detail.contains("bundled macOS-only"));
    }

    #[test]
    fn doctor_report_surfaces_cloud_hypervisor_platform_and_binary_checks() {
        let tempdir = tempdir().expect("tempdir should exist");
        let _binary = write_fake_firecracker_binary(tempdir.path(), "cloud-hypervisor");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let report = collect_doctor_report_with_facts(
            Some(&PortConfig::sample()),
            &DoctorHostFacts {
                host_os: String::from("linux"),
                host_architecture: String::from("x86_64"),
                proc_cmdline: None,
                pvm_firecracker_binary: None,
            },
        );

        let platform = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:host-platform")
            .expect("cloud hypervisor platform check should exist");
        let architecture = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:host-architecture")
            .expect("cloud hypervisor architecture check should exist");
        let protection_mode = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:protection-mode")
            .expect("cloud hypervisor protection-mode check should exist");
        let binary = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:binary")
            .expect("cloud hypervisor binary check should exist");

        assert!(platform.ok);
        assert!(platform.detail.contains("local Cloud Hypervisor lane"));
        assert!(architecture.ok);
        assert!(architecture.detail.contains("x86_64"));
        assert!(protection_mode.ok);
        assert!(protection_mode.detail.contains("standard protection lane"));
        assert!(binary.ok);
        assert!(binary.detail.contains("cloud-hypervisor"));
    }

    #[test]
    fn doctor_report_fails_fast_for_unsupported_cloud_hypervisor_platform_and_protection_mode() {
        let tempdir = tempdir().expect("tempdir should exist");
        let _binary = write_fake_firecracker_binary(tempdir.path(), "cloud-hypervisor");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        let mut config = PortConfig::sample();
        let machine = config
            .machines
            .get_mut("demo-ch")
            .expect("demo-ch should exist");
        machine.host = String::from("mac-local");
        machine.protection_mode = ProtectionMode::Pvm;

        let report = collect_doctor_report_with_facts(
            Some(&config),
            &DoctorHostFacts {
                host_os: String::from("macos"),
                host_architecture: String::from("x86_64"),
                proc_cmdline: None,
                pvm_firecracker_binary: None,
            },
        );

        let machine_contract = report
            .checks
            .iter()
            .find(|check| check.name == "machine:demo-ch")
            .expect("cloud hypervisor machine contract should exist");
        let platform = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:host-platform")
            .expect("cloud hypervisor platform check should exist");
        let protection_mode = report
            .checks
            .iter()
            .find(|check| check.name == "cloud-hypervisor:demo-ch:protection-mode")
            .expect("cloud hypervisor protection-mode check should exist");

        assert!(!machine_contract.ok);
        assert!(
            machine_contract
                .detail
                .contains("Cloud Hypervisor execution currently expects a Linux host platform.")
        );
        assert!(
            machine_contract
                .detail
                .contains("Port does not currently define a Cloud Hypervisor PVM lane.")
        );
        assert!(!platform.ok);
        assert!(platform.detail.contains("requires a local Linux host"));
        assert!(!protection_mode.ok);
        assert!(
            protection_mode
                .detail
                .contains("does not currently define a Cloud Hypervisor PVM lane")
        );
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
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = ProtectionMode::Pvm;

        let report = collect_doctor_report_with_facts(
            Some(&config),
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

        assert!(!aws.ok);
        assert!(aws.detail.contains("firecracker-pvm-host-kit@2026.04"));
        assert!(aws.detail.contains("cloud-aws"));
        assert!(aws.detail.contains("prepare-pvm-node"));
        assert!(aws.detail.contains("planned"));
        assert!(!generic.ok);
        assert!(generic.detail.contains("host-kit contract"));
    }

    #[test]
    fn doctor_report_uses_imported_prepared_hosted_pvm_state() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));

        let config = PortConfig::sample();
        let host_kit = config.nodes["aws-linux-node"].capabilities.pvm_lanes[0]
            .host_kit
            .clone()
            .expect("aws x86_64 PVM lane should define a host-kit");
        let mut imported_summary = config.nodes["aws-linux-node"].capabilities.clone();
        imported_summary.pvm_lanes[0].state = PvmCapabilityState::Ready;
        imported_summary.pvm_lanes[0].host_kit = Some(host_kit.clone());
        write_imported_inventory_state(
            "demo",
            BTreeMap::from([(
                String::from("aws-linux-node"),
                HostedImportedNodeRecord {
                    provider: HostProvider::Aws,
                    provenance: String::from("inventory/aws-linux-node.json"),
                    imported_at: 1_700_000_123,
                    capability_summary: imported_summary,
                    pvm_host_kit_packages: vec![port_model::HostedPvmHostKitPackageAttachment {
                        architecture: MachineArchitecture::X86_64,
                        package: host_kit.package.clone(),
                    }],
                },
            )]),
        );

        let report = collect_doctor_report_with_facts(
            Some(&config),
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

        assert!(aws.ok);
        assert!(aws.detail.contains("inventory/aws-linux-node.json"));
        assert!(aws.detail.contains("firecracker-pvm-host-kit@2026.04"));
        assert!(aws.detail.contains("6.12.0-port-pvm"));
        assert!(
            aws.detail
                .contains("v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c")
        );

        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
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
            runtime_class: None,
            attached_volumes: Vec::new(),
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
            runtime_class: None,
            attached_volumes: Vec::new(),
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

    #[cfg(target_os = "linux")]
    #[test]
    fn process_is_live_treats_zombie_pid_as_not_running() {
        let mut child = Command::new("bash")
            .args(["-lc", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("zombie candidate should start");
        wait_for_process_state(child.id(), 'Z');

        assert!(!super::process_is_live(child.id()).expect("zombie liveness probe should succeed"));

        let _ = child.wait();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn detached_forward_status_treats_zombie_pid_as_stale() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");

        let mut child = Command::new("bash")
            .args(["-lc", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("forward helper should start");
        wait_for_process_state(child.id(), 'Z');

        write_detached_forward_manifest(
            &paths,
            "web",
            child.id(),
            "127.0.0.1:8081",
            "127.0.0.1:80",
        );

        let forwards = super::load_detached_forward_statuses(&paths.runtime_dir, "demo")
            .expect("forward status should load");
        assert_eq!(forwards.len(), 1);
        assert_eq!(forwards[0].state, MachineRuntimeState::Stale);
        assert_eq!(forwards[0].pid, Some(child.id()));
        assert!(forwards[0].detail.contains("no longer live"));

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
                    workspace: String::from("demo"),
                    lane: String::from("scratch"),
                }),
            }),
            attached_volumes: Vec::new(),
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
        assert_eq!(status.runtime_class, manifest.runtime_class);
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
                    workspace: String::from("demo"),
                    lane: String::from("scratch"),
                }),
            }),
            attached_volumes: Vec::new(),
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
                runtime_class: manifest.runtime_class.clone(),
                attached_volumes: Vec::new(),
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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
    fn hosted_machine_status_prefers_live_candidate_selection_under_stale_placement() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let stored_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let live_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let live_paths = RuntimePaths::for_machine(&live_runtime_root, "cloud-aws");
        write_manifest(&live_paths, "cloud-aws", 424242);

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node-b",
            &stored_runtime_root,
            "Stored on alternate AWS node.",
        );

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted status should load");
        assert_eq!(status.machine_name, "cloud-aws");
        assert_eq!(status.state, MachineRuntimeState::Stopped);
        assert_eq!(status.runtime_dir, live_paths.runtime_dir);
        assert!(
            status.detail.contains("control plane 'demo'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("node 'aws-linux-node'"),
            "{}",
            status.detail
        );
        assert!(
            !status.detail.contains("Stored on alternate AWS node."),
            "{}",
            status.detail
        );
    }

    #[test]
    fn hosted_machine_resolution_uses_live_control_plane_status_for_remote_runtime_roots() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        let remote_runtime_root = PathBuf::from("/remote/hosted/aws-linux-node-b");
        config
            .nodes
            .get_mut("aws-linux-node-b")
            .expect("aws-linux-node-b should exist")
            .runtime_root = remote_runtime_root.clone();
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node-b",
            &remote_runtime_root,
            "Stored on alternate AWS node.",
        );

        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let runtime_root_for_route = remote_runtime_root.clone();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router = Router::new().route(
                    "/v1/machines/{machine}",
                    get(move |AxumPath(machine): AxumPath<String>| {
                        let runtime_root = runtime_root_for_route.clone();
                        async move {
                            Json(HostedSuccess {
                                route: HostedRouteContext {
                                    control_plane: Some(String::from("demo")),
                                    machine_name: Some(machine.clone()),
                                    node_name: Some(String::from("aws-linux-node-b")),
                                    runtime_root: Some(runtime_root.clone()),
                                    ..HostedRouteContext::default()
                                },
                                result: MachineStatus {
                                    machine_name: machine.clone(),
                                    state: MachineRuntimeState::Running,
                                    pid: Some(4321),
                                    control:
                                        port_model::MachineControlContract::hosted_control_plane(),
                                    runtime_dir: runtime_root.join(&machine),
                                    config_path: runtime_root.join(&machine).join("config.json"),
                                    manifest_path: runtime_root
                                        .join(&machine)
                                        .join("manifest.json"),
                                    pid_path: runtime_root.join(&machine).join("machine.pid"),
                                    firecracker_log: runtime_root
                                        .join(&machine)
                                        .join("firecracker.log"),
                                    stdout_log: runtime_root.join(&machine).join("stdout.log"),
                                    stderr_log: runtime_root.join(&machine).join("stderr.log"),
                                    runtime_class: None,
                                    attached_volumes: Vec::new(),
                                    hosted_fleet_nodes: Vec::new(),
                                    guest_refresh_age_seconds: None,
                                    wedged_since_unix_s: None,
                                    wedge_class: None,
                                    recovery_attempts: RecoveryAttemptCounters::default(),
                                    last_recovery_action: None,
                                    recovery_state: RecoveryState::default(),
                                    detail: String::from("mock remote machine status"),
                                },
                            })
                        }
                    }),
                );
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let resolution =
            hosted_machine_resolution(&config, "cloud-aws").expect("hosted resolution should load");
        assert_eq!(resolution.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(resolution.runtime_root, remote_runtime_root);
        assert_eq!(resolution.status.state, MachineRuntimeState::Running);
        assert!(
            !resolution
                .status
                .detail
                .contains("does not contain machine state"),
            "{}",
            resolution.status.detail
        );
        assert!(
            resolution
                .status
                .detail
                .contains("Stored on alternate AWS node."),
            "{}",
            resolution.status.detail
        );

        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_machine_resolution_keeps_stored_placement_when_live_status_is_fallback_malformed() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let stored_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let fallback_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();

        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node-b",
            &stored_runtime_root,
            "Stored on alternate AWS node.",
        );

        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let fallback_runtime_root_for_route = fallback_runtime_root.clone();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router = Router::new().route(
                    "/v1/machines/{machine}",
                    get(move |AxumPath(machine): AxumPath<String>| {
                        let fallback_runtime_root = fallback_runtime_root_for_route.clone();
                        async move {
                            Json(HostedSuccess {
                                route: HostedRouteContext {
                                    control_plane: Some(String::from("demo")),
                                    machine_name: Some(machine.clone()),
                                    node_name: Some(String::from("aws-linux-node")),
                                    runtime_root: Some(fallback_runtime_root.clone()),
                                    ..HostedRouteContext::default()
                                },
                                result: MachineStatus {
                                    machine_name: machine.clone(),
                                    state: MachineRuntimeState::Malformed,
                                    pid: None,
                                    control:
                                        port_model::MachineControlContract::hosted_control_plane(),
                                    runtime_dir: fallback_runtime_root.join(&machine),
                                    config_path: fallback_runtime_root
                                        .join(&machine)
                                        .join("config.json"),
                                    manifest_path: fallback_runtime_root
                                        .join(&machine)
                                        .join("manifest.json"),
                                    pid_path: fallback_runtime_root
                                        .join(&machine)
                                        .join("machine.pid"),
                                    firecracker_log: fallback_runtime_root
                                        .join(&machine)
                                        .join("firecracker.log"),
                                    stdout_log: fallback_runtime_root
                                        .join(&machine)
                                        .join("stdout.log"),
                                    stderr_log: fallback_runtime_root
                                        .join(&machine)
                                        .join("stderr.log"),
                                    runtime_class: None,
                                    attached_volumes: Vec::new(),
                                    hosted_fleet_nodes: Vec::new(),
                                    guest_refresh_age_seconds: None,
                                    wedged_since_unix_s: None,
                                    wedge_class: None,
                                    recovery_attempts: RecoveryAttemptCounters::default(),
                                    last_recovery_action: None,
                                    recovery_state: RecoveryState::default(),
                                    detail: String::from(
                                        "fallback candidate produced malformed status",
                                    ),
                                },
                            })
                        }
                    }),
                );
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let resolution =
            hosted_machine_resolution(&config, "cloud-aws").expect("hosted resolution should load");
        assert_eq!(resolution.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(resolution.runtime_root, stored_runtime_root);
        assert_eq!(resolution.status.state, MachineRuntimeState::Malformed);
        assert!(
            resolution
                .status
                .detail
                .contains("stored node 'aws-linux-node-b'"),
            "{}",
            resolution.status.detail
        );
        assert!(
            resolution
                .status
                .detail
                .contains("Stored on alternate AWS node."),
            "{}",
            resolution.status.detail
        );
        assert!(
            !resolution
                .status
                .detail
                .contains("node 'aws-linux-node'. Stored on alternate AWS node."),
            "{}",
            resolution.status.detail
        );

        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn resolve_service_runtime_context_skips_local_fs_gates_for_remote_hosted_runtime() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let local_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let remote_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let remote_paths = RuntimePaths::for_machine(&remote_runtime_root, "cloud-aws");

        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let runtime_root_for_route = remote_runtime_root.clone();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router = Router::new().route(
                    "/v1/machines/{machine}",
                    get(move |AxumPath(machine): AxumPath<String>| {
                        let runtime_root = runtime_root_for_route.clone();
                        async move {
                            Json(HostedSuccess {
                                route: HostedRouteContext {
                                    control_plane: Some(String::from("demo")),
                                    machine_name: Some(machine.clone()),
                                    node_name: Some(String::from("aws-linux-node-b")),
                                    runtime_root: Some(runtime_root.clone()),
                                    ..HostedRouteContext::default()
                                },
                                result: MachineStatus {
                                    machine_name: machine.clone(),
                                    state: MachineRuntimeState::Running,
                                    pid: Some(4321),
                                    control:
                                        port_model::MachineControlContract::hosted_control_plane(),
                                    runtime_dir: runtime_root.join(&machine),
                                    config_path: runtime_root.join(&machine).join("config.json"),
                                    manifest_path: runtime_root
                                        .join(&machine)
                                        .join("manifest.json"),
                                    pid_path: runtime_root.join(&machine).join("machine.pid"),
                                    firecracker_log: runtime_root
                                        .join(&machine)
                                        .join("firecracker.log"),
                                    stdout_log: runtime_root.join(&machine).join("stdout.log"),
                                    stderr_log: runtime_root.join(&machine).join("stderr.log"),
                                    runtime_class: None,
                                    attached_volumes: Vec::new(),
                                    hosted_fleet_nodes: Vec::new(),
                                    guest_refresh_age_seconds: None,
                                    wedged_since_unix_s: None,
                                    wedge_class: None,
                                    recovery_attempts: RecoveryAttemptCounters::default(),
                                    last_recovery_action: None,
                                    recovery_state: RecoveryState::default(),
                                    detail: String::from("mock remote hosted status"),
                                },
                            })
                        }
                    }),
                );
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let context =
            super::resolve_service_runtime_context(&config, &local_runtime_root, "cloud-aws", None)
                .expect("service runtime context should resolve to remote control-plane truth");

        assert_eq!(context.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(context.control_plane.as_deref(), Some("demo"));
        assert_eq!(context.status.runtime_dir, remote_paths.runtime_dir);
        assert_eq!(context.status.state, MachineRuntimeState::Running);
        assert!(
            !context.status.runtime_dir.exists(),
            "remote hosted runtime dir should be accepted as a valid control-plane routed contract"
        );
    }

    #[test]
    fn hosted_machine_list_monitor_and_stop_prefer_live_candidate_selection_under_stale_placement()
    {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let stored_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let live_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let live_paths = RuntimePaths::for_machine(&live_runtime_root, "cloud-aws");
        write_manifest(&live_paths, "cloud-aws", 424242);

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node-b",
            &stored_runtime_root,
            "Stored on alternate AWS node.",
        );

        let machines = list_machines(&config, tempdir.path()).expect("machine list should load");
        let hosted = machines
            .iter()
            .find(|machine| machine.machine_name == "cloud-aws")
            .expect("hosted machine should appear in machine list");
        assert_eq!(hosted.state, MachineRuntimeState::Stopped);
        assert_eq!(hosted.runtime_dir, live_paths.runtime_dir);
        assert!(
            hosted.detail.contains("control plane 'demo'"),
            "{}",
            hosted.detail
        );
        assert!(
            hosted.detail.contains("node 'aws-linux-node'"),
            "{}",
            hosted.detail
        );
        assert!(
            !hosted.detail.contains("Stored on alternate AWS node."),
            "{}",
            hosted.detail
        );

        let monitor =
            machine_monitor(&config, tempdir.path(), "cloud-aws").expect("monitor should load");
        assert_eq!(monitor.state, MachineRuntimeState::Stopped);
        assert_eq!(monitor.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(monitor.runtime_dir, live_paths.runtime_dir);
        assert!(
            monitor.detail.contains("control plane 'demo'"),
            "{}",
            monitor.detail
        );
        assert!(
            monitor.detail.contains("node 'aws-linux-node'"),
            "{}",
            monitor.detail
        );
        assert!(
            !monitor.detail.contains("Stored on alternate AWS node."),
            "{}",
            monitor.detail
        );

        let stop = stop_machine(&config, tempdir.path(), "cloud-aws", Duration::from_secs(1))
            .expect("stop should load");
        assert_eq!(stop.previous_state, MachineRuntimeState::Stopped);
        assert_eq!(stop.current_state, MachineRuntimeState::Stopped);
        assert_eq!(stop.runtime_dir, live_paths.runtime_dir);
        assert!(
            stop.detail.contains("control plane 'demo'"),
            "{}",
            stop.detail
        );
        assert!(
            stop.detail.contains("node 'aws-linux-node'"),
            "{}",
            stop.detail
        );
        assert!(
            !stop.detail.contains("Stored on alternate AWS node."),
            "{}",
            stop.detail
        );
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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
        assert_eq!(hosted.hosted_fleet_nodes.len(), 1);
        assert_eq!(hosted.hosted_fleet_nodes[0].node_name, "aws-linux-node");
        assert_eq!(
            hosted.hosted_fleet_nodes[0].freshness,
            crate::HostedFleetFreshnessState::Live
        );
        assert_eq!(
            hosted.hosted_fleet_nodes[0].routing_eligibility,
            crate::HostedFleetRoutingEligibility::Eligible
        );
    }

    #[test]
    fn list_machines_skips_hosted_control_planes_when_auth_env_is_missing() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let local_paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        write_manifest(&local_paths, "demo", 424242);

        let previous = std::env::var_os("PORT_DEMO_TOKEN");
        unsafe {
            std::env::remove_var("PORT_DEMO_TOKEN");
        }
        let machines = list_machines(&config, tempdir.path())
            .expect("machine list should still load local machines without hosted auth");
        match previous {
            Some(value) => unsafe {
                std::env::set_var("PORT_DEMO_TOKEN", value);
            },
            None => unsafe {
                std::env::remove_var("PORT_DEMO_TOKEN");
            },
        }

        let demo = machines
            .iter()
            .find(|machine| machine.machine_name == "demo")
            .expect("local demo machine should still be listed");
        assert!(
            matches!(
                demo.state,
                MachineRuntimeState::Running | MachineRuntimeState::Stopped
            ),
            "local demo machine should still resolve to a local runtime state"
        );
        assert!(
            machines
                .iter()
                .all(|machine| machine.machine_name != "cloud-aws"),
            "hosted machines should be skipped when hosted auth is unavailable"
        );
    }

    #[test]
    fn hosted_fleet_state_surfaces_live_stale_and_imported_only_nodes() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_machine_config(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");

        let mut missing = config
            .nodes
            .get("aws-linux-node")
            .expect("aws-linux-node should exist")
            .clone();
        missing.runtime_root = tempdir.path().join("hosted/aws-linux-node-c");
        config
            .nodes
            .insert(String::from("aws-linux-node-c"), missing);

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        write_manifest(&paths, "cloud-aws", 424242);

        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }
        let _ = fs::remove_dir_all(super::hosted_placeholder_runtime_root_for_config(
            &config, "demo",
        ));
        write_imported_inventory_state(
            "demo",
            BTreeMap::from([
                (
                    String::from("aws-linux-node"),
                    port_model::HostedImportedNodeRecord {
                        provider: port_model::HostProvider::Aws,
                        provenance: String::from("inventory-sync"),
                        imported_at: 100,
                        capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                        pvm_host_kit_packages: Vec::new(),
                    },
                ),
                (
                    String::from("aws-linux-node-b"),
                    port_model::HostedImportedNodeRecord {
                        provider: port_model::HostProvider::Aws,
                        provenance: String::from("inventory-sync"),
                        imported_at: 200,
                        capability_summary: config.nodes["aws-linux-node-b"].capabilities.clone(),
                        pvm_host_kit_packages: Vec::new(),
                    },
                ),
                (
                    String::from("aws-linux-node-c"),
                    port_model::HostedImportedNodeRecord {
                        provider: port_model::HostProvider::Aws,
                        provenance: String::from("inventory-sync"),
                        imported_at: 300,
                        capability_summary: config.nodes["aws-linux-node-c"].capabilities.clone(),
                        pvm_host_kit_packages: Vec::new(),
                    },
                ),
            ]),
        );
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix timestamp should resolve")
            .as_secs();
        write_registered_node_state(
            "demo",
            BTreeMap::from([(
                String::from("aws-linux-node-b"),
                port_model::HostedNodeRegistration {
                    endpoint: String::from("http://127.0.0.1:9"),
                    token: String::from("node-secret"),
                    registered_at: now.saturating_sub(10),
                    refreshed_at: now.saturating_sub(10),
                    ttl_seconds: 1,
                },
            )]),
        );

        let mut client_config = config.clone();
        let control_plane_addr =
            start_live_control_plane(&client_config, None).expect("control plane should start");
        client_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{control_plane_addr}");
        start_live_named_node_agent(&client_config, "aws-linux-node")
            .expect("named node agent should start");

        let status = machine_status(&client_config, tempdir.path(), "cloud-aws")
            .expect("hosted status should load");
        assert_eq!(status.hosted_fleet_nodes.len(), 3);

        let live = status
            .hosted_fleet_nodes
            .iter()
            .find(|node| node.node_name == "aws-linux-node")
            .expect("live node should exist");
        assert!(live.configured);
        assert!(live.imported);
        assert!(live.registered);
        assert!(live.selected);
        assert_eq!(live.freshness, crate::HostedFleetFreshnessState::Live);
        assert_eq!(
            live.routing_eligibility,
            crate::HostedFleetRoutingEligibility::Eligible
        );

        let stale = status
            .hosted_fleet_nodes
            .iter()
            .find(|node| node.node_name == "aws-linux-node-b")
            .expect("stale node should exist");
        assert!(stale.configured);
        assert!(stale.imported);
        assert!(stale.registered);
        assert!(!stale.selected);
        assert_eq!(stale.freshness, crate::HostedFleetFreshnessState::Stale);
        assert_eq!(
            stale.routing_eligibility,
            crate::HostedFleetRoutingEligibility::StaleRegistration
        );
        assert!(stale.detail.contains("inventory-sync"));
        assert!(stale.detail.contains("expired"));

        let missing = status
            .hosted_fleet_nodes
            .iter()
            .find(|node| node.node_name == "aws-linux-node-c")
            .expect("missing-registration node should exist");
        assert!(missing.configured);
        assert!(missing.imported);
        assert!(!missing.registered);
        assert!(!missing.selected);
        assert_eq!(
            missing.freshness,
            crate::HostedFleetFreshnessState::MissingRegistration
        );
        assert_eq!(
            missing.routing_eligibility,
            crate::HostedFleetRoutingEligibility::MissingRegistration
        );
        assert!(
            missing
                .detail
                .contains("No registered node-agent endpoint.")
        );

        let machines =
            list_machines(&client_config, tempdir.path()).expect("machine list should load");
        let listed = machines
            .iter()
            .find(|machine| machine.machine_name == "cloud-aws")
            .expect("cloud-aws should be listed");
        assert_eq!(listed.hosted_fleet_nodes, status.hosted_fleet_nodes);
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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
    fn hosted_guest_exec_routes_cloud_hypervisor_machine_through_node_runtime_root() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .substrate = ExecutionSubstrate::CloudHypervisor;
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .substrates = vec![ExecutionSubstrate::CloudHypervisor];

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
                        vec![String::from("/bin/echo"), String::from("hosted-ch-ok")]
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
                        stdout: String::from("hosted-ch-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("response should encode");
        });
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

        let result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("hosted-ch-ok")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("hosted cloud-hypervisor guest exec should succeed");

        match result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "hosted-ch-ok\n"),
            other => panic!("unexpected result: {other:?}"),
        }

        server.join().expect("server thread should complete");
    }

    #[test]
    fn hosted_guest_pty_routes_through_live_control_plane() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("pty accept");
            let reader_stream = stream.try_clone().expect("pty clone");
            let mut reader = BufReader::new(reader_stream);
            let request: RequestEnvelope = read_frame(&mut reader).expect("pty request");
            let GuestOperation::Pty(request) = request.operation else {
                panic!("unexpected hosted pty operation");
            };
            assert_eq!(
                request.command,
                vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf hosted-pty-ok"),
                ]
            );
            write_frame(
                &mut stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Pty,
                    size_bytes: None,
                },
            )
            .expect("pty accepted should encode");
            let close: StreamRequestFrame =
                read_frame(&mut reader).expect("pty close should decode");
            assert!(matches!(close, StreamRequestFrame::Close));
            write_frame(
                &mut stream,
                &StreamResponseFrame::Data {
                    channel: port_agent_protocol::StreamOutputChannel::Stdout,
                    data: String::from("hosted-pty-ok"),
                },
            )
            .expect("pty data should encode");
            write_frame(&mut stream, &StreamResponseFrame::Exit { exit_code: 0 })
                .expect("pty exit should encode");
        });

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        let result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Pty(PtyRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("printf hosted-pty-ok"),
                    ],
                    cols: 80,
                    rows: 24,
                }),
            },
        )
        .expect("hosted guest pty should succeed");

        let OperationResult::Pty(result) = result else {
            panic!("unexpected hosted pty result: {result:?}");
        };
        assert_eq!(result.transcript, "hosted-pty-ok");

        server.join().expect("server thread should complete");
    }

    #[test]
    fn hosted_guest_exec_explains_unresolved_node_routing() {
        let config = start_live_hosted_servers(&PortConfig::sample(), false)
            .expect("hosted control plane should start");
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
    fn hosted_k3s_bootstrap_and_join_workflow() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let worker_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let server_guest = spawn_hosted_guest_exec_server_with_optional_service_start(
            worker_paths,
            "k3s-server",
            k3s_bootstrap_command(
                "server",
                &[
                    String::from("--disable=traefik"),
                    String::from("--node-name"),
                    String::from("cloud-aws"),
                    String::from("--node-external-ip"),
                    String::from("127.0.0.1"),
                    String::from("--flannel-external-ip"),
                ],
                Some("--cluster-init"),
                None,
                None,
            ),
            hosted_k3s_service_policy("server", "cloud-aws"),
            hosted_k3s_join_token_command(),
            String::from("demo-join-token\n"),
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let result = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");

        assert_eq!(result.cluster_name, "demo");
        assert_eq!(result.control_plane, "demo");
        assert_eq!(result.host_group, "aws-builders");
        assert_eq!(result.server_machines, vec![String::from("cloud-aws")]);
        assert_eq!(
            result.worker_machines,
            vec![String::from("cloud-aws-worker")]
        );
        assert_eq!(result.api_endpoint, "https://demo-k3s.internal:6443");
        assert_eq!(
            result.stable_endpoint_posture,
            super::HostedK3sStableEndpointPosture::ManualRewriteRequired
        );
        assert!(
            result
                .stable_endpoint_detail
                .contains("stable endpoint posture is manual-rewrite-required"),
            "{}",
            result.stable_endpoint_detail
        );
        assert_eq!(result.join_token, "demo-join-token");
        assert_eq!(result.server_launches.len(), 1);
        assert_eq!(result.worker_launches.len(), 1);

        server_guest
            .join()
            .expect("server guest thread should complete");

        let _ = Command::new("kill")
            .arg(result.server_launches[0].pid.to_string())
            .status();
        for metadata in result.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn hosted_k3s_service_start_accepts_matching_status_after_apply_timeout() {
        #[derive(Clone)]
        struct MockHostedServiceState {
            control_plane: String,
            status: ServiceDefinitionStatus,
        }

        async fn ready_handler() -> StatusCode {
            StatusCode::OK
        }

        async fn apply_handler(
            State(state): State<MockHostedServiceState>,
            AxumPath(machine): AxumPath<String>,
        ) -> (StatusCode, Json<HostedSuccess<ServiceDefinitionStatus>>) {
            tokio::time::sleep(Duration::from_millis(150)).await;
            (
                StatusCode::OK,
                Json(HostedSuccess {
                    route: HostedRouteContext {
                        control_plane: Some(state.control_plane.clone()),
                        machine_name: Some(machine),
                        service_name: Some(String::from("k3s-agent")),
                        ..HostedRouteContext::default()
                    },
                    result: state.status,
                }),
            )
        }

        async fn status_handler(
            State(state): State<MockHostedServiceState>,
            AxumPath((machine, service)): AxumPath<(String, String)>,
        ) -> Json<HostedSuccess<ServiceDefinitionStatus>> {
            Json(HostedSuccess {
                route: HostedRouteContext {
                    control_plane: Some(state.control_plane.clone()),
                    machine_name: Some(machine),
                    service_name: Some(service),
                    ..HostedRouteContext::default()
                },
                result: state.status,
            })
        }

        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        let token_var = format!("PORT_TEST_HOSTED_K3S_APPLY_TIMEOUT_{}", std::process::id());
        let control_plane = String::from("demo");
        let args = vec![
            String::from("--node-label=role=worker"),
            String::from("--node-name"),
            String::from("cloud-aws-worker"),
            String::from("--node-external-ip"),
            String::from("127.0.0.1"),
            String::from("--flannel-external-ip"),
        ];
        let command = super::hosted_k3s_service_command(
            "agent",
            &args,
            None,
            Some("https://demo-k3s.internal:6443"),
            Some("demo-join-token"),
        );
        let status = ServiceDefinitionStatus {
            machine_name: String::from("cloud-aws-worker"),
            name: String::from("k3s-agent"),
            kind: ServiceKind::Service,
            desired_state: ServiceDesiredState::Active,
            runtime: super::ServiceRuntimeObservation {
                state: ServiceRuntimeState::Running,
                record_path: tempdir.path().join("runtime/k3s-agent.json"),
                restart_count: 0,
                pid: Some(96),
                exit_code: None,
                last_exit_code: None,
                last_exit_detail: None,
                health_state: ServiceHealthState::Healthy,
                health_detail: Some(String::from("mock healthy")),
                stdout_path: None,
                stderr_path: None,
            },
            command: command.clone(),
            secret_bindings: Vec::new(),
            secret_sources: Vec::new(),
            policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
            control: port_model::MachineControlContract::hosted_control_plane(),
            control_plane: Some(control_plane.clone()),
            node_name: Some(String::from("aws-linux-node")),
            host_groups: vec![String::from("aws-builders")],
            host_group_policies: BTreeMap::new(),
            target_host_group: Some(String::from("aws-builders")),
            scheduler: Some(HostedSchedulerPolicy::DeterministicFirstFit),
            manifest_path: tempdir.path().join("runtime/k3s-agent.manifest.json"),
            detail: String::from("mock service status"),
        };

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        config
            .control_planes
            .get_mut(&control_plane)
            .expect("control plane should exist")
            .endpoint = format!("http://{addr}");
        config
            .control_planes
            .get_mut(&control_plane)
            .expect("control plane should exist")
            .auth
            .source = port_model::HostedAuthTokenSource::Env {
            variable: token_var.clone(),
        };
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let router = Router::new()
            .route("/__ready", get(ready_handler))
            .route("/v1/machines/{machine}/services", post(apply_handler))
            .route(
                "/v1/machines/{machine}/services/{service}",
                get(status_handler),
            )
            .with_state(MockHostedServiceState {
                control_plane: control_plane.clone(),
                status: status.clone(),
            });
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        tokio::time::sleep(Duration::from_millis(25)).await;

        let runtime_root = tempdir.path().to_path_buf();
        let config_for_call = config.clone();
        let args_for_call = args.clone();
        let control_plane_for_call = control_plane.clone();
        let result = tokio::task::spawn_blocking(move || {
            execute_hosted_k3s_managed_service_start_with_retry(
                &config_for_call,
                &runtime_root,
                "cloud-aws-worker",
                "aws-builders",
                "agent",
                &args_for_call,
                None,
                Some("https://demo-k3s.internal:6443"),
                Some("demo-join-token"),
                "join the K3s worker",
                &control_plane_for_call,
                Duration::from_millis(50),
                Duration::from_millis(50),
                Duration::from_millis(250),
                Duration::from_millis(10),
            )
        })
        .await
        .expect("blocking hosted service start should join")
        .expect("matching follow-up status should satisfy hosted service start");

        unsafe {
            std::env::remove_var(&token_var);
        }

        assert_eq!(result.name, "k3s-agent");
        assert_eq!(result.pid, Some(96));
        assert_eq!(result.state, ManagedServiceRuntimeState::Running);
        assert_eq!(result.health_state, ServiceHealthState::Healthy);
    }

    #[test]
    fn hosted_k3s_server_healthcheck_requires_runtime_and_readyz() {
        let policy = hosted_k3s_service_policy("server", "cloud-aws");
        assert_eq!(policy.restart, ServiceRestartPolicy::Always);
        assert_eq!(policy.healthcheck.policy, ServiceHealthPolicy::Command);
        assert!(policy.healthcheck.restart_on_unhealthy);
        assert_eq!(policy.healthcheck.command[0], "/bin/sh");
        assert_eq!(policy.healthcheck.command[1], "-lc");
        let shell = &policy.healthcheck.command[2];
        assert!(shell.contains("/usr/bin/k3s crictl info"));
        assert!(shell.contains(
            "/usr/bin/k3s kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml --request-timeout=10s get --raw=/readyz"
        ));
    }

    #[test]
    fn hosted_k3s_agent_healthcheck_uses_lease_grace_window_for_transient_failures() {
        let policy = hosted_k3s_service_policy("agent", "cloud-aws-worker");
        assert_eq!(policy.restart, ServiceRestartPolicy::Always);
        assert_eq!(policy.healthcheck.policy, ServiceHealthPolicy::Command);
        assert!(policy.healthcheck.restart_on_unhealthy);
        assert_eq!(policy.healthcheck.command[0], "/bin/sh");
        assert_eq!(policy.healthcheck.command[1], "-lc");
        let shell = &policy.healthcheck.command[2];
        assert!(shell.contains("/usr/bin/k3s crictl info"));
        assert!(shell.contains(
            "/usr/bin/k3s kubectl --kubeconfig /var/lib/rancher/k3s/agent/kubelet.kubeconfig --request-timeout=10s get --raw=/readyz"
        ));
        assert!(shell.contains("last_ok_file=\"$state_dir/k3s-agent-cluster-ok\""));
        assert!(shell.contains("bootstrap_start_file=\"$state_dir/k3s-agent-bootstrap-start\""));
        assert!(shell.contains("cluster_ok=0"));
        assert!(shell.contains("lease_renew_time="));
        assert!(shell.contains("-n kube-node-lease get lease 'cloud-aws-worker'"));
        assert!(shell.contains(".spec.renewTime"));
        assert!(shell.contains("if [ \"$cluster_ok\" -eq 1 ]"));
        assert!(shell.contains("if [ -f \"$last_ok_file\" ]"));
        assert!(shell.contains("bootstrap_epoch=$(cat \"$bootstrap_start_file\" 2>/dev/null)"));
        assert!(shell.contains("/bin/busybox date -u -D '%Y-%m-%dT%H:%M:%S'"));
        assert!(shell.contains("now_epoch=$(/bin/busybox date -u +%s)"));
        assert!(shell.contains("test $((now_epoch - lease_epoch)) -le 120"));
        assert!(shell.contains("test $((now_epoch - last_ok_epoch)) -le 300"));
        assert!(shell.contains("if [ ! -f \"$last_ok_file\" ] && [ -n \"$bootstrap_epoch\" ]"));
        assert!(shell.contains("test $((now_epoch - bootstrap_epoch)) -le 600"));
    }

    #[test]
    fn hosted_k3s_bootstrap_uses_native_snapshotter_for_overlay_rootfs_guests() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        for machine_name in ["cloud-aws", "cloud-aws-worker"] {
            let machine = config
                .machines
                .get_mut(machine_name)
                .expect("hosted machine should exist");
            machine.rootfs_read_only = true;
            machine.rootfs_overlay = Some(port_model::MachineRootfsOverlaySpec { size_mib: 64 });
        }
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let guest_path = config
            .artifacts
            .guest_images
            .get("demo-guest")
            .expect("demo-guest should exist")
            .variants
            .iter()
            .find(|variant| {
                variant.selector.architecture == MachineArchitecture::X86_64
                    && variant.selector.substrate == ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == ProtectionMode::Standard
            })
            .expect("standard guest variant should exist")
            .path
            .clone();
        fs::write(guest_path.with_file_name("initrd.cpio.gz"), b"initrd")
            .expect("overlay guest initrd should write");
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let worker_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let server_guest = spawn_hosted_guest_exec_server_with_optional_service_start(
            worker_paths,
            "k3s-server",
            k3s_bootstrap_command(
                "server",
                &[
                    String::from("--disable=traefik"),
                    String::from("--snapshotter=native"),
                    String::from("--node-name"),
                    String::from("cloud-aws"),
                    String::from("--node-external-ip"),
                    String::from("127.0.0.1"),
                    String::from("--flannel-external-ip"),
                ],
                Some("--cluster-init"),
                None,
                None,
            ),
            hosted_k3s_service_policy("server", "cloud-aws"),
            hosted_k3s_join_token_command(),
            String::from("demo-join-token\n"),
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let result = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        assert_eq!(result.join_token, "demo-join-token");

        server_guest
            .join()
            .expect("server guest thread should complete");

        let _ = Command::new("kill")
            .arg(result.server_launches[0].pid.to_string())
            .status();
        for metadata in result.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[test]
    fn hosted_k3s_bootstrap_persists_placement_and_service_records() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let hosted_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let server_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws");
        let worker_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws-worker");
        let server_guest = spawn_hosted_guest_exec_server_with_optional_service_start(
            server_paths.clone(),
            "k3s-server",
            k3s_bootstrap_command(
                "server",
                &[
                    String::from("--disable=traefik"),
                    String::from("--node-name"),
                    String::from("cloud-aws"),
                    String::from("--node-external-ip"),
                    String::from("127.0.0.1"),
                    String::from("--flannel-external-ip"),
                ],
                Some("--cluster-init"),
                None,
                None,
            ),
            hosted_k3s_service_policy("server", "cloud-aws"),
            hosted_k3s_join_token_command(),
            String::from("demo-join-token\n"),
        );
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");

        let placement_state_path =
            super::hosted_placeholder_runtime_root_for_config(&config, "demo")
                .join("machine-placements.json");
        let placement_state: serde_json::Value = serde_json::from_slice(
            &fs::read(&placement_state_path).expect("machine placement state should exist"),
        )
        .expect("machine placement state should decode");
        for machine_name in ["cloud-aws", "cloud-aws-worker"] {
            assert_eq!(
                placement_state["machines"][machine_name]["node_name"].as_str(),
                Some("aws-linux-node")
            );
            assert_eq!(
                placement_state["machines"][machine_name]["runtime_root"].as_str(),
                Some(hosted_runtime_root.to_string_lossy().as_ref())
            );
        }

        let server_record: ServiceDefinitionRecord = read_json_file(
            &service_definition_dir(&server_paths.runtime_dir).join("k3s-server.json"),
        )
        .expect("server service definition should persist");
        assert_eq!(server_record.machine_name, "cloud-aws");
        assert_eq!(server_record.name, "k3s-server");
        assert_eq!(server_record.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            server_record.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(server_record.desired_state, ServiceDesiredState::Active);

        let worker_record: ServiceDefinitionRecord = read_json_file(
            &service_definition_dir(&worker_paths.runtime_dir).join("k3s-agent.json"),
        )
        .expect("worker service definition should persist");
        assert_eq!(worker_record.machine_name, "cloud-aws-worker");
        assert_eq!(worker_record.name, "k3s-agent");
        assert_eq!(worker_record.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            worker_record.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(worker_record.desired_state, ServiceDesiredState::Active);

        server_guest
            .join()
            .expect("server guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_k3s_service_status_survives_from_persisted_records_after_launch() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let hosted_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let server_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws");
        let worker_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws-worker");
        let server_guest = spawn_hosted_guest_sequence_server(
            server_paths,
            vec![
                HostedGuestExpectedOperation::ManagedServiceStart {
                    name: String::from("k3s-server"),
                    command: k3s_bootstrap_command(
                        "server",
                        &[
                            String::from("--disable=traefik"),
                            String::from("--node-name"),
                            String::from("cloud-aws"),
                            String::from("--node-external-ip"),
                            String::from("127.0.0.1"),
                            String::from("--flannel-external-ip"),
                        ],
                        Some("--cluster-init"),
                        None,
                        None,
                    ),
                    policy: hosted_k3s_service_policy("server", "cloud-aws"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
            ],
        );
        let worker_guest = spawn_hosted_guest_sequence_server(
            worker_paths,
            vec![HostedGuestExpectedOperation::ManagedServiceStart {
                name: String::from("k3s-agent"),
                command: k3s_bootstrap_command(
                    "agent",
                    &[
                        String::from("--node-label=role=worker"),
                        String::from("--node-name"),
                        String::from("cloud-aws-worker"),
                        String::from("--node-external-ip"),
                        String::from("127.0.0.1"),
                        String::from("--flannel-external-ip"),
                    ],
                    None,
                    Some("https://demo-k3s.internal:6443"),
                    Some("demo-join-token"),
                ),
                policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
            }],
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");

        for (machine_name, service_name) in [
            ("cloud-aws", "k3s-server"),
            ("cloud-aws-worker", "k3s-agent"),
        ] {
            let running = (0..100).any(|attempt| {
                let status =
                    machine_service_status(&config, tempdir.path(), machine_name, service_name)
                        .expect("live hosted service status should succeed");
                if status.runtime.state == ServiceRuntimeState::Running {
                    true
                } else {
                    if attempt < 99 {
                        thread::sleep(Duration::from_millis(20));
                    }
                    false
                }
            });
            assert!(
                running,
                "hosted service runtime record should converge to running before stale fallback"
            );
        }

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let stale_control_plane = start_live_control_plane_with_bindings(&config, Vec::new())
            .expect("stale control plane should start");
        let mut stale_config = config.clone();
        stale_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{stale_control_plane}");

        let server_status =
            machine_service_status(&stale_config, tempdir.path(), "cloud-aws", "k3s-server")
                .expect("server service status should survive from persisted records");
        assert_eq!(server_status.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            server_status.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(server_status.desired_state, ServiceDesiredState::Active);
        assert_eq!(server_status.runtime.state, ServiceRuntimeState::Running);
        assert!(
            server_status
                .detail
                .contains("Stored runtime record returned because live refresh failed"),
            "{}",
            server_status.detail
        );

        let worker_status = machine_service_status(
            &stale_config,
            tempdir.path(),
            "cloud-aws-worker",
            "k3s-agent",
        )
        .expect("worker service status should survive from persisted records");
        assert_eq!(worker_status.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            worker_status.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(worker_status.desired_state, ServiceDesiredState::Active);
        assert_eq!(worker_status.runtime.state, ServiceRuntimeState::Running);
        assert!(
            worker_status
                .detail
                .contains("Stored runtime record returned because live refresh failed"),
            "{}",
            worker_status.detail
        );

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[test]
    fn hosted_k3s_machine_access_uses_loaded_config_state_root_when_cwd_differs() {
        let model_root = tempdir().expect("model root should exist");
        let cwd_root = tempdir().expect("cwd root should exist");
        let runtime_root = model_root.path().join("runtime");
        let config_path = model_root.path().join("port.toml");
        let config = sample_hosted_k3s_config(&runtime_root);

        fs::write(
            &config_path,
            config.to_toml_string().expect("config should encode"),
        )
        .expect("config path should write");

        let host_kit = config.nodes["aws-linux-node"].capabilities.pvm_lanes[0]
            .host_kit
            .clone()
            .expect("aws x86_64 PVM lane should define a host-kit");
        let mut imported_summary = config.nodes["aws-linux-node"].capabilities.clone();
        imported_summary.pvm_lanes[0].state = PvmCapabilityState::Ready;
        imported_summary.pvm_lanes[0].host_kit = Some(host_kit.clone());
        write_imported_inventory_state_at(
            model_root.path(),
            "demo",
            BTreeMap::from([(
                String::from("aws-linux-node"),
                HostedImportedNodeRecord {
                    provider: HostProvider::Aws,
                    provenance: String::from("inventory-sync"),
                    imported_at: 1,
                    capability_summary: imported_summary,
                    pvm_host_kit_packages: vec![port_model::HostedPvmHostKitPackageAttachment {
                        architecture: MachineArchitecture::X86_64,
                        package: host_kit.package,
                    }],
                },
            )]),
        );

        let loaded = PortConfig::from_path(&config_path).expect("config should load");
        let access = with_current_dir(cwd_root.path(), || {
            hosted_k3s_machine_access(
                &loaded,
                "demo",
                "demo",
                "aws-builders",
                "cloud-aws",
                "control-plane",
            )
        })
        .expect("hosted k3s machine access should resolve through config state root");

        assert_eq!(access.route.control_plane.as_deref(), Some("demo"));
        assert_eq!(access.route.node_name.as_deref(), Some("aws-linux-node"));
        assert!(
            access.detail.contains("selected node 'aws-linux-node'"),
            "{}",
            access.detail
        );
    }

    #[test]
    fn hosted_k3s_cluster_access_contract() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let worker_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_join_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            worker_paths,
            vec![
                HostedGuestExpectedOperation::ManagedServiceStart {
                    name: String::from("k3s-server"),
                    command: k3s_bootstrap_command(
                        "server",
                        &[
                            String::from("--disable=traefik"),
                            String::from("--node-name"),
                            String::from("cloud-aws"),
                            String::from("--node-external-ip"),
                            String::from("127.0.0.1"),
                            String::from("--flannel-external-ip"),
                        ],
                        Some("--cluster-init"),
                        None,
                        None,
                    ),
                    policy: hosted_k3s_service_policy("server", "cloud-aws"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_kubeconfig_command(),
                    stdout: String::from(
                        "apiVersion: v1\nclusters:\n- cluster:\n    server: https://demo-k3s.internal:6443\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_visibility_command(),
                    stdout: String::from(
                        "NAME              STATUS   ROLES                  AGE   VERSION\ncloud-aws         Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws-worker  Ready    <none>                 1m    v1.35.2+k3s1\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::new(),
                },
            ],
        );
        let worker_guest = spawn_hosted_guest_sequence_server(
            worker_join_paths,
            vec![HostedGuestExpectedOperation::ManagedServiceStart {
                name: String::from("k3s-agent"),
                command: k3s_bootstrap_command(
                    "agent",
                    &[
                        String::from("--node-label=role=worker"),
                        String::from("--node-name"),
                        String::from("cloud-aws-worker"),
                        String::from("--node-external-ip"),
                        String::from("127.0.0.1"),
                        String::from("--flannel-external-ip"),
                    ],
                    None,
                    Some("https://demo-k3s.internal:6443"),
                    Some("demo-join-token"),
                ),
                policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
            }],
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let report = hosted_k3s_cluster_access(&config, tempdir.path(), "demo")
            .expect("hosted k3s access should succeed");

        assert_eq!(report.cluster_name, "demo");
        assert_eq!(report.control_plane, "demo");
        assert_eq!(report.host_group, "aws-builders");
        assert_eq!(report.server_machines, vec![String::from("cloud-aws")]);
        assert_eq!(
            report.worker_machines,
            vec![String::from("cloud-aws-worker")]
        );
        assert_eq!(report.api_endpoint, "https://demo-k3s.internal:6443");
        assert_eq!(report.machines.len(), 2);
        assert_eq!(report.machines[0].role, "control-plane");
        assert_eq!(report.machines[0].machine_name, "cloud-aws");
        assert_eq!(
            report.machines[0].node_name.as_deref(),
            Some("aws-linux-node")
        );
        assert_eq!(report.machines[1].role, "worker");
        assert_eq!(report.machines[1].machine_name, "cloud-aws-worker");
        assert_eq!(report.managed_services.len(), 2);
        assert_eq!(report.managed_services[0].service_name, "k3s-server");
        assert_eq!(
            report.managed_services[0].state,
            super::HostedK3sManagedServiceTruthState::Running
        );
        assert_eq!(report.managed_services[0].pid, Some(4242));
        assert_eq!(report.managed_services[1].service_name, "k3s-agent");
        assert_eq!(
            report.managed_services[1].state,
            super::HostedK3sManagedServiceTruthState::Running
        );
        assert_eq!(
            report.stable_endpoint_posture,
            super::HostedK3sStableEndpointPosture::ManualRewriteRequired
        );
        assert!(
            report
                .stable_endpoint_detail
                .contains("stable endpoint posture is manual-rewrite-required"),
            "{}",
            report.stable_endpoint_detail
        );
        assert!(
            report
                .kubeconfig_surface
                .contains("port guest exec --machine cloud-aws"),
            "{}",
            report.kubeconfig_surface
        );
        assert_eq!(
            report.machine_runtime_readiness.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.api_readiness.state,
            super::HostedK3sReadinessState::Ready
        );
        assert!(
            report.api_surface.contains("get --raw=/readyz"),
            "{}",
            report.api_surface
        );
        assert_eq!(report.api_output, "ok");
        assert_eq!(
            report.kubeconfig_availability.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.node_visibility.state,
            super::HostedK3sReadinessState::Ready
        );
        assert!(report.kubeconfig.contains("apiVersion: v1"));
        assert!(
            report
                .visibility_surface
                .contains("k3s kubectl get nodes -o wide"),
            "{}",
            report.visibility_surface
        );
        assert!(report.visibility_output.contains("cloud-aws"));
        assert!(report.visibility_output.contains("cloud-aws"));
        assert!(report.visibility_output.contains("cloud-aws-worker"));
        assert_eq!(report.machine_access.len(), 2);
        assert_eq!(report.ha_status, super::HostedK3sHaStatus::NonHaTopology);
        assert_eq!(report.control_plane_placements.len(), 1);
        assert_eq!(
            report.control_plane_placements[0].machine_name,
            String::from("cloud-aws")
        );
        assert_eq!(
            report.control_plane_placements[0].node_name.as_deref(),
            Some("aws-linux-node")
        );
        assert!(
            report
                .ha_status_detail
                .contains("Hosted AWS x86_64 PVM real-HA status is non-ha-topology"),
            "{}",
            report.ha_status_detail
        );
        assert_eq!(
            report.legacy_runtime_drift,
            super::HostedK3sLegacyRuntimeDriftState::Clear
        );
        assert!(report.legacy_runtime_artifacts.is_empty());
        assert!(
            report
                .legacy_runtime_drift_detail
                .contains("legacy-runtime drift is clear"),
            "{}",
            report.legacy_runtime_drift_detail
        );
        assert!(
            report
                .boundary_notes
                .iter()
                .any(|note| note.contains("stateless"))
        );
        assert!(report.boundary_notes.iter().any(|note| note.contains("HA")));
        assert!(
            report
                .boundary_notes
                .iter()
                .any(|note| note.contains("ingress"))
        );

        let server = report
            .machine_access
            .iter()
            .find(|machine| machine.role == "control-plane")
            .expect("server route should exist");
        assert_eq!(server.route.control_plane.as_deref(), Some("demo"));
        assert_eq!(server.route.machine_name.as_deref(), Some("cloud-aws"));
        assert_eq!(server.route.node_name.as_deref(), Some("aws-linux-node"));
        assert!(
            server
                .route
                .host_groups
                .contains(&String::from("aws-builders"))
        );
        assert_eq!(
            server.route.candidate_nodes,
            vec![String::from("aws-linux-node")]
        );
        assert!(server.detail.contains("host group 'aws-builders'"));
        assert_eq!(
            server.network_identity.identity,
            "port-hosted://demo/nodes/aws-linux-node/machines/cloud-aws"
        );
        assert_eq!(
            server.network_identity.endpoint_ip,
            Some(std::net::IpAddr::from([127, 0, 0, 1]))
        );
        assert_eq!(
            server.network_identity.endpoint_scope,
            super::HostedK3sGuestNetworkEndpointScope::SharedPerExecutionHost
        );
        assert_eq!(
            server.network_identity.shared_with_machines,
            vec![String::from("cloud-aws-worker")]
        );

        let worker = report
            .machine_access
            .iter()
            .find(|machine| machine.role == "worker")
            .expect("worker route should exist");
        assert_eq!(
            worker.network_identity.identity,
            "port-hosted://demo/nodes/aws-linux-node/machines/cloud-aws-worker"
        );
        assert_eq!(
            worker.network_identity.endpoint_scope,
            super::HostedK3sGuestNetworkEndpointScope::SharedPerExecutionHost
        );
        assert_eq!(
            worker.network_identity.shared_with_machines,
            vec![String::from("cloud-aws")]
        );

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_k3s_cluster_access_reports_kubeconfig_handoff_separately() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let server_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            server_paths,
            vec![
                hosted_demo_server_start(),
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::ExecFailure {
                    command: hosted_k3s_kubeconfig_command(),
                    stderr: String::from("cat: /etc/rancher/k3s/k3s.yaml: No such file"),
                    exit_code: 1,
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_visibility_command(),
                    stdout: String::from(
                        "NAME              STATUS   ROLES                  AGE   VERSION\ncloud-aws         Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws-worker  Ready    <none>                 1m    v1.35.2+k3s1\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::new(),
                },
            ],
        );
        let worker_guest =
            spawn_hosted_guest_sequence_server(worker_paths, vec![hosted_demo_worker_start()]);

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let report = hosted_k3s_cluster_access(&config, tempdir.path(), "demo")
            .expect("hosted cluster status should preserve readiness gates");

        assert_eq!(
            report.machine_runtime_readiness.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.api_readiness.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.node_visibility.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.kubeconfig_availability.state,
            super::HostedK3sReadinessState::Unavailable
        );
        assert!(report.kubeconfig.is_empty());
        assert!(
            report
                .kubeconfig_availability
                .detail
                .contains("could not read '/etc/rancher/k3s/k3s.yaml'"),
            "{}",
            report.kubeconfig_availability.detail
        );

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_k3s_cluster_kubeconfig_succeeds_without_node_visibility() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let server_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            server_paths,
            vec![
                hosted_demo_server_start(),
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_kubeconfig_command(),
                    stdout: String::from(
                        "apiVersion: v1\nclusters:\n- cluster:\n    server: https://demo-k3s.internal:6443\n",
                    ),
                },
                HostedGuestExpectedOperation::ExecFailure {
                    command: hosted_k3s_visibility_command(),
                    stderr: String::from("timed out waiting for node list"),
                    exit_code: 1,
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::new(),
                },
            ],
        );
        let worker_guest =
            spawn_hosted_guest_sequence_server(worker_paths, vec![hosted_demo_worker_start()]);

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let report = hosted_k3s_cluster_kubeconfig(&config, tempdir.path(), "demo")
            .expect("kubeconfig handoff should not depend on node visibility");

        assert_eq!(
            report.kubeconfig_availability.state,
            super::HostedK3sReadinessState::Ready
        );
        assert_eq!(
            report.node_visibility.state,
            super::HostedK3sReadinessState::Unavailable
        );
        assert!(report.kubeconfig.contains("apiVersion: v1"));

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_k3s_cluster_kubeconfig_failure_preserves_readiness_summary() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let server_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            server_paths,
            vec![
                hosted_demo_server_start(),
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::ExecFailure {
                    command: hosted_k3s_kubeconfig_command(),
                    stderr: String::from("cat: /etc/rancher/k3s/k3s.yaml: No such file"),
                    exit_code: 1,
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_visibility_command(),
                    stdout: String::from(
                        "NAME              STATUS   ROLES                  AGE   VERSION\ncloud-aws         Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws-worker  Ready    <none>                 1m    v1.35.2+k3s1\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::new(),
                },
            ],
        );
        let worker_guest =
            spawn_hosted_guest_sequence_server(worker_paths, vec![hosted_demo_worker_start()]);

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let error = hosted_k3s_cluster_kubeconfig(&config, tempdir.path(), "demo")
            .expect_err("kubeconfig handoff should fail on the kubeconfig boundary");
        let message = error.to_string();

        assert!(
            message.contains("kubeconfig handoff is unavailable"),
            "{message}"
        );
        assert!(message.contains("machine-runtime=ready"), "{message}");
        assert!(message.contains("api=ready"), "{message}");
        assert!(message.contains("node-visibility=ready"), "{message}");
        assert!(message.contains("kubeconfig=unavailable"), "{message}");

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_k3s_cluster_access_reports_legacy_detached_runtime_drift() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let worker_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_join_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            worker_paths,
            vec![
                HostedGuestExpectedOperation::ManagedServiceStart {
                    name: String::from("k3s-server"),
                    command: k3s_bootstrap_command(
                        "server",
                        &[
                            String::from("--disable=traefik"),
                            String::from("--node-name"),
                            String::from("cloud-aws"),
                            String::from("--node-external-ip"),
                            String::from("127.0.0.1"),
                            String::from("--flannel-external-ip"),
                        ],
                        Some("--cluster-init"),
                        None,
                        None,
                    ),
                    policy: hosted_k3s_service_policy("server", "cloud-aws"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_kubeconfig_command(),
                    stdout: String::from(
                        "apiVersion: v1\nclusters:\n- cluster:\n    server: https://demo-k3s.internal:6443\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_visibility_command(),
                    stdout: String::from(
                        "NAME              STATUS   ROLES                  AGE   VERSION\ncloud-aws         Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws-worker  Ready    <none>                 1m    v1.35.2+k3s1\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::from("/run/port/k3s-server.pid\n/var/log/k3s-server.log\n"),
                },
            ],
        );
        let worker_guest = spawn_hosted_guest_sequence_server(
            worker_join_paths,
            vec![HostedGuestExpectedOperation::ManagedServiceStart {
                name: String::from("k3s-agent"),
                command: k3s_bootstrap_command(
                    "agent",
                    &[
                        String::from("--node-label=role=worker"),
                        String::from("--node-name"),
                        String::from("cloud-aws-worker"),
                        String::from("--node-external-ip"),
                        String::from("127.0.0.1"),
                        String::from("--flannel-external-ip"),
                    ],
                    None,
                    Some("https://demo-k3s.internal:6443"),
                    Some("demo-join-token"),
                ),
                policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
            }],
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let report = hosted_k3s_cluster_access(&config, tempdir.path(), "demo")
            .expect("hosted k3s access should succeed");

        assert_eq!(
            report.legacy_runtime_drift,
            super::HostedK3sLegacyRuntimeDriftState::DetachedRuntimeDetected
        );
        assert_eq!(report.legacy_runtime_artifacts.len(), 2);
        assert_eq!(report.legacy_runtime_artifacts[0].machine_name, "cloud-aws");
        assert_eq!(
            report.legacy_runtime_artifacts[0].path,
            "/run/port/k3s-server.pid"
        );
        assert_eq!(
            report.legacy_runtime_artifacts[1].path,
            "/var/log/k3s-server.log"
        );
        assert!(
            report
                .legacy_runtime_drift_detail
                .contains("detached-runtime-detected"),
            "{}",
            report.legacy_runtime_drift_detail
        );
        assert!(
            report
                .legacy_runtime_drift_detail
                .contains("/run/port/services/*"),
            "{}",
            report.legacy_runtime_drift_detail
        );

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[test]
    fn hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts() {
        let cluster = port_model::K3sClusterSpec {
            control_plane: String::from("demo"),
            host_group: String::from("aws-builders"),
            server_machines: vec![
                String::from("cloud-aws"),
                String::from("cloud-aws-b"),
                String::from("cloud-aws-c"),
            ],
            worker_machines: Vec::new(),
            api_endpoint: String::from("https://demo-k3s.internal:6443"),
            control_plane_scheduler: port_model::HostedSchedulerPolicy::Spread,
            version: Some(String::from("v1.35.2+k3s1")),
            server_args: vec![String::from("--disable=traefik")],
            worker_args: Vec::new(),
        };
        let placements = vec![
            super::HostedK3sControlPlanePlacement {
                machine_name: String::from("cloud-aws"),
                node_name: Some(String::from("aws-linux-node")),
                runtime_root: Some(PathBuf::from("/tmp/aws-linux-node")),
                detail: String::from("primary"),
            },
            super::HostedK3sControlPlanePlacement {
                machine_name: String::from("cloud-aws-b"),
                node_name: Some(String::from("aws-linux-node-b")),
                runtime_root: Some(PathBuf::from("/tmp/aws-linux-node-b")),
                detail: String::from("secondary"),
            },
            super::HostedK3sControlPlanePlacement {
                machine_name: String::from("cloud-aws-c"),
                node_name: Some(String::from("aws-linux-node-c")),
                runtime_root: Some(PathBuf::from("/tmp/aws-linux-node-c")),
                detail: String::from("tertiary"),
            },
        ];

        assert_eq!(
            super::hosted_k3s_ha_status(&cluster, &placements),
            super::HostedK3sHaStatus::SpreadSatisfied
        );
        let detail = super::hosted_k3s_ha_status_detail(&cluster, &placements);
        assert!(detail.contains("Hosted AWS x86_64 PVM"));
        assert!(detail.contains("spread-satisfied"));
        assert!(detail.contains("3 distinct execution hosts"));

        assert_eq!(
            super::hosted_k3s_access_stable_endpoint_posture(
                super::HostedK3sHaStatus::SpreadSatisfied
            ),
            super::HostedK3sStableEndpointPosture::HaEligible
        );
        let endpoint_detail = super::hosted_k3s_access_stable_endpoint_detail(
            &cluster,
            super::HostedK3sHaStatus::SpreadSatisfied,
        );
        assert!(endpoint_detail.contains("stable endpoint posture is ha-eligible"));
        assert!(endpoint_detail.contains("https://demo-k3s.internal:6443"));
        assert!(endpoint_detail.contains("Supported failover condition"));
    }

    #[test]
    fn hosted_k3s_boundary_failures() {
        let tempdir = tempdir().expect("tempdir should exist");

        let mut local_route = sample_hosted_k3s_config(tempdir.path());
        local_route
            .k3s_clusters
            .get_mut("demo")
            .expect("demo cluster should exist")
            .server_machines[0] = String::from("demo");
        let local_error = hosted_k3s_cluster_access(&local_route, tempdir.path(), "demo")
            .expect_err("non-hosted K3s route should fail fast");
        let local_message = local_error.to_string();
        assert!(
            local_message.contains("must target a hosted control plane"),
            "{local_message}"
        );
        assert!(local_message.contains("HA"), "{local_message}");
        assert!(local_message.contains("load balancer"), "{local_message}");

        let mut persistent = sample_hosted_k3s_config(tempdir.path());
        let volume_path = tempdir.path().join("k3s-data.ext4");
        fs::write(&volume_path, b"k3s-data").expect("volume should write");
        persistent
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .volumes = vec![MachineVolumeSpec {
            name: String::from("data"),
            backend: MachineVolumeBackend::HostFile,
            persistence: MachineVolumePersistence::Persistent,
            path: volume_path,
        }];
        let persistent_error = hosted_k3s_cluster_access(&persistent, tempdir.path(), "demo")
            .expect_err("persistent K3s route should fail fast");
        let persistent_message = persistent_error.to_string();
        assert!(
            persistent_message.contains("attached volumes are only supported"),
            "{persistent_message}"
        );
        assert!(
            persistent_message.contains("persistent"),
            "{persistent_message}"
        );

        let mut no_capacity = sample_hosted_k3s_config(tempdir.path());
        no_capacity
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .substrates = vec![ExecutionSubstrate::CloudHypervisor];
        let capacity_error = hosted_k3s_cluster_access(&no_capacity, tempdir.path(), "demo")
            .expect_err("missing hosted K3s placement capacity should fail fast");
        let capacity_message = capacity_error.to_string();
        assert!(
            capacity_message.contains("no hosted placement candidates"),
            "{capacity_message}"
        );
        assert!(
            capacity_message.contains("placement capacity"),
            "{capacity_message}"
        );
        assert!(
            capacity_message.contains("aws-linux-node"),
            "{capacity_message}"
        );
    }

    #[test]
    fn hosted_k3s_route_context_visibility() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_hosted_k3s_config(tempdir.path());
        config.nodes.insert(
            String::from("aws-shadow-node"),
            HostedNodeSpec {
                host: String::from("aws-linux"),
                runtime_root: tempdir.path().join("runtime/hosted/aws-shadow-node"),
                capabilities: HostedNodeCapabilities {
                    providers: vec![HostProvider::Aws],
                    platforms: vec![HostPlatform::Linux],
                    substrates: vec![ExecutionSubstrate::CloudHypervisor],
                    architectures: vec![MachineArchitecture::X86_64],
                    protection_modes: vec![ProtectionMode::Standard],
                    pvm_lanes: Vec::new(),
                },
                notes: vec![String::from(
                    "Shadow hosted node exists to keep rejected-node detail visible in route rendering tests.",
                )],
            },
        );
        config
            .host_groups
            .get_mut("aws-builders")
            .expect("aws-builders should exist")
            .nodes
            .push(String::from("aws-shadow-node"));
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let worker_paths =
            RuntimePaths::for_machine(&config.nodes["aws-linux-node"].runtime_root, "cloud-aws");
        let worker_join_paths = RuntimePaths::for_machine(
            &config.nodes["aws-linux-node"].runtime_root,
            "cloud-aws-worker",
        );
        let server_guest = spawn_hosted_guest_sequence_server(
            worker_paths,
            vec![
                HostedGuestExpectedOperation::ManagedServiceStart {
                    name: String::from("k3s-server"),
                    command: k3s_bootstrap_command(
                        "server",
                        &[
                            String::from("--disable=traefik"),
                            String::from("--node-name"),
                            String::from("cloud-aws"),
                            String::from("--node-external-ip"),
                            String::from("127.0.0.1"),
                            String::from("--flannel-external-ip"),
                        ],
                        Some("--cluster-init"),
                        None,
                        None,
                    ),
                    policy: hosted_k3s_service_policy("server", "cloud-aws"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_join_token_command(),
                    stdout: String::from("demo-join-token\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_api_readiness_command(),
                    stdout: String::from("ok\n"),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_kubeconfig_command(),
                    stdout: String::from(
                        "apiVersion: v1\nclusters:\n- cluster:\n    server: https://demo-k3s.internal:6443\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: hosted_k3s_visibility_command(),
                    stdout: String::from(
                        "NAME              STATUS   ROLES                  AGE   VERSION\ncloud-aws         Ready    control-plane,master   1m    v1.35.2+k3s1\ncloud-aws-worker  Ready    <none>                 1m    v1.35.2+k3s1\n",
                    ),
                },
                HostedGuestExpectedOperation::Exec {
                    command: super::hosted_k3s_legacy_runtime_drift_command(),
                    stdout: String::new(),
                },
            ],
        );
        let worker_guest = spawn_hosted_guest_sequence_server(
            worker_join_paths,
            vec![HostedGuestExpectedOperation::ManagedServiceStart {
                name: String::from("k3s-agent"),
                command: k3s_bootstrap_command(
                    "agent",
                    &[
                        String::from("--node-label=role=worker"),
                        String::from("--node-name"),
                        String::from("cloud-aws-worker"),
                        String::from("--node-external-ip"),
                        String::from("127.0.0.1"),
                        String::from("--flannel-external-ip"),
                    ],
                    None,
                    Some("https://demo-k3s.internal:6443"),
                    Some("demo-join-token"),
                ),
                policy: hosted_k3s_service_policy("agent", "cloud-aws-worker"),
            }],
        );

        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let bootstrap = bootstrap_hosted_k3s_cluster(&config, tempdir.path(), "demo")
            .expect("hosted k3s bootstrap should succeed");
        let report = hosted_k3s_cluster_access(&config, tempdir.path(), "demo")
            .expect("hosted k3s access should succeed");
        let server = report
            .machine_access
            .iter()
            .find(|machine| machine.role == "control-plane")
            .expect("server route should exist");
        let rendered = render_hosted_route_context(Some(&server.route));
        assert!(rendered.contains("control-plane=demo"), "{rendered}");
        assert!(rendered.contains("machine=cloud-aws"), "{rendered}");
        assert!(rendered.contains("node=aws-linux-node"), "{rendered}");
        assert!(rendered.contains("host-groups="), "{rendered}");
        assert!(rendered.contains("aws-builders"), "{rendered}");
        assert!(
            rendered.contains("candidate-nodes=aws-linux-node"),
            "{rendered}"
        );
        assert!(
            rendered.contains("rejected-nodes=aws-shadow-node"),
            "{rendered}"
        );
        assert!(rendered.contains("placement="), "{rendered}");
        assert!(!rendered.contains("session="), "{rendered}");

        server_guest
            .join()
            .expect("server guest thread should complete");
        worker_guest
            .join()
            .expect("worker guest thread should complete");

        let _ = Command::new("kill")
            .arg(bootstrap.server_launches[0].pid.to_string())
            .status();
        for metadata in bootstrap.worker_launches {
            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[test]
    fn hosted_copy_uses_stream_route_and_round_trips_bytes() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        async fn handler(body: axum::body::Bytes) -> impl axum::response::IntoResponse {
            let mut reader = BufReader::new(Cursor::new(body.to_vec()));
            let request: RequestEnvelope =
                read_frame(&mut reader).expect("copy stream request should decode");
            let GuestOperation::Copy(request) = request.operation else {
                panic!("unexpected stream request");
            };

            let mut response = Vec::new();
            match request.direction {
                CopyDirection::HostToGuest => {
                    let size_bytes = request.size_bytes.expect("upload size should exist");
                    let mut uploaded = Vec::new();
                    reader
                        .by_ref()
                        .take(size_bytes)
                        .read_to_end(&mut uploaded)
                        .expect("upload bytes should read");
                    assert_eq!(uploaded, b"copy-ok");
                    write_frame(
                        &mut response,
                        &ResponseEnvelope::Accepted {
                            id: 1,
                            stream: StreamKind::Bytes,
                            size_bytes: None,
                        },
                    )
                    .expect("upload accepted should encode");
                    write_frame(
                        &mut response,
                        &ResponseEnvelope::Completed {
                            id: 1,
                            exit_code: 0,
                            result: OperationResult::Copy(port_agent_protocol::CopyResult {
                                bytes_copied: size_bytes,
                                path: request.destination,
                                direction: CopyDirection::HostToGuest,
                            }),
                        },
                    )
                    .expect("upload completion should encode");
                }
                CopyDirection::GuestToHost => {
                    write_frame(
                        &mut response,
                        &ResponseEnvelope::Accepted {
                            id: 1,
                            stream: StreamKind::Bytes,
                            size_bytes: Some(7),
                        },
                    )
                    .expect("download accepted should encode");
                    response.extend_from_slice(b"copy-ok");
                    write_frame(
                        &mut response,
                        &ResponseEnvelope::Completed {
                            id: 1,
                            exit_code: 0,
                            result: OperationResult::Copy(port_agent_protocol::CopyResult {
                                bytes_copied: 7,
                                path: request.destination,
                                direction: CopyDirection::GuestToHost,
                            }),
                        },
                    )
                    .expect("download completion should encode");
                }
            }

            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
                response,
            )
        }

        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router =
                    Router::new().route("/v1/machines/{machine}/guest:copy:stream", post(handler));
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let source = tempdir.path().join("upload.txt");
        fs::write(&source, "copy-ok").expect("source should write");
        let download = tempdir.path().join("download.txt");

        let upload = copy_guest_file(
            &config,
            GuestCopyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                source: &source,
                destination: Path::new("/workspace/copied.txt"),
                direction: CopyDirection::HostToGuest,
            },
        )
        .expect("hosted upload should succeed");
        assert_eq!(upload.bytes_copied, 7);
        assert_eq!(upload.path, "/workspace/copied.txt");

        let download_result = copy_guest_file(
            &config,
            GuestCopyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                source: Path::new("/workspace/copied.txt"),
                destination: &download,
                direction: CopyDirection::GuestToHost,
            },
        )
        .expect("hosted download should succeed");
        assert_eq!(download_result.bytes_copied, 7);
        assert_eq!(download_result.path, download.display().to_string());
        assert_eq!(
            fs::read_to_string(&download).expect("download should read"),
            "copy-ok"
        );
    }

    #[test]
    fn hosted_copy_stream_errors_include_route_context() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        async fn handler(AxumPath(machine): AxumPath<String>) -> (StatusCode, Json<HostedError>) {
            (
                StatusCode::BAD_GATEWAY,
                Json(HostedError {
                    route: Some(HostedRouteContext {
                        control_plane: Some(String::from("demo")),
                        machine_name: Some(machine),
                        node_name: Some(String::from("aws-linux-node")),
                        runtime_root: Some(PathBuf::from(
                            "/runtime/hosted/aws-linux-node/cloud-aws",
                        )),
                        guest_session: Some(port_hosted_protocol::HostedGuestSessionContract {
                            id: String::from("port-hosted://demo/machines/cloud-aws/guest-session"),
                            scope: port_hosted_protocol::HostedGuestSessionScope::Machine,
                            driver: port_hosted_protocol::HostedShellDriverContract {
                                id: String::from("port-guest-shell-driver-v1"),
                                route: port_model::MachineCommandRoute::HostedControlPlane,
                                broker: port_model::MachineGuestBroker::ControlPlaneNodeAgentTunnel,
                                protocol:
                                    port_model::HostedGuestProtocolContract::PortAgentProtocol,
                                command_surface: vec![
                                    port_model::GuestCommandVerb::Exec,
                                    port_model::GuestCommandVerb::Copy,
                                    port_model::GuestCommandVerb::Pty,
                                    port_model::GuestCommandVerb::Logs,
                                    port_model::GuestCommandVerb::Forward,
                                ],
                            },
                        }),
                        ..HostedRouteContext::default()
                    }),
                    message: String::from("stream route deliberately unavailable"),
                }),
            )
        }

        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router =
                    Router::new().route("/v1/machines/{machine}/guest:copy:stream", post(handler));
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let source = tempdir.path().join("upload.txt");
        fs::write(&source, "copy-ok").expect("source should write");
        let error = copy_guest_file(
            &config,
            GuestCopyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                source: &source,
                destination: Path::new("/workspace/copied.txt"),
                direction: CopyDirection::HostToGuest,
            },
        )
        .expect_err("hosted copy should surface the stream-route failure");

        let message = error.to_string();
        assert!(message.contains("guest:copy:stream"), "{message}");
        assert!(
            message.contains("stream route deliberately unavailable"),
            "{message}"
        );
        assert!(message.contains("control-plane=demo"), "{message}");
        assert!(message.contains("machine=cloud-aws"), "{message}");
        assert!(message.contains("node=aws-linux-node"), "{message}");
        assert!(
            message.contains("session=port-hosted://demo/machines/cloud-aws/guest-session"),
            "{message}"
        );
        assert!(
            message.contains("driver=port-guest-shell-driver-v1"),
            "{message}"
        );
        assert!(
            message.contains("driver-route=hosted-control-plane"),
            "{message}"
        );
        assert!(
            message.contains("driver-broker=control-plane-node-agent-tunnel"),
            "{message}"
        );
        assert!(
            message.contains("/runtime/hosted/aws-linux-node/cloud-aws"),
            "{message}"
        );
    }

    #[test]
    fn hosted_guest_forward_routes_through_live_control_plane() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("forward accept");
            let reader_stream = stream.try_clone().expect("forward clone");
            let mut reader = BufReader::new(reader_stream);
            let request: RequestEnvelope = read_frame(&mut reader).expect("forward request");
            let GuestOperation::Forward(request) = request.operation else {
                panic!("unexpected hosted forward operation");
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
            let mut echoed = [0_u8; 32];
            let len = reader.read(&mut echoed).expect("forward bytes should read");
            stream
                .write_all(&echoed[..len])
                .expect("forward bytes should echo");
        });

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        let result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Forward(ForwardRequest {
                    listen: String::from("127.0.0.1:0"),
                    target: String::from("127.0.0.1:8081"),
                }),
            },
        )
        .expect("hosted guest forward should succeed");

        let OperationResult::Forward(result) = result else {
            panic!("unexpected hosted forward result: {result:?}");
        };
        let mut forwarded = None;
        for _ in 0..500 {
            match TcpStream::connect(&result.listen) {
                Ok(stream) => {
                    forwarded = Some(stream);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
        let mut forwarded = forwarded.expect("should connect to hosted forwarded listener");
        let mut eager = [0_u8; 5];
        forwarded
            .read_exact(&mut eager)
            .expect("forward eager bytes should read");
        assert_eq!(&eager, b"ready");
        forwarded
            .write_all(b"hosted-forward-ok")
            .expect("forward write");
        forwarded
            .shutdown(Shutdown::Write)
            .expect("forward shutdown");
        let mut echoed = Vec::new();
        forwarded
            .read_to_end(&mut echoed)
            .expect("forward read should complete");
        assert_eq!(echoed, b"hosted-forward-ok");

        server
            .join()
            .expect("hosted forward server thread should complete");
    }

    #[test]
    fn hosted_guest_forward_errors_include_route_context() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        async fn handler(AxumPath(machine): AxumPath<String>) -> (StatusCode, Json<HostedError>) {
            (
                StatusCode::BAD_GATEWAY,
                Json(HostedError {
                    route: Some(HostedRouteContext {
                        control_plane: Some(String::from("demo")),
                        machine_name: Some(machine),
                        node_name: Some(String::from("aws-linux-node")),
                        runtime_root: Some(PathBuf::from(
                            "/runtime/hosted/aws-linux-node/cloud-aws",
                        )),
                        guest_session: Some(port_hosted_protocol::HostedGuestSessionContract {
                            id: String::from("port-hosted://demo/machines/cloud-aws/guest-session"),
                            scope: port_hosted_protocol::HostedGuestSessionScope::Machine,
                            driver: port_hosted_protocol::HostedShellDriverContract {
                                id: String::from("port-guest-shell-driver-v1"),
                                route: port_model::MachineCommandRoute::HostedControlPlane,
                                broker: port_model::MachineGuestBroker::ControlPlaneNodeAgentTunnel,
                                protocol:
                                    port_model::HostedGuestProtocolContract::PortAgentProtocol,
                                command_surface: vec![
                                    port_model::GuestCommandVerb::Exec,
                                    port_model::GuestCommandVerb::Copy,
                                    port_model::GuestCommandVerb::Pty,
                                    port_model::GuestCommandVerb::Logs,
                                    port_model::GuestCommandVerb::Forward,
                                ],
                            },
                        }),
                        ..HostedRouteContext::default()
                    }),
                    message: String::from("forward route deliberately unavailable"),
                }),
            )
        }

        let listener = StdTcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime should build");
            runtime.block_on(async move {
                let listener =
                    TcpListener::from_std(listener).expect("listener should convert to tokio");
                let router =
                    Router::new().route("/v1/machines/{machine}/guest:forward", post(handler));
                let _ = axum::serve(listener, router).await;
            });
        });

        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{addr}");

        let error = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Forward(ForwardRequest {
                    listen: String::from("127.0.0.1:0"),
                    target: String::from("127.0.0.1:8081"),
                }),
            },
        )
        .expect_err("hosted guest forward should surface the hosted route failure");

        let message = error.to_string();
        assert!(message.contains("guest:forward"), "{message}");
        assert!(
            message.contains("forward route deliberately unavailable"),
            "{message}"
        );
        assert!(message.contains("control-plane=demo"), "{message}");
        assert!(message.contains("machine=cloud-aws"), "{message}");
        assert!(message.contains("node=aws-linux-node"), "{message}");
        assert!(
            message.contains("session=port-hosted://demo/machines/cloud-aws/guest-session"),
            "{message}"
        );
        assert!(
            message.contains("driver=port-guest-shell-driver-v1"),
            "{message}"
        );
        assert!(
            message.contains("driver-route=hosted-control-plane"),
            "{message}"
        );
        assert!(
            message.contains("driver-broker=control-plane-node-agent-tunnel"),
            "{message}"
        );
        assert!(
            message.contains("/runtime/hosted/aws-linux-node/cloud-aws"),
            "{message}"
        );
    }

    #[test]
    fn hosted_detached_forward_start_returns_node_owned_manifest() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let _guest_listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        let client = port_sdk::HostedClient::from_machine(&config, "cloud-aws", "demo-token")
            .expect("hosted client should resolve");
        let response: HostedSuccess<HostedDetachedForwardStatusContract> = client
            .execute_json(
                client
                    .guest()
                    .forward_detached_start(
                        "cloud-aws",
                        HostedDetachedForwardStartRequest {
                            listen: String::from("127.0.0.1:0"),
                            target: String::from("127.0.0.1:8081"),
                            name: Some(String::from("demo-web")),
                        },
                    )
                    .expect("detached start request should encode"),
            )
            .expect("hosted detached forward start should succeed");

        assert_eq!(response.route.control_plane.as_deref(), Some("demo"));
        assert_eq!(response.route.machine_name.as_deref(), Some("cloud-aws"));
        assert_eq!(response.route.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(response.route.forward_name.as_deref(), Some("demo-web"));
        assert_eq!(response.result.name, "demo-web");
        assert_eq!(response.result.state, HostedDetachedForwardState::Running);
        assert!(response.result.pid.is_some());
        assert!(response.result.manifest_path.exists());
        assert_eq!(
            response.result.manifest_path,
            paths.runtime_dir.join("forwards/demo-web.json")
        );
        let forward_config =
            fs::read_to_string(paths.runtime_dir.join("forwards/demo-web.config.toml"))
                .expect("detached forward config should exist");
        assert!(
            !forward_config.contains("[clusters.demo]"),
            "{forward_config}"
        );
        assert!(
            !forward_config.contains("[k3s_clusters.demo]"),
            "{forward_config}"
        );

        let status = Command::new("kill")
            .args([
                "-TERM",
                &response.result.pid.expect("pid should exist").to_string(),
            ])
            .status()
            .expect("detached forward should stop");
        assert!(status.success());
    }

    #[test]
    fn hosted_detached_forward_list_and_stop_use_node_runtime_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let _guest_listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");
        let listen_socket = tempdir.path().join("hosted-forward.sock");

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        let client = port_sdk::HostedClient::from_machine(&config, "cloud-aws", "demo-token")
            .expect("hosted client should resolve");
        let start: HostedSuccess<HostedDetachedForwardStatusContract> = client
            .execute_json(
                client
                    .guest()
                    .forward_detached_start(
                        "cloud-aws",
                        HostedDetachedForwardStartRequest {
                            listen: format!("unix:{}", listen_socket.display()),
                            target: String::from("unix:/var/run/demo.sock"),
                            name: Some(String::from("demo-sock")),
                        },
                    )
                    .expect("detached start request should encode"),
            )
            .expect("hosted detached forward start should succeed");

        let listed: HostedSuccess<Vec<HostedDetachedForwardStatusContract>> = client
            .execute_json(client.guest().forward_detached_list("cloud-aws"))
            .expect("hosted detached forward list should succeed");
        assert_eq!(listed.result.len(), 1);
        assert_eq!(listed.result[0].name, "demo-sock");
        assert_eq!(listed.result[0].state, HostedDetachedForwardState::Running);
        assert!(listen_socket.exists());

        let stopped: HostedSuccess<HostedDetachedForwardStopResult> = client
            .execute_json(
                client
                    .guest()
                    .forward_detached_stop("cloud-aws", "demo-sock"),
            )
            .expect("hosted detached forward stop should succeed");
        assert_eq!(stopped.route.forward_name.as_deref(), Some("demo-sock"));
        assert_eq!(stopped.result.name, "demo-sock");
        assert_eq!(stopped.result.state, HostedDetachedForwardState::Stopped);
        assert!(!start.result.manifest_path.exists());
        assert!(!listen_socket.exists());

        let listed_again: HostedSuccess<Vec<HostedDetachedForwardStatusContract>> = client
            .execute_json(client.guest().forward_detached_list("cloud-aws"))
            .expect("hosted detached forward list should succeed after stop");
        assert!(listed_again.result.is_empty());
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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

        let status = machine_status(&config, tempdir.path(), "cloud-generic")
            .expect("hosted pvm status should load");

        assert_eq!(status.state, MachineRuntimeState::Malformed);
        assert!(status.detail.contains("generic-linux-node"));
        assert!(status.detail.contains("planned"));
        assert!(status.detail.contains("PVM"));
    }

    #[test]
    fn hosted_aws_pvm_status_surfaces_preparation_guidance() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted aws pvm status should load");

        assert_eq!(status.state, MachineRuntimeState::Malformed);
        assert!(status.detail.contains("cloud-aws"));
        assert!(status.detail.contains("aws-linux-node"));
        assert!(status.detail.contains("provider 'aws'"));
        assert!(status.detail.contains("prepare-pvm-node"));
        assert!(!status.detail.contains("generic-linux-node"));
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
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
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

        let host_kit = {
            let host_kit = config
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
            host_kit.firecracker_binary_env =
                Some(String::from("PORT_TEST_HOSTED_PVM_FIRECRACKER"));
            host_kit.clone()
        };
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws node should exist")
            .capabilities
            .pvm_lanes[0]
            .host_kit = Some(host_kit.clone());
        let package = host_kit.package.clone();
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "firecracker-pvm");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        unsafe {
            std::env::set_var("PORT_TEST_HOSTED_PVM_FIRECRACKER", &fake_binary);
        }

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        let prepared = crate::prepare_hosted_pvm_node(
            &config,
            crate::HostedPvmNodePrepareRequest {
                control_plane: String::from("demo"),
                node_name: String::from("aws-linux-node"),
                architecture: MachineArchitecture::X86_64,
                provenance: String::from("inventory/aws-linux-node.json"),
                package,
            },
        )
        .expect("aws hosted PVM preparation should succeed");
        assert_eq!(prepared.provenance, "inventory/aws-linux-node.json");
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
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_pvm_status_stop_route_through_live_control_plane_and_prepared_node() {
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
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

        let host_kit = {
            let host_kit = config
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
            host_kit.firecracker_binary_env =
                Some(String::from("PORT_TEST_HOSTED_PVM_FIRECRACKER"));
            host_kit.clone()
        };
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws node should exist")
            .capabilities
            .pvm_lanes[0]
            .host_kit = Some(host_kit.clone());
        let package = host_kit.package.clone();
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "firecracker-pvm");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        unsafe {
            std::env::set_var("PORT_TEST_HOSTED_PVM_FIRECRACKER", &fake_binary);
        }

        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");
        crate::prepare_hosted_pvm_node(
            &config,
            crate::HostedPvmNodePrepareRequest {
                control_plane: String::from("demo"),
                node_name: String::from("aws-linux-node"),
                architecture: MachineArchitecture::X86_64,
                provenance: String::from("inventory/aws-linux-node.json"),
                package,
            },
        )
        .expect("aws hosted PVM preparation should succeed");
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

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted pvm status should load");
        assert_eq!(status.machine_name, "cloud-aws");
        assert_eq!(status.state, MachineRuntimeState::Running);
        assert_eq!(
            status.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(
            status.detail.contains("control plane 'demo'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("node 'aws-linux-node'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("provider 'aws'"),
            "{}",
            status.detail
        );

        let stop = stop_machine(&config, tempdir.path(), "cloud-aws", Duration::from_secs(1))
            .expect("hosted pvm stop should succeed");
        assert_eq!(stop.machine_name, "cloud-aws");
        assert_eq!(stop.previous_state, MachineRuntimeState::Running);
        assert_eq!(stop.current_state, MachineRuntimeState::Stopped);
        assert_eq!(
            stop.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(
            stop.detail.contains("control plane 'demo'"),
            "{}",
            stop.detail
        );
        assert!(
            stop.detail.contains("node 'aws-linux-node'"),
            "{}",
            stop.detail
        );
        assert!(stop.detail.contains("provider 'aws'"), "{}", stop.detail);
        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
    }

    #[test]
    fn hosted_standard_launch_routes_through_live_control_plane_for_each_provider() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        let config = start_named_live_hosted_servers_inner(
            &config,
            &["generic-linux-node", "aws-linux-node", "gcp-linux-node"],
        )
        .expect("hosted servers should start");
        let placement_state_path =
            hosted_placeholder_runtime_root("demo").join("machine-placements.json");

        for (machine_name, node_name) in [
            ("cloud-generic", "generic-linux-node"),
            ("cloud-aws", "aws-linux-node"),
            ("cloud-gcp", "gcp-linux-node"),
        ] {
            let metadata = launch_local_machine(
                &config,
                &LaunchRequest {
                    machine_name,
                    runtime_root: tempdir.path(),
                    boot_wait: Duration::from_secs(0),
                },
            )
            .unwrap_or_else(|error| {
                panic!("standard hosted launch for {machine_name} should succeed: {error}")
            });

            let expected_runtime_root = config.nodes[node_name].runtime_root.clone();
            let expected_paths = RuntimePaths::for_machine(&expected_runtime_root, machine_name);
            assert_eq!(metadata.machine_name, machine_name);
            assert_eq!(metadata.firecracker_binary, fake_binary);
            assert_eq!(metadata.runtime_dir, expected_paths.runtime_dir);
            assert_eq!(metadata.manifest_path, expected_paths.manifest_path);

            let placement_state: serde_json::Value = serde_json::from_slice(
                &fs::read(&placement_state_path).expect("machine placement state should exist"),
            )
            .expect("machine placement state should decode");
            assert_eq!(
                placement_state["machines"][machine_name]["node_name"].as_str(),
                Some(node_name)
            );
            assert_eq!(
                placement_state["machines"][machine_name]["runtime_root"].as_str(),
                Some(expected_runtime_root.to_string_lossy().as_ref())
            );

            let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
        }
    }

    #[test]
    fn hosted_standard_launch_precreates_firecracker_log_file() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let fake_binary = write_log_asserting_firecracker_binary(tempdir.path(), "firecracker");
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let metadata = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect("hosted standard launch should create the firecracker log path first");

        assert_eq!(metadata.firecracker_binary, fake_binary);
        assert!(
            metadata.log_path.exists(),
            "{}",
            metadata.log_path.display()
        );

        let _ = Command::new("kill").arg(metadata.pid.to_string()).status();
    }

    #[test]
    fn hosted_standard_launch_errors_surface_provider_and_selected_node_context() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("standard hosted launch should fail when firecracker is missing");

        let message = error.to_string();
        assert!(message.contains("cloud-aws"), "{message}");
        assert!(message.contains("control-plane=demo"), "{message}");
        assert!(message.contains("node=aws-linux-node"), "{message}");
        assert!(message.contains("placement="), "{message}");
        assert!(message.contains("provider 'aws'"), "{message}");
        assert!(message.contains("host 'aws-linux'"), "{message}");
        assert!(
            !message.contains("Run Port on the AWS Linux host itself."),
            "{message}"
        );
    }

    #[test]
    fn hosted_standard_status_stop_include_provider_and_hosted_node_detail() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let metadata = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect("standard hosted launch should succeed");
        assert_eq!(metadata.firecracker_binary, fake_binary);
        let placement_detail = config
            .hosted_machine_summary_contract("cloud-aws")
            .expect("summary should resolve")
            .expect("summary should exist")
            .placement_detail;
        let stored_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node",
            &stored_runtime_root,
            &placement_detail,
        );

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted standard status should load");
        assert_eq!(status.machine_name, "cloud-aws");
        assert_eq!(
            status.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(
            status.detail.contains("control plane 'demo'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("node 'aws-linux-node'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("provider 'aws'"),
            "{}",
            status.detail
        );

        let stop = stop_machine(&config, tempdir.path(), "cloud-aws", Duration::from_secs(1))
            .expect("hosted standard stop should succeed");
        assert_eq!(stop.machine_name, "cloud-aws");
        assert_eq!(
            stop.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(
            stop.detail.contains("control plane 'demo'"),
            "{}",
            stop.detail
        );
        assert!(
            stop.detail.contains("node 'aws-linux-node'"),
            "{}",
            stop.detail
        );
        assert!(stop.detail.contains("provider 'aws'"), "{}", stop.detail);
    }

    #[test]
    fn hosted_cloud_hypervisor_launch_status_stop_route_through_live_control_plane() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .substrate = ExecutionSubstrate::CloudHypervisor;
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .substrates = vec![ExecutionSubstrate::CloudHypervisor];
        write_fake_cloud_hypervisor_artifacts(&mut config, tempdir.path());
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "cloud-hypervisor");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");
        let placement_state_path =
            hosted_placeholder_runtime_root("demo").join("machine-placements.json");

        let metadata = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect("hosted cloud hypervisor launch should succeed");

        let expected_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let expected_paths = RuntimePaths::for_machine(&expected_runtime_root, "cloud-aws");
        assert_eq!(metadata.machine_name, "cloud-aws");
        assert_eq!(metadata.firecracker_binary, fake_binary);
        assert_eq!(metadata.runtime_dir, expected_paths.runtime_dir);
        assert_eq!(metadata.manifest_path, expected_paths.manifest_path);
        assert_eq!(
            metadata.log_path,
            cloud_hypervisor_log_path(&expected_paths)
        );
        assert!(
            expected_paths
                .runtime_dir
                .join("cloud-hypervisor-runtime.json")
                .exists()
        );

        let placement_state: serde_json::Value = serde_json::from_slice(
            &fs::read(&placement_state_path).expect("machine placement state should exist"),
        )
        .expect("machine placement state should decode");
        assert_eq!(
            placement_state["machines"]["cloud-aws"]["node_name"].as_str(),
            Some("aws-linux-node")
        );
        assert_eq!(
            placement_state["machines"]["cloud-aws"]["runtime_root"].as_str(),
            Some(expected_runtime_root.to_string_lossy().as_ref())
        );

        let status = machine_status(&config, tempdir.path(), "cloud-aws")
            .expect("hosted cloud hypervisor status should load");
        assert_eq!(status.machine_name, "cloud-aws");
        assert_eq!(status.state, MachineRuntimeState::Running);
        assert_eq!(
            status.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(
            status.detail.contains("Cloud Hypervisor"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("control plane 'demo'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("node 'aws-linux-node'"),
            "{}",
            status.detail
        );
        assert!(
            status.detail.contains("provider 'aws'"),
            "{}",
            status.detail
        );

        let stop = stop_machine(&config, tempdir.path(), "cloud-aws", Duration::from_secs(1))
            .expect("hosted cloud hypervisor stop should succeed");
        assert_eq!(stop.machine_name, "cloud-aws");
        assert_eq!(stop.previous_state, MachineRuntimeState::Running);
        assert_eq!(stop.current_state, MachineRuntimeState::Stopped);
        assert_eq!(
            stop.control,
            port_model::MachineControlContract::hosted_control_plane()
        );
        assert!(stop.detail.contains("Cloud Hypervisor"), "{}", stop.detail);
        assert!(
            stop.detail.contains("control plane 'demo'"),
            "{}",
            stop.detail
        );
        assert!(
            stop.detail.contains("node 'aws-linux-node'"),
            "{}",
            stop.detail
        );
        assert!(stop.detail.contains("provider 'aws'"), "{}", stop.detail);
    }

    #[test]
    fn hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback() {
        let _guard = hosted_server_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config
            .machines
            .retain(|name, _| name == "demo" || name == "cloud-aws");
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .substrate = ExecutionSubstrate::CloudHypervisor;
        let config = start_named_live_hosted_servers_inner(&config, &["aws-linux-node"])
            .expect("hosted servers should start");

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("hosted cloud hypervisor launch should reject firecracker-only nodes");

        let message = error.to_string();
        assert!(message.contains("cloud-aws"), "{message}");
        assert!(message.contains("control plane 'demo'"), "{message}");
        assert!(message.contains("aws-linux-node"), "{message}");
        assert!(message.contains("cloud-hypervisor"), "{message}");
        assert!(message.contains("rejected nodes"), "{message}");
        assert!(
            message.contains("requires standard protection on x86_64 via cloud-hypervisor"),
            "{message}"
        );
        assert!(
            !message.contains(
                "failed to launch machine 'cloud-aws' through the live hosted control-plane route"
            ),
            "{message}"
        );
    }

    #[test]
    fn avf_launch_fails_fast_on_non_macos_hosts() {
        let config = sample_avf_config();
        let tempdir = tempdir().expect("tempdir should exist");

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("non-macOS host should fail fast");

        assert!(error.to_string().contains("requires running Port on macOS"));
    }

    #[test]
    fn avf_launch_status_and_stop_write_canonical_runtime_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let (config, paths, metadata) = launch_sample_avf_machine(tempdir.path());

        let avf_metadata: AvfRuntimeMetadata =
            read_json_file(&paths.runtime_dir.join("avf-runtime.json"))
                .expect("avf runtime metadata should decode");

        assert_eq!(metadata.machine_name, "demo");
        assert_eq!(metadata.firecracker_binary, avf_metadata.launcher);
        assert!(metadata.manifest_path.exists());
        assert_eq!(avf_metadata.machine_name, "demo");
        assert_eq!(avf_metadata.launcher, metadata.firecracker_binary);
        assert_eq!(avf_metadata.pid, metadata.pid);
        assert_eq!(avf_metadata.config_path, metadata.config_path);
        assert_eq!(avf_metadata.guest_agent_socket, paths.guest_agent_socket);
        assert_eq!(
            fs::read_to_string(&paths.firecracker_log).expect("console log should read"),
            "avf-launcher booted\n"
        );

        let status = machine_status(&config, tempdir.path(), "demo")
            .expect("status should route through avf driver");
        assert_eq!(status.state, MachineRuntimeState::Running);
        assert_eq!(status.pid, Some(metadata.pid));
        assert!(status.detail.contains("AVF"));
        assert!(status.detail.contains("avf-runtime.json"));

        let stopped = stop_machine(&config, tempdir.path(), "demo", Duration::from_secs(1))
            .expect("stop should route through avf driver");
        assert_eq!(stopped.previous_state, MachineRuntimeState::Running);
        assert_eq!(stopped.current_state, MachineRuntimeState::Stopped);
        assert_eq!(stopped.pid, Some(metadata.pid));
        assert!(stopped.detail.contains("AVF"));
    }

    #[test]
    fn cloud_hypervisor_launch_status_and_stop_write_canonical_runtime_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = PortConfig::sample();
        write_fake_cloud_hypervisor_artifacts(&mut config, tempdir.path());
        let fake_binary = write_fake_firecracker_binary(tempdir.path(), "cloud-hypervisor");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let metadata = cloud_hypervisor_local_launch_machine(
            &config,
            &LaunchRequest {
                machine_name: "demo-ch",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect("cloud hypervisor launch should succeed");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo-ch");
        let runtime_metadata: CloudHypervisorRuntimeMetadata =
            read_json_file(&paths.runtime_dir.join("cloud-hypervisor-runtime.json"))
                .expect("cloud hypervisor runtime metadata should decode");

        assert_eq!(metadata.machine_name, "demo-ch");
        assert_eq!(metadata.firecracker_binary, fake_binary);
        assert_eq!(metadata.config_path, cloud_hypervisor_config_path(&paths));
        assert_eq!(metadata.log_path, cloud_hypervisor_log_path(&paths));
        assert!(metadata.manifest_path.exists());
        assert_eq!(runtime_metadata.machine_name, "demo-ch");
        assert_eq!(runtime_metadata.binary, fake_binary);
        assert_eq!(runtime_metadata.pid, metadata.pid);
        assert_eq!(runtime_metadata.config_path, metadata.config_path);
        assert_eq!(
            runtime_metadata.api_socket_path,
            cloud_hypervisor_api_socket_path(&paths)
        );
        assert_eq!(runtime_metadata.console_log, metadata.log_path);

        let status = machine_status(&config, tempdir.path(), "demo-ch")
            .expect("status should route through cloud hypervisor driver");
        assert_eq!(status.state, MachineRuntimeState::Running);
        assert_eq!(status.pid, Some(metadata.pid));
        assert_eq!(status.config_path, metadata.config_path);
        assert_eq!(status.firecracker_log, metadata.log_path);
        assert!(status.detail.contains("Cloud Hypervisor"));
        assert!(status.detail.contains("cloud-hypervisor-runtime.json"));

        let stopped = stop_machine(&config, tempdir.path(), "demo-ch", Duration::from_secs(1))
            .expect("stop should route through cloud hypervisor driver");
        assert_eq!(stopped.previous_state, MachineRuntimeState::Running);
        assert_eq!(stopped.current_state, MachineRuntimeState::Stopped);
        assert_eq!(stopped.pid, Some(metadata.pid));
        assert!(stopped.detail.contains("Cloud Hypervisor"));
    }

    #[test]
    fn cloud_hypervisor_launch_surfaces_missing_binary_preflight() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = PortConfig::sample();
        write_fake_cloud_hypervisor_artifacts(&mut config, tempdir.path());

        let error = launch_local_machine(
            &config,
            &LaunchRequest {
                machine_name: "demo-ch",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("launch should fail without cloud-hypervisor binary");

        let message = error.to_string();
        assert!(message.contains("Cloud Hypervisor local launch requires"));
        assert!(message.contains("cloud-hypervisor"));
        assert!(message.contains("demo-ch"));
    }

    #[test]
    fn avf_guest_exec_pty_and_logs_use_runtime_socket_after_launch() {
        let tempdir = tempdir().expect("tempdir should exist");
        let (config, paths, metadata) = launch_sample_avf_machine(tempdir.path());
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let server = thread::spawn(move || {
            let (mut exec_stream, _) = listener.accept().expect("exec accept");
            let exec_reader_stream = exec_stream.try_clone().expect("exec clone");
            let mut exec_reader = BufReader::new(exec_reader_stream);
            let exec_request: RequestEnvelope = read_frame(&mut exec_reader).expect("exec request");
            let GuestOperation::Exec(exec_request) = exec_request.operation else {
                panic!("unexpected exec operation");
            };
            assert_eq!(
                exec_request.command,
                vec![String::from("/bin/echo"), String::from("avf-ok")]
            );
            write_frame(
                &mut exec_stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Exec(ExecResult {
                        stdout: String::from("avf-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("exec response should encode");

            let (mut pty_stream, _) = listener.accept().expect("pty accept");
            let pty_reader_stream = pty_stream.try_clone().expect("pty clone");
            let mut pty_reader = BufReader::new(pty_reader_stream);
            let pty_request: RequestEnvelope = read_frame(&mut pty_reader).expect("pty request");
            let GuestOperation::Pty(pty_request) = pty_request.operation else {
                panic!("unexpected pty operation");
            };
            assert_eq!(
                pty_request.command,
                vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("tty")
                ]
            );
            write_frame(
                &mut pty_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Pty,
                    size_bytes: None,
                },
            )
            .expect("pty accepted should encode");
            write_frame(
                &mut pty_stream,
                &StreamResponseFrame::Data {
                    channel: port_agent_protocol::StreamOutputChannel::Stdout,
                    data: String::from("pty-ok\r\n"),
                },
            )
            .expect("pty data should encode");
            write_frame(&mut pty_stream, &StreamResponseFrame::Exit { exit_code: 0 })
                .expect("pty exit should encode");

            let (mut logs_stream, _) = listener.accept().expect("logs accept");
            let logs_reader_stream = logs_stream.try_clone().expect("logs clone");
            let mut logs_reader = BufReader::new(logs_reader_stream);
            let logs_request: RequestEnvelope = read_frame(&mut logs_reader).expect("logs request");
            let GuestOperation::Logs(logs_request) = logs_request.operation else {
                panic!("unexpected logs operation");
            };
            assert_eq!(logs_request.path, String::from("var/log/app.log"));
            assert_eq!(logs_request.tail_lines, Some(10));
            write_frame(
                &mut logs_stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Logs(LogsResult {
                        contents: String::from("log-one\nlog-two\n"),
                    }),
                },
            )
            .expect("logs response should encode");
        });

        let exec_result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("avf-ok")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("avf guest exec should succeed");
        match exec_result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "avf-ok\n"),
            other => panic!("unexpected exec result: {other:?}"),
        }

        let pty_result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Pty(PtyRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("tty"),
                    ],
                    cols: 80,
                    rows: 24,
                }),
            },
        )
        .expect("avf guest pty should succeed");
        match pty_result {
            OperationResult::Pty(result) => assert_eq!(result.transcript, "pty-ok\r\n"),
            other => panic!("unexpected pty result: {other:?}"),
        }

        let logs_result = execute_guest_operation(
            &config,
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Logs(LogsRequest {
                    path: String::from("var/log/app.log"),
                    follow: false,
                    tail_lines: Some(10),
                }),
            },
        )
        .expect("avf guest logs should succeed");
        match logs_result {
            OperationResult::Logs(result) => assert_eq!(result.contents, "log-one\nlog-two\n"),
            other => panic!("unexpected logs result: {other:?}"),
        }

        server.join().expect("server thread should complete");
        let _ = stop_machine(&config, tempdir.path(), "demo", Duration::from_secs(1))
            .expect("avf machine should stop");
        let _ = metadata;
    }

    #[test]
    fn avf_copy_and_forward_use_runtime_socket_after_launch() {
        let tempdir = tempdir().expect("tempdir should exist");
        let (config, paths, _metadata) = launch_sample_avf_machine(tempdir.path());
        let host_source = tempdir.path().join("host.txt");
        fs::write(&host_source, "copy-ok").expect("host source should write");
        let host_destination = tempdir.path().join("downloaded.txt");
        let host_destination_for_server = host_destination.clone();

        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");
        let server = thread::spawn(move || {
            let (mut upload_stream, _) = listener.accept().expect("upload accept");
            let upload_reader_stream = upload_stream.try_clone().expect("upload clone");
            let mut upload_reader = BufReader::new(upload_reader_stream);
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
            drop(download_stream);

            let (mut forward_stream, _) = listener.accept().expect("forward accept");
            let forward_reader_stream = forward_stream.try_clone().expect("forward clone");
            let mut forward_reader = BufReader::new(forward_reader_stream);
            let request: RequestEnvelope =
                read_frame(&mut forward_reader).expect("forward request");
            let GuestOperation::Forward(request) = request.operation else {
                panic!("unexpected forward operation");
            };
            assert_eq!(request.target, "127.0.0.1:8081");
            write_frame(
                &mut forward_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: None,
                },
            )
            .expect("forward accepted should encode");
            forward_stream
                .write_all(b"ready")
                .expect("forward eager bytes should write");
            forward_stream
                .flush()
                .expect("forward eager bytes should flush");
            let mut proxied = [0_u8; 4];
            forward_reader
                .read_exact(&mut proxied)
                .expect("forward payload should read");
            assert_eq!(&proxied, b"ping");
            forward_stream
                .write_all(b"pong")
                .expect("forward response should write");
            let _ = forward_stream.shutdown(Shutdown::Write);
        });

        let upload = copy_guest_file(
            &config,
            GuestCopyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                source: &host_source,
                destination: Path::new("/workspace/copied.txt"),
                direction: CopyDirection::HostToGuest,
            },
        )
        .expect("avf upload should succeed");
        assert_eq!(upload.bytes_copied, 7);

        let download = copy_guest_file(
            &config,
            GuestCopyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                source: Path::new("/workspace/copied.txt"),
                destination: &host_destination,
                direction: CopyDirection::GuestToHost,
            },
        )
        .expect("avf download should succeed");
        assert_eq!(download.bytes_copied, 7);
        assert_eq!(
            fs::read_to_string(&host_destination).expect("downloaded file should read"),
            "copy-ok"
        );

        let session = prepare_guest_forward(
            &config,
            GuestForwardRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                listen: "127.0.0.1:0",
                target: "127.0.0.1:8081",
            },
        )
        .expect("avf forward session should prepare");
        let listen_addr = session.listen_addr();
        let super::GuestForwardSession {
            listener,
            endpoint,
            target,
        } = session;
        let thread = thread::spawn(move || match listener {
            super::ForwardListener::Tcp(listener) => {
                let (inbound, _) = listener.accept().expect("forward listener should accept");
                super::proxy_guest_forward_connection(endpoint, target, inbound)
                    .expect("forward session should proxy");
            }
            super::ForwardListener::Unix {
                listener,
                socket_path,
            } => {
                let (inbound, _) = listener.accept().expect("forward listener should accept");
                let result = super::proxy_guest_forward_connection(endpoint, target, inbound);
                let _ = fs::remove_file(socket_path);
                result.expect("forward session should proxy");
            }
        });
        thread::sleep(Duration::from_millis(100));

        let mut client = TcpStream::connect(&listen_addr).expect("forward listener should accept");
        let mut eager = [0_u8; 5];
        client
            .read_exact(&mut eager)
            .expect("forward eager bytes should read");
        assert_eq!(&eager, b"ready");
        client
            .write_all(b"ping")
            .expect("forward payload should write");
        let mut response = [0_u8; 4];
        client
            .read_exact(&mut response)
            .expect("forward response should read");
        assert_eq!(&response, b"pong");
        drop(client);

        server.join().expect("copy/forward server should complete");
        thread.join().expect("forward thread should complete");

        let monitor = machine_monitor(&config, tempdir.path(), "demo")
            .expect("avf machine monitor should load");
        assert_eq!(monitor.firecracker_log, paths.firecracker_log);
        assert_eq!(
            fs::read_to_string(&monitor.firecracker_log).expect("console log should read"),
            "avf-launcher booted\n"
        );
        let top =
            machine_top(&config, tempdir.path(), "demo").expect("avf machine top should load");
        let hypervisor = top
            .entries
            .iter()
            .find(|entry| entry.kind == super::MachineTopEntryKind::Hypervisor)
            .expect("avf top entry should exist");
        assert_eq!(hypervisor.name, "avf");

        let _ = stop_machine(&config, tempdir.path(), "demo", Duration::from_secs(1))
            .expect("avf machine should stop");
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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
        let config = start_live_hosted_servers(&config, true).expect("hosted servers should start");

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
    fn hosted_standard_aws_launch_uses_hosted_route_context_instead_of_provider_guidance() {
        let tempdir = tempdir().expect("tempdir should exist");
        let error = launch_local_machine(
            &PortConfig::sample(),
            &LaunchRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                boot_wait: Duration::from_secs(0),
            },
        )
        .expect_err("hosted AWS launch should fail through the hosted route");

        let message = error.to_string();
        assert!(message.contains("cloud-aws"));
        assert!(
            message.contains("failed to resolve live hosted client transport"),
            "{message}"
        );
        assert!(!message.contains("Run Port on the AWS Linux host itself"));
    }

    #[test]
    fn launch_rejects_pvm_host_kit_when_runtime_is_not_prepared() {
        let mut config = PortConfig::sample();
        config.clusters.clear();
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
    fn launch_preflight_requires_kvm_for_standard_firecracker_lane() {
        let config = PortConfig::sample();
        let machine = config.machines.get("demo").expect("demo should exist");
        let checks = crate::launch_preflight_checks(
            machine,
            Path::new("/tmp/kernel"),
            Path::new("/tmp/guest"),
        );

        assert!(checks.iter().any(|check| check.name == "kvm-device"));
    }

    #[test]
    fn launch_preflight_skips_kvm_for_firecracker_pvm_lane() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let machine = config
            .machines
            .get("cloud-aws")
            .expect("cloud-aws should exist");
        let checks = crate::launch_preflight_checks(
            machine,
            Path::new("/tmp/kernel"),
            Path::new("/tmp/guest"),
        );

        assert!(!checks.iter().any(|check| check.name == "kvm-device"));
    }

    #[test]
    fn launch_preflight_requires_overlay_dependencies_for_rootfs_overlay() {
        let tempdir = tempdir().expect("tempdir should exist");
        let kernel = tempdir.path().join("vmlinux");
        let rootfs = tempdir.path().join("rootfs.ext4");
        let initrd = tempdir.path().join("initrd.cpio.gz");
        fs::write(&kernel, "kernel").expect("kernel should write");
        fs::write(&rootfs, "rootfs").expect("rootfs should write");
        fs::write(&initrd, "initrd").expect("initrd should write");

        let mut config = PortConfig::sample();
        let machine = config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist");
        machine.architecture = port_model::MachineArchitecture::X86_64;
        machine.protection_mode = port_model::ProtectionMode::Pvm;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(port_model::MachineRootfsOverlaySpec { size_mib: 4096 });

        let checks = crate::launch_preflight_checks(machine, &kernel, &rootfs);

        assert!(checks.iter().any(|check| check.name == "mkfs-ext4"));
        assert!(
            checks
                .iter()
                .any(|check| check.name == "rootfs-overlay-initrd" && check.ok)
        );
    }

    #[test]
    fn launch_preflight_honors_explicit_iptables_binary_override() {
        let tempdir = tempdir().expect("tempdir should exist");
        let ip_path = tempdir.path().join("ip");
        fs::write(
            &ip_path,
            "#!/bin/sh\nif [ \"${1:-}\" = \"-V\" ]; then\n  echo 'ip utility, iproute2-6.19.0, libbpf 1.6.3'\n  exit 0\nfi\nexit 0\n",
        )
        .expect("fake ip should write");
        let mut permissions = fs::metadata(&ip_path)
            .expect("fake ip metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&ip_path, permissions).expect("fake ip permissions should update");

        let iptables_path = tempdir.path().join("iptables-legacy");
        fs::write(
            &iptables_path,
            "#!/bin/sh\nif [ \"${1:-}\" = \"--version\" ]; then\n  echo 'iptables v1.8.12 (legacy)'\n  exit 0\nfi\nexit 0\n",
        )
        .expect("fake iptables should write");
        let mut permissions = fs::metadata(&iptables_path)
            .expect("fake iptables metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&iptables_path, permissions)
            .expect("fake iptables permissions should update");

        let _path_guard = ScopedPathEnv::replace(tempdir.path());
        let _iptables_guard = ScopedEnvVar::set(crate::PORT_IPTABLES_BINARY_ENV, &iptables_path);

        let config = PortConfig::sample();
        let machine = config
            .machines
            .get("cloud-aws")
            .expect("cloud-aws should exist");
        let checks = crate::launch_preflight_checks(
            machine,
            Path::new("/tmp/kernel"),
            Path::new("/tmp/guest"),
        );

        let iptables_check = checks
            .iter()
            .find(|check| check.name == "iptables")
            .expect("iptables check should exist");
        assert!(iptables_check.ok, "{}", iptables_check.detail);
        assert!(
            iptables_check
                .detail
                .contains(&iptables_path.display().to_string()),
            "{}",
            iptables_check.detail
        );
    }

    #[test]
    fn launch_preflight_prefers_iproute2_binary_over_busybox_ip_on_path() {
        let tempdir = tempdir().expect("tempdir should exist");
        let busybox_dir = tempdir.path().join("busybox");
        let iproute_dir = tempdir.path().join("iproute2");
        fs::create_dir_all(&busybox_dir).expect("busybox dir should exist");
        fs::create_dir_all(&iproute_dir).expect("iproute dir should exist");

        write_fake_ip_binary(
            &busybox_dir,
            "ip",
            "#!/bin/sh\nif [ \"${1:-}\" = \"-V\" ]; then\n  echo 'BusyBox v1.37.0'\n  exit 0\nfi\nexit 0\n",
        );
        write_fake_ip_binary(
            &iproute_dir,
            "ip",
            "#!/bin/sh\nif [ \"${1:-}\" = \"-V\" ]; then\n  echo 'ip utility, iproute2-6.19.0, libbpf 1.6.3'\n  exit 0\nfi\nif [ \"${1:-}\" = \"route\" ]; then\n  echo 'default via 192.0.2.1 dev eth0'\n  exit 0\nfi\nexit 0\n",
        );
        write_fake_network_binaries(tempdir.path());
        let _path_guard = ScopedPathEnv::from_paths([
            busybox_dir.as_path(),
            iproute_dir.as_path(),
            tempdir.path(),
        ]);

        let mut config = PortConfig::sample();
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let machine = config
            .machines
            .get("demo")
            .expect("demo machine should exist");
        let kernel = config.artifacts.kernels["demo-kernel"].variants[0]
            .path
            .clone();
        let rootfs = config.artifacts.guest_images["demo-guest"].variants[0]
            .path
            .clone();

        let checks = crate::launch_preflight_checks(machine, &kernel, &rootfs);
        let ip_check = checks
            .iter()
            .find(|check| check.name == "iproute2")
            .expect("iproute2 check should exist");
        assert!(ip_check.ok, "{}", ip_check.detail);
    }

    #[test]
    fn default_outbound_interface_prefers_iproute2_binary_over_busybox_ip_on_path() {
        let tempdir = tempdir().expect("tempdir should exist");
        let busybox_dir = tempdir.path().join("busybox");
        let iproute_dir = tempdir.path().join("iproute2");
        fs::create_dir_all(&busybox_dir).expect("busybox dir should exist");
        fs::create_dir_all(&iproute_dir).expect("iproute dir should exist");

        write_fake_ip_binary(
            &busybox_dir,
            "ip",
            "#!/bin/sh\nif [ \"${1:-}\" = \"-V\" ]; then\n  echo 'BusyBox v1.37.0'\n  exit 0\nfi\necho 'busybox-ip-was-used' >&2\nexit 1\n",
        );
        write_fake_ip_binary(
            &iproute_dir,
            "ip",
            "#!/bin/sh\nif [ \"${1:-}\" = \"-V\" ]; then\n  echo 'ip utility, iproute2-6.19.0, libbpf 1.6.3'\n  exit 0\nfi\nif [ \"${1:-}\" = \"route\" ] && [ \"${2:-}\" = \"show\" ] && [ \"${3:-}\" = \"default\" ]; then\n  echo 'default via 192.0.2.1 dev eth0'\n  exit 0\nfi\nexit 1\n",
        );
        let _path_guard = ScopedPathEnv::from_paths([busybox_dir.as_path(), iproute_dir.as_path()]);

        assert_eq!(
            crate::default_outbound_interface().expect("default route should resolve"),
            "eth0"
        );
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
    fn guest_exec_uses_cloud_hypervisor_vsock_tunnel_when_runtime_socket_is_absent() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo-ch");
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
                        vec![String::from("/bin/echo"), String::from("live-ch-ok")]
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
                        stdout: String::from("live-ch-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("response should encode");
        });

        let result = execute_guest_operation(
            &PortConfig::sample(),
            GuestRequest {
                machine_name: "demo-ch",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("live-ch-ok")],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("live cloud-hypervisor guest exec should succeed");

        match result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "live-ch-ok\n"),
            other => panic!("unexpected result: {other:?}"),
        }

        server.join().expect("server thread should complete");
    }

    #[test]
    fn guest_vsock_tunnel_times_out_when_handshake_stalls() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener = UnixListener::bind(&paths.vsock_path).expect("vsock listener should bind");

        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().expect("should accept guest transport");
            thread::sleep(Duration::from_millis(150));
        });

        let start = std::time::Instant::now();
        let error = super::connect_vsock_tunnel_with_timeout(
            "Firecracker",
            &paths.vsock_path,
            7000,
            Duration::from_millis(50),
        )
        .expect_err("stalled handshake should time out");

        assert!(
            start.elapsed() < Duration::from_millis(120),
            "handshake timeout took {:?}",
            start.elapsed()
        );
        assert!(
            error
                .to_string()
                .contains("failed to read Firecracker response"),
            "{error}"
        );

        server.join().expect("server thread should complete");
    }

    #[test]
    fn guest_streaming_operations_aggregate_pty_and_followed_logs_from_runtime_socket() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest agent socket should bind");

        let server = thread::spawn(move || {
            let (mut pty_stream, _) = listener.accept().expect("pty accept");
            let pty_reader_stream = pty_stream.try_clone().expect("pty clone");
            let mut pty_reader = BufReader::new(pty_reader_stream);
            let pty_request: RequestEnvelope = read_frame(&mut pty_reader).expect("pty request");
            let GuestOperation::Pty(pty_request) = pty_request.operation else {
                panic!("unexpected pty operation");
            };
            assert_eq!(
                pty_request.command,
                vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf pty-stream-ok"),
                ]
            );
            write_frame(
                &mut pty_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Pty,
                    size_bytes: None,
                },
            )
            .expect("pty accepted should encode");
            write_frame(
                &mut pty_stream,
                &StreamResponseFrame::Data {
                    channel: port_agent_protocol::StreamOutputChannel::Stdout,
                    data: String::from("pty-stream-ok"),
                },
            )
            .expect("pty data should encode");
            write_frame(&mut pty_stream, &StreamResponseFrame::Exit { exit_code: 0 })
                .expect("pty exit should encode");

            let (mut logs_stream, _) = listener.accept().expect("logs accept");
            let logs_reader_stream = logs_stream.try_clone().expect("logs clone");
            let mut logs_reader = BufReader::new(logs_reader_stream);
            let logs_request: RequestEnvelope = read_frame(&mut logs_reader).expect("logs request");
            let GuestOperation::Logs(logs_request) = logs_request.operation else {
                panic!("unexpected logs operation");
            };
            assert!(logs_request.follow);
            write_frame(
                &mut logs_stream,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Logs,
                    size_bytes: None,
                },
            )
            .expect("logs accepted should encode");
            write_frame(
                &mut logs_stream,
                &StreamResponseFrame::Data {
                    channel: port_agent_protocol::StreamOutputChannel::Logs,
                    data: String::from("line-1\n"),
                },
            )
            .expect("logs data should encode");
            write_frame(
                &mut logs_stream,
                &StreamResponseFrame::Data {
                    channel: port_agent_protocol::StreamOutputChannel::Logs,
                    data: String::from("line-2\n"),
                },
            )
            .expect("logs data should encode");
            write_frame(&mut logs_stream, &StreamResponseFrame::Eof)
                .expect("logs eof should encode");
        });

        let pty_result = execute_guest_operation(
            &PortConfig::sample(),
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Pty(PtyRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("printf pty-stream-ok"),
                    ],
                    cols: 80,
                    rows: 24,
                }),
            },
        )
        .expect("streamed guest pty should succeed");
        match pty_result {
            OperationResult::Pty(result) => assert_eq!(result.transcript, "pty-stream-ok"),
            other => panic!("unexpected pty result: {other:?}"),
        }

        let logs_result = execute_guest_operation(
            &PortConfig::sample(),
            GuestRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                operation: GuestOperation::Logs(LogsRequest {
                    path: String::from("/var/log/app.log"),
                    follow: true,
                    tail_lines: None,
                }),
            },
        )
        .expect("streamed guest logs should succeed");
        match logs_result {
            OperationResult::Logs(result) => assert_eq!(result.contents, "line-1\nline-2\n"),
            other => panic!("unexpected logs result: {other:?}"),
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
    fn stage_local_cluster_bootstrap_copies_offline_inputs_and_proves_install() {
        let tempdir = tempdir().expect("tempdir should exist");
        let guest_root = tempdir.path().join("guest-root");
        let runtime_root = tempdir.path().join("runtime");
        let socket_path = runtime_root.join("demo").join("guest-agent.sock");
        fs::create_dir_all(&guest_root).expect("guest root should exist");
        write_fake_guest_k3s_runtime(&guest_root);
        write_fake_cluster_bootstrap_assets(tempdir.path());
        let _repo_root = ScopedEnvVar::set("PORT_REPO_ROOT", tempdir.path());
        fs::create_dir_all(
            socket_path
                .parent()
                .expect("guest agent socket parent should exist"),
        )
        .expect("runtime root should exist");

        let server_socket = socket_path.clone();
        let server_root = guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&server_socket, server_root).expect("guest agent should serve");
        });
        for _ in 0..100 {
            if socket_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(socket_path.exists(), "guest agent socket should appear");

        let result = stage_local_cluster_bootstrap(
            &PortConfig::sample(),
            ClusterStageRequest {
                cluster_name: "demo",
                runtime_root: &runtime_root,
            },
        )
        .expect("local cluster stage should succeed");

        assert_eq!(result.cluster_name, "demo");
        assert_eq!(result.machine_name, "demo");
        assert_eq!(result.guest_profile, "kube-ready");
        assert_eq!(result.stage_root, PathBuf::from("/opt/port/clusters/demo"));
        assert_eq!(result.staged_files.len(), 2);
        assert!(result.preflight_stdout.contains("required-command:sh"));
        assert!(result.preflight_stdout.contains("guest-profile-ok"));
        assert!(result.install_stdout.contains("offline-install-ok"));
        assert!(
            result
                .install_stdout
                .contains("installed-binary:/opt/port/clusters/demo/bin/k3s")
        );
        assert!(
            result
                .install_stdout
                .contains("installed-kubectl:/opt/port/clusters/demo/bin/kubectl")
        );
        let rendered_install_command = result.install_command.join(" ");
        assert!(rendered_install_command.contains("install-k3s-offline.sh"));
        assert!(!rendered_install_command.contains("curl"));
        assert!(!rendered_install_command.contains("get.k3s.io"));

        let staged_root = guest_root.join("opt/port/clusters/demo");
        let install_script = staged_root.join("install-k3s-offline.sh");
        let binary = staged_root.join("k3s");
        let installed_binary = staged_root.join("bin/k3s");
        let installed_kubectl = staged_root.join("bin/kubectl");
        assert!(install_script.exists(), "install script should be staged");
        assert!(binary.exists(), "binary should be staged");
        assert!(
            fs::read_to_string(&binary)
                .expect("staged binary should read")
                .contains("exec usr/bin/k3s")
        );
        assert_eq!(
            fs::metadata(&binary)
                .expect("staged binary metadata should exist")
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
        assert!(installed_binary.exists(), "installed binary should exist");
        assert!(
            fs::symlink_metadata(&installed_kubectl).is_ok(),
            "installed kubectl link should exist"
        );
        assert_eq!(
            fs::read_link(&installed_kubectl).expect("installed kubectl link should read"),
            PathBuf::from("k3s")
        );
    }

    #[test]
    fn local_cluster_lifecycle_reports_readiness_and_returns_kubeconfig() {
        let tempdir = tempdir().expect("tempdir should exist");
        let guest_root = tempdir.path().join("guest-root");
        let runtime_root = tempdir.path().join("runtime");
        fs::create_dir_all(&guest_root).expect("guest root should exist");
        write_fake_guest_k3s_runtime(&guest_root);
        write_fake_cluster_bootstrap_assets(tempdir.path());
        let _repo_root = ScopedEnvVar::set("PORT_REPO_ROOT", tempdir.path());

        let mut config = PortConfig::sample();
        write_fake_standard_firecracker_artifacts(&mut config, tempdir.path());
        let _binary = write_fake_firecracker_binary(tempdir.path(), "firecracker");
        let _path_guard = ScopedPathEnv::prepend(tempdir.path());

        let runtime_dir = runtime_root.join("demo");
        let paths = RuntimePaths::for_machine(&runtime_root, "demo");
        let backend_socket = tempdir.path().join("demo-guest-agent.sock");
        let guest_root_for_thread = guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&backend_socket, guest_root_for_thread)
                .expect("guest agent should serve");
        });

        let backend_socket = tempdir.path().join("demo-guest-agent.sock");
        let vsock_path = paths.vsock_path.clone();
        thread::spawn(move || {
            for _ in 0..200 {
                if runtime_dir.exists() {
                    serve_vsock_guest_agent_proxy(&vsock_path, &backend_socket);
                    return;
                }
                thread::sleep(Duration::from_millis(20));
            }
            panic!("cluster runtime dir did not appear in time");
        });

        let up = up_local_cluster(
            &config,
            ClusterUpRequest {
                cluster_name: "demo",
                runtime_root: &runtime_root,
                boot_wait: Duration::from_secs(1),
            },
        )
        .expect("local cluster up should succeed");
        assert_eq!(up.cluster_name, "demo");
        assert_eq!(up.machine_name, "demo");
        assert_eq!(up.launch_action, "launched");
        assert_eq!(up.status.readiness, ClusterReadinessState::Ready);
        assert!(up.status.health_output.contains("control-plane,master"));
        assert!(up.status.kubeconfig_available);
        assert!(
            up.status
                .detail
                .contains("Downstream GitOps/bootstrap convergence remains separate work.")
        );

        let status = local_cluster_status(
            &config,
            ClusterStatusRequest {
                cluster_name: "demo",
                runtime_root: &runtime_root,
            },
        )
        .expect("local cluster status should succeed");
        assert_eq!(status.readiness, ClusterReadinessState::Ready);
        assert_eq!(status.machine_state, MachineRuntimeState::Running);
        assert!(status.health_output.contains("control-plane,master"));
        assert_eq!(status.api_forward_target, "127.0.0.1:6443");

        let kubeconfig = local_cluster_kubeconfig(
            &config,
            ClusterStatusRequest {
                cluster_name: "demo",
                runtime_root: &runtime_root,
            },
        )
        .expect("local cluster kubeconfig should succeed");
        assert_eq!(kubeconfig.cluster_name, "demo");
        assert_eq!(kubeconfig.machine_name, "demo");
        assert_eq!(kubeconfig.api_forward_target, "127.0.0.1:6443");
        assert!(
            kubeconfig
                .kubeconfig
                .contains("server: https://127.0.0.1:6443")
        );
        assert!(
            guest_root.join("etc/rancher/k3s/k3s.yaml").exists(),
            "offline install should materialize kubeconfig in the guest root"
        );

        let down = down_local_cluster(
            &config,
            ClusterDownRequest {
                cluster_name: "demo",
                runtime_root: &runtime_root,
                stop_wait: Duration::from_secs(1),
            },
        )
        .expect("local cluster down should succeed");
        assert_eq!(down.cluster_name, "demo");
        assert_eq!(down.machine_name, "demo");
        assert_eq!(down.stop.current_state, MachineRuntimeState::Stopped);
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

    #[test]
    fn service_status_exposes_runtime_contract_even_before_execution() {
        let manifest_path = PathBuf::from("/tmp/runtime/demo/services/definitions/buildbox.json");
        let record = ServiceDefinitionRecord {
            machine_name: String::from("demo"),
            name: String::from("buildbox"),
            kind: ServiceKind::Sandbox,
            desired_state: ServiceDesiredState::Active,
            command: vec![
                String::from("/bin/sh"),
                String::from("-lc"),
                String::from("make test"),
            ],
            secret_bindings: Vec::new(),
            policy: ServicePolicy::default(),
            control: port_model::MachineControlContract::local_runtime_root(),
            control_plane: None,
            node_name: None,
            host_groups: Vec::new(),
            host_group_policies: BTreeMap::from([(
                String::from("aws-builders"),
                HostedSchedulerPolicy::DeterministicFirstFit,
            )]),
            target_host_group: Some(String::from("aws-builders")),
            scheduler: Some(HostedSchedulerPolicy::DeterministicFirstFit),
            created_at_unix_s: 1,
            detail: String::from("stored definition"),
        };

        let status = service_status_from_record(record, manifest_path.clone());
        assert_eq!(status.runtime.state, super::ServiceRuntimeState::Stored);
        assert_eq!(
            status.runtime.record_path,
            service_runtime_dir(Path::new("/tmp/runtime/demo")).join("buildbox.json")
        );
        assert_eq!(status.runtime.restart_count, 0);
        assert_eq!(status.runtime.pid, None);
        assert_eq!(status.runtime.exit_code, None);
        assert_eq!(status.runtime.last_exit_code, None);
        assert_eq!(status.runtime.last_exit_detail, None);
        assert_eq!(status.runtime.health_state, ServiceHealthState::Unknown);
        assert_eq!(status.runtime.health_detail, None);
        assert_eq!(status.runtime.stdout_path, None);
        assert_eq!(status.runtime.stderr_path, None);
        assert_eq!(
            status.host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(status.target_host_group.as_deref(), Some("aws-builders"));
        assert_eq!(
            status.scheduler,
            Some(HostedSchedulerPolicy::DeterministicFirstFit)
        );
        assert_eq!(status.manifest_path, manifest_path);
    }

    #[test]
    fn service_secret_status_projects_runtime_owned_provenance_without_value_leak() {
        let tempdir = tempdir().expect("tempdir should exist");
        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        write_manifest(&paths, "demo", 1);
        let secret = super::store_machine_secret(&paths.runtime_dir, "demo-token", "s3cr3t")
            .expect("secret should store");
        let manifest_path = service_definition_dir(&paths.runtime_dir).join("api.json");
        let record = ServiceDefinitionRecord {
            machine_name: String::from("demo"),
            name: String::from("api"),
            kind: ServiceKind::Service,
            desired_state: ServiceDesiredState::Active,
            command: vec![String::from("/bin/true")],
            secret_bindings: vec![ServiceSecretBinding {
                env: String::from("API_TOKEN"),
                secret: String::from("demo-token"),
            }],
            policy: ServicePolicy::default(),
            control: port_model::MachineControlContract::local_runtime_root(),
            control_plane: None,
            node_name: None,
            host_groups: Vec::new(),
            host_group_policies: BTreeMap::new(),
            target_host_group: None,
            scheduler: None,
            created_at_unix_s: 1,
            detail: String::from("stored definition"),
        };

        let first = service_status_from_record(record.clone(), manifest_path.clone());
        let second = service_status_from_record(record, manifest_path);
        assert_eq!(first.secret_bindings.len(), 1);
        assert_eq!(first.secret_sources.len(), 1);
        assert_eq!(first.secret_sources, second.secret_sources);
        assert_eq!(
            first.secret_sources[0].backend,
            ServiceSecretBackend::RuntimeFile
        );
        assert_eq!(
            first.secret_sources[0].materialization,
            ServiceSecretMaterialization::Env
        );
        assert_eq!(first.secret_sources[0].path, secret.backend_path);
        assert!(!first.secret_sources[0].detail.contains("s3cr3t"));
        let status_debug = format!("{first:?}");
        assert!(!status_debug.contains("s3cr3t"));
        assert!(!first.detail.contains("s3cr3t"));
    }

    #[test]
    fn service_supervision_restarts_local_service_and_projects_last_exit_state() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = PortConfig::sample();
        config.machines.retain(|name, _| name == "demo");

        let paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        write_manifest(&paths, "demo", 1);

        let guest_root = tempdir.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let guest_socket = paths.guest_agent_socket.clone();
        let guest_root_for_thread = guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&guest_socket, guest_root_for_thread)
                .expect("guest agent should serve")
        });
        for _ in 0..100 {
            if paths.guest_agent_socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let applied = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                name: "api",
                kind: ServiceKind::Service,
                host_group: None,
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from(
                        "count_file=workspace/restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; if [ \"$count\" -eq 1 ]; then sleep 0.2; exit 23; fi; trap 'exit 0' TERM; while :; do sleep 1; done",
                    ),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy {
                    restart: ServiceRestartPolicy::OnFailure,
                    healthcheck: ServiceHealthcheck::default(),
                },
            },
        )
        .expect("service apply should succeed");
        assert_eq!(applied.runtime.state, ServiceRuntimeState::Running);
        assert_eq!(applied.runtime.restart_count, 0);

        thread::sleep(Duration::from_millis(350));

        let mut restarted = None;
        for _ in 0..20 {
            let status = machine_service_status(&config, tempdir.path(), "demo", "api")
                .expect("service status should succeed");
            if status.runtime.state == ServiceRuntimeState::Running
                && status.runtime.restart_count >= 1
            {
                restarted = Some(status);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let restarted = restarted.expect("service should restart after the first failure");
        assert_eq!(restarted.runtime.restart_count, 1);
        assert_eq!(restarted.runtime.last_exit_code, Some(23));
        assert!(
            restarted
                .runtime
                .last_exit_detail
                .as_deref()
                .unwrap_or_default()
                .contains("exited with code 23")
        );
        assert_eq!(restarted.runtime.health_state, ServiceHealthState::Unknown);

        let listed = list_machine_services(&config, tempdir.path(), "demo")
            .expect("service list should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].runtime.restart_count, 1);
        assert_eq!(listed[0].runtime.last_exit_code, Some(23));

        let runtime_record =
            fs::read_to_string(service_runtime_dir(&paths.runtime_dir).join("api.json"))
                .expect("runtime record should read");
        assert!(runtime_record.contains("\"restart_count\": 1"));
        assert!(runtime_record.contains("\"last_exit_code\": 23"));
    }

    #[test]
    fn service_health_projects_local_and_hosted_status_consistently() {
        let tempdir = tempdir().expect("tempdir should exist");

        let mut local_config = PortConfig::sample();
        local_config.machines.retain(|name, _| name == "demo");
        let local_paths = RuntimePaths::for_machine(tempdir.path(), "demo");
        write_manifest(&local_paths, "demo", 1);

        let local_guest_root = tempdir.path().join("guest-local");
        fs::create_dir_all(local_guest_root.join("workspace")).expect("workspace should exist");
        let local_guest_socket = local_paths.guest_agent_socket.clone();
        let local_guest_root_for_thread = local_guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&local_guest_socket, local_guest_root_for_thread)
                .expect("guest agent should serve")
        });
        for _ in 0..100 {
            if local_paths.guest_agent_socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let health_policy = ServicePolicy {
            restart: ServiceRestartPolicy::Never,
            healthcheck: ServiceHealthcheck {
                policy: ServiceHealthPolicy::Command,
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("test -f workspace/healthy"),
                ],
                restart_on_unhealthy: false,
            },
        };

        let _local_applied = apply_machine_service(
            &local_config,
            ServiceApplyRequest {
                machine_name: "demo",
                runtime_root: tempdir.path(),
                name: "health-local",
                kind: ServiceKind::Service,
                host_group: None,
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: health_policy.clone(),
            },
        )
        .expect("local service apply should succeed");

        let initial_local =
            machine_service_status(&local_config, tempdir.path(), "demo", "health-local")
                .expect("local service status should succeed");
        assert_eq!(initial_local.runtime.state, ServiceRuntimeState::Running);
        assert_eq!(
            initial_local.runtime.health_state,
            ServiceHealthState::Unknown
        );
        assert_eq!(initial_local.runtime.health_detail, None);

        fs::write(local_guest_root.join("workspace/healthy"), "ok")
            .expect("healthy marker should write");
        let healthy_local =
            machine_service_status(&local_config, tempdir.path(), "demo", "health-local")
                .expect("local service status should succeed");
        assert_eq!(
            healthy_local.runtime.health_state,
            ServiceHealthState::Unknown
        );
        assert_eq!(healthy_local.runtime.health_detail, None);

        let _guard = hosted_server_lock().lock().expect("lock should work");
        let mut hosted_config = sample_config_with_hosted_runtime_roots(tempdir.path());
        hosted_config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let hosted_runtime_root = hosted_config.nodes["aws-linux-node"].runtime_root.clone();
        let hosted_paths = RuntimePaths::for_machine(&hosted_runtime_root, "cloud-aws");
        write_manifest(&hosted_paths, "cloud-aws", 2);

        let hosted_guest_root = tempdir.path().join("guest-hosted");
        fs::create_dir_all(hosted_guest_root.join("workspace")).expect("workspace should exist");
        let hosted_guest_socket = hosted_paths.guest_agent_socket.clone();
        let hosted_guest_root_for_thread = hosted_guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&hosted_guest_socket, hosted_guest_root_for_thread)
                .expect("guest agent should serve")
        });
        for _ in 0..100 {
            if hosted_paths.guest_agent_socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let hosted_config = start_live_hosted_servers_inner(&hosted_config, true)
            .expect("hosted servers should start");
        let _hosted_applied = apply_machine_service(
            &hosted_config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "health-hosted",
                kind: ServiceKind::Service,
                host_group: Some("aws-builders"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: health_policy,
            },
        )
        .expect("hosted service apply should succeed");

        let initial_hosted =
            machine_service_status(&hosted_config, tempdir.path(), "cloud-aws", "health-hosted")
                .expect("hosted service status should succeed");
        assert_eq!(
            initial_hosted.runtime.health_state,
            ServiceHealthState::Unknown
        );
        assert_eq!(initial_hosted.runtime.health_detail, None);

        fs::write(hosted_guest_root.join("workspace/healthy"), "ok")
            .expect("healthy marker should write");
        let healthy_hosted =
            machine_service_status(&hosted_config, tempdir.path(), "cloud-aws", "health-hosted")
                .expect("hosted service status should succeed");
        assert_eq!(
            healthy_hosted.runtime.health_state,
            ServiceHealthState::Unknown
        );
        assert_eq!(healthy_hosted.runtime.health_detail, None);

        let hosted_list = list_machine_services(&hosted_config, tempdir.path(), "cloud-aws")
            .expect("hosted service list should succeed");
        assert_eq!(hosted_list.len(), 1);
        assert_eq!(
            hosted_list[0].runtime.health_state,
            ServiceHealthState::Unknown
        );
    }

    #[test]
    fn service_secret_backend_hosted_lifecycle_uses_runtime_file_backend() {
        let _guard = hosted_server_lock().lock().expect("lock should work");
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_config_with_hosted_runtime_roots(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        write_manifest(&paths, "cloud-aws", 1);

        let guest_root = tempdir.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let guest_socket = paths.guest_agent_socket.clone();
        let guest_root_for_thread = guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&guest_socket, guest_root_for_thread)
                .expect("guest agent should serve")
        });
        for _ in 0..50 {
            if paths.guest_agent_socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let config =
            start_live_hosted_servers_inner(&config, true).expect("hosted servers should start");

        let secret = put_machine_secret(
            &config,
            super::SecretPutRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "demo-token",
                value: "s3cr3t",
            },
        )
        .expect("secret put should succeed");
        assert_eq!(secret.backend, ServiceSecretBackend::RuntimeFile);
        assert_eq!(secret.materialization, ServiceSecretMaterialization::Env);
        assert!(
            secret
                .path
                .ends_with("cloud-aws/services/secrets/demo-token.json")
        );
        assert!(
            secret
                .backend_path
                .ends_with("cloud-aws/services/secrets/runtime-file/demo-token")
        );
        let metadata = fs::read_to_string(&secret.path).expect("secret metadata should read");
        assert!(metadata.contains("\"backend\": \"runtime-file\""));
        assert!(metadata.contains("\"materialization\": \"env\""));
        assert!(!metadata.contains("s3cr3t"));
        assert_eq!(
            fs::read_to_string(&secret.backend_path).expect("secret backend should read"),
            "s3cr3t"
        );
        assert_eq!(
            fs::metadata(&secret.backend_path)
                .expect("secret backend metadata should read")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        let cleanup_secret = put_machine_secret(
            &config,
            super::SecretPutRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "cleanup-token",
                value: "cleanup",
            },
        )
        .expect("second secret put should succeed");
        let listed_secrets = list_machine_secrets(&config, tempdir.path(), "cloud-aws")
            .expect("secret list should succeed");
        assert_eq!(listed_secrets.len(), 2);
        assert!(
            listed_secrets
                .iter()
                .all(|secret| secret.backend == ServiceSecretBackend::RuntimeFile)
        );
        assert!(
            listed_secrets
                .iter()
                .all(|secret| secret.materialization == ServiceSecretMaterialization::Env)
        );
        let removed = delete_machine_secret(&config, tempdir.path(), "cloud-aws", "cleanup-token")
            .expect("unused secret should delete");
        assert_eq!(removed.backend_path, cleanup_secret.backend_path);
        assert!(!removed.path.exists());
        assert!(!removed.backend_path.exists());

        let applied = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "buildbox",
                kind: ServiceKind::Sandbox,
                host_group: Some("aws-builders"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from(
                        "printf '%s\\n' \"$API_TOKEN\" >&2; trap 'exit 0' TERM; while :; do sleep 1; done",
                    ),
                ],
                secret_bindings: vec![ServiceSecretBinding {
                    env: String::from("API_TOKEN"),
                    secret: String::from("demo-token"),
                }],
                policy: ServicePolicy::default(),
            },
        )
        .expect("service apply should succeed");

        assert_eq!(applied.control_plane.as_deref(), Some("demo"));
        assert_eq!(applied.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(applied.runtime.state, ServiceRuntimeState::Running);
        assert_eq!(applied.target_host_group.as_deref(), Some("aws-builders"));
        assert_eq!(
            applied.host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(
            applied.runtime.stderr_path.as_deref(),
            Some(Path::new("/run/port/services/buildbox.stderr.log"))
        );
        assert_eq!(applied.secret_sources.len(), 1);
        assert_eq!(
            applied.secret_sources[0].backend,
            ServiceSecretBackend::RuntimeFile
        );
        assert_eq!(
            applied.secret_sources[0].materialization,
            ServiceSecretMaterialization::Env
        );
        assert_eq!(applied.secret_sources[0].path, secret.backend_path);
        assert!(!applied.secret_sources[0].detail.contains("s3cr3t"));

        let runtime_record = service_runtime_dir(&paths.runtime_dir).join("buildbox.json");
        for _ in 0..100 {
            if runtime_record.exists()
                && fs::read_to_string(&runtime_record)
                    .unwrap_or_default()
                    .contains("\"state\": \"running\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(runtime_record.exists());
        assert!(
            !fs::read_to_string(&runtime_record)
                .expect("runtime record should read")
                .contains("s3cr3t")
        );

        let listed = list_machine_services(&config, tempdir.path(), "cloud-aws")
            .expect("service list should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].runtime.state, ServiceRuntimeState::Running);
        assert_eq!(
            listed[0].host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(listed[0].secret_sources[0].path, secret.backend_path);

        let status = machine_service_status(&config, tempdir.path(), "cloud-aws", "buildbox")
            .expect("service status should succeed");
        assert_eq!(status.runtime.state, ServiceRuntimeState::Running);
        assert_eq!(
            status.host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(status.secret_sources.len(), 1);
        assert_eq!(status.secret_sources[0].path, secret.backend_path);
        assert!(!status.detail.contains("s3cr3t"));

        let mut stopped = stop_machine_service(&config, tempdir.path(), "cloud-aws", "buildbox")
            .expect("service stop should succeed");
        for _ in 0..100 {
            if stopped.runtime.state == ServiceRuntimeState::Stopped
                && stopped.runtime.exit_code.or(stopped.runtime.last_exit_code) == Some(0)
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
            stopped = machine_service_status(&config, tempdir.path(), "cloud-aws", "buildbox")
                .expect("service status should succeed after stop");
        }
        assert_eq!(stopped.runtime.state, ServiceRuntimeState::Stopped);
        assert_eq!(
            stopped.runtime.exit_code.or(stopped.runtime.last_exit_code),
            Some(0)
        );

        let runtime_record_contents =
            fs::read_to_string(&runtime_record).expect("runtime record should read");
        assert!(runtime_record_contents.contains("\"state\": \"stopped\""));
        assert!(!runtime_record_contents.contains("s3cr3t"));
    }

    #[test]
    fn hosted_service_apply_targets_requested_host_group_and_uses_deterministic_first_fit() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_service_config(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let primary_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let primary_paths = RuntimePaths::for_machine(&primary_runtime_root, "cloud-aws");
        write_manifest(&primary_paths, "cloud-aws", 1);

        let secondary_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let secondary_paths = RuntimePaths::for_machine(&secondary_runtime_root, "cloud-aws");
        write_manifest(&secondary_paths, "cloud-aws", 2);

        for guest_socket in [
            primary_paths.guest_agent_socket.clone(),
            secondary_paths.guest_agent_socket.clone(),
        ] {
            let guest_root = tempdir.path().join(format!(
                "guest-{}",
                guest_socket.display().to_string().replace('/', "_")
            ));
            fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
            thread::spawn(move || {
                serve_guest_agent(&guest_socket, guest_root).expect("guest agent should serve")
            });
        }
        for _ in 0..100 {
            if primary_paths.guest_agent_socket.exists()
                && secondary_paths.guest_agent_socket.exists()
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let config =
            start_named_live_hosted_servers(&config, &["aws-linux-node", "aws-linux-node-b"])
                .expect("hosted servers should start");

        let service_one = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-one",
                kind: ServiceKind::Service,
                host_group: Some("aws-builders"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect("first hosted service apply should succeed");
        let service_two = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-two",
                kind: ServiceKind::Sandbox,
                host_group: Some("aws-builders"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect("second hosted service apply should succeed");

        assert_eq!(service_one.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(service_two.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            service_one.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(
            service_two.target_host_group.as_deref(),
            Some("aws-builders")
        );
        assert_eq!(
            service_one.scheduler,
            Some(HostedSchedulerPolicy::DeterministicFirstFit)
        );
        assert_eq!(
            service_two.scheduler,
            Some(HostedSchedulerPolicy::DeterministicFirstFit)
        );

        assert!(
            service_definition_dir(&primary_paths.runtime_dir)
                .join("svc-one.json")
                .exists()
        );
        assert!(
            service_definition_dir(&primary_paths.runtime_dir)
                .join("svc-two.json")
                .exists()
        );
        assert!(
            !service_definition_dir(&secondary_paths.runtime_dir)
                .join("svc-one.json")
                .exists()
        );
        assert!(
            !service_definition_dir(&secondary_paths.runtime_dir)
                .join("svc-two.json")
                .exists()
        );
    }

    #[test]
    fn hosted_service_apply_reports_requested_host_group_rejection_detail() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_service_config(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        config
            .nodes
            .get_mut("aws-linux-node-b")
            .expect("aws-linux-node-b should exist")
            .capabilities
            .architectures = vec![MachineArchitecture::Aarch64];
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let config = start_named_live_hosted_servers(&config, &["aws-linux-node"])
            .expect("control plane should start");
        let error = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-fail",
                kind: ServiceKind::Service,
                host_group: Some("aws-secondary"),
                command: vec![String::from("/bin/true")],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect_err("requested host group should reject ineligible placement");

        let message = error.to_string();
        assert!(message.contains("aws-secondary"), "{message}");
        assert!(message.contains("aws-linux-node-b"), "{message}");
        assert!(
            message.contains("architecture 'x86_64' is required"),
            "{message}"
        );
    }

    #[test]
    fn hosted_service_list_status_and_stop_follow_stored_placement() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_service_config(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let primary_runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let primary_paths = RuntimePaths::for_machine(&primary_runtime_root, "cloud-aws");
        write_manifest(&primary_paths, "cloud-aws", 1);

        let secondary_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let secondary_paths = RuntimePaths::for_machine(&secondary_runtime_root, "cloud-aws");
        write_manifest(&secondary_paths, "cloud-aws", 2);

        for guest_socket in [
            primary_paths.guest_agent_socket.clone(),
            secondary_paths.guest_agent_socket.clone(),
        ] {
            let guest_root = tempdir.path().join(format!(
                "guest-{}",
                guest_socket.display().to_string().replace('/', "_")
            ));
            fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
            thread::spawn(move || {
                serve_guest_agent(&guest_socket, guest_root).expect("guest agent should serve")
            });
        }
        for _ in 0..100 {
            if primary_paths.guest_agent_socket.exists()
                && secondary_paths.guest_agent_socket.exists()
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let config =
            start_named_live_hosted_servers(&config, &["aws-linux-node", "aws-linux-node-b"])
                .expect("hosted servers should start");

        let applied = apply_machine_service(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-secondary",
                kind: ServiceKind::Service,
                host_group: Some("aws-secondary"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect("secondary placement should succeed");
        assert_eq!(applied.node_name.as_deref(), Some("aws-linux-node-b"));
        assert!(
            service_definition_dir(&secondary_paths.runtime_dir)
                .join("svc-secondary.json")
                .exists(),
            "secondary runtime should retain the applied service definition"
        );

        let listed = list_machine_services(&config, tempdir.path(), "cloud-aws")
            .expect("service list should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(
            listed[0].target_host_group.as_deref(),
            Some("aws-secondary")
        );

        let status = machine_service_status(&config, tempdir.path(), "cloud-aws", "svc-secondary")
            .expect("service status should succeed");
        assert_eq!(status.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(status.target_host_group.as_deref(), Some("aws-secondary"));
        assert_eq!(
            status.scheduler,
            Some(HostedSchedulerPolicy::DeterministicFirstFit)
        );

        let stopped = stop_machine_service(&config, tempdir.path(), "cloud-aws", "svc-secondary")
            .expect("service stop should succeed");
        assert_eq!(stopped.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(stopped.target_host_group.as_deref(), Some("aws-secondary"));
        assert_eq!(stopped.runtime.state, ServiceRuntimeState::Stopped);

        assert!(
            !service_definition_dir(&primary_paths.runtime_dir)
                .join("svc-secondary.json")
                .exists()
        );
        assert!(
            service_definition_dir(&secondary_paths.runtime_dir)
                .join("svc-secondary.json")
                .exists()
        );
    }

    #[test]
    fn hosted_stored_service_placements_follow_machine_placement_even_if_machine_host_drifts() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_service_config(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");

        let secondary_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let secondary_paths = RuntimePaths::for_machine(&secondary_runtime_root, "cloud-aws");
        write_manifest(&secondary_paths, "cloud-aws", 2);
        write_machine_placement_state(
            "demo",
            "cloud-aws",
            "aws-linux-node-b",
            &secondary_runtime_root,
            "stored on secondary node",
        );

        crate::apply_machine_service_local(
            &config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-secondary",
                kind: ServiceKind::Service,
                host_group: None,
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect("service definition should store under the machine's placed runtime root");

        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .host = String::from("gcp-linux");

        let placements =
            crate::hosted_stored_service_placements(&config, "cloud-aws", Some("svc-secondary"))
                .expect("stored placement lookup should follow persisted machine placement");
        assert_eq!(placements.len(), 1);
        assert_eq!(
            placements[0].status.node_name.as_deref(),
            Some("aws-linux-node-b")
        );
        assert_eq!(
            placements[0].status.manifest_path,
            service_definition_dir(&secondary_paths.runtime_dir).join("svc-secondary.json")
        );
    }

    #[test]
    fn hosted_service_output_surfaces_stale_stored_placement_detail() {
        let tempdir = tempdir().expect("tempdir should exist");
        let mut config = sample_multi_node_service_config(tempdir.path());
        config.machines.retain(|name, _| name == "cloud-aws");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let secondary_runtime_root = config.nodes["aws-linux-node-b"].runtime_root.clone();
        let secondary_paths = RuntimePaths::for_machine(&secondary_runtime_root, "cloud-aws");
        write_manifest(&secondary_paths, "cloud-aws", 2);

        let guest_root = tempdir.path().join("guest-secondary");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let guest_socket = secondary_paths.guest_agent_socket.clone();
        let guest_root_for_thread = guest_root.clone();
        thread::spawn(move || {
            serve_guest_agent(&guest_socket, guest_root_for_thread)
                .expect("guest agent should serve")
        });
        for _ in 0..100 {
            if secondary_paths.guest_agent_socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let applied_config = start_named_live_hosted_servers(&config, &["aws-linux-node-b"])
            .expect("secondary control plane should start");
        let _applied = apply_machine_service(
            &applied_config,
            ServiceApplyRequest {
                machine_name: "cloud-aws",
                runtime_root: tempdir.path(),
                name: "svc-stale",
                kind: ServiceKind::Sandbox,
                host_group: Some("aws-secondary"),
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("trap 'exit 0' TERM; while :; do sleep 1; done"),
                ],
                secret_bindings: Vec::new(),
                policy: ServicePolicy::default(),
            },
        )
        .expect("secondary placement should succeed");

        let _ = fs::remove_dir_all(hosted_placeholder_runtime_root("demo"));
        let stale_control_plane = start_live_control_plane_with_bindings(&config, Vec::new())
            .expect("stale control plane should start");
        let mut stale_config = config.clone();
        stale_config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane should exist")
            .endpoint = format!("http://{stale_control_plane}");

        let listed = list_machine_services(&stale_config, tempdir.path(), "cloud-aws")
            .expect("service list should still succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(
            listed[0].target_host_group.as_deref(),
            Some("aws-secondary")
        );
        assert!(
            listed[0]
                .detail
                .contains("no live registered node-agent endpoint for it"),
            "{}",
            listed[0].detail
        );

        let status =
            machine_service_status(&stale_config, tempdir.path(), "cloud-aws", "svc-stale")
                .expect("service status should still succeed");
        assert_eq!(status.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(status.target_host_group.as_deref(), Some("aws-secondary"));
        assert!(
            status
                .detail
                .contains("no live registered node-agent endpoint for it"),
            "{}",
            status.detail
        );

        let stopped = stop_machine_service(&stale_config, tempdir.path(), "cloud-aws", "svc-stale")
            .expect("service stop should return stored placement detail");
        assert_eq!(stopped.node_name.as_deref(), Some("aws-linux-node-b"));
        assert_eq!(stopped.target_host_group.as_deref(), Some("aws-secondary"));
        assert!(
            stopped
                .detail
                .contains("Stop request could not reach node 'aws-linux-node-b'"),
            "{}",
            stopped.detail
        );
        assert_eq!(stopped.desired_state, ServiceDesiredState::Active);
    }

    #[test]
    fn hosted_k3s_machine_truth_serde_round_trips_with_wedge_fields() {
        let populated = super::HostedK3sMachineTruth {
            role: String::from("worker"),
            machine_name: String::from("cloud-aws-worker-2"),
            node_name: Some(String::from("aws-linux-cell-1")),
            runtime_root: Some(PathBuf::from("/var/lib/port/aws-hosted/runtime")),
            network_identity: super::HostedK3sGuestNetworkIdentity {
                identity: String::from(
                    "port-hosted://prod/nodes/aws-linux-cell-1/machines/cloud-aws-worker-2",
                ),
                endpoint_ip: Some(std::net::IpAddr::from([3, 238, 162, 153])),
                endpoint_scope: super::HostedK3sGuestNetworkEndpointScope::UniquePerGuest,
                shared_with_machines: Vec::new(),
                detail: String::from("worker-2 has a unique execution-host endpoint"),
            },
            detail: String::from("worker placed on aws-linux-cell-1"),
            guest_refresh_age_seconds: Some(248),
            wedged_since_unix_s: Some(1_745_000_000),
            wedge_class: Some(String::from("guest")),
            recovery_attempts: super::RecoveryAttemptCounters {
                tier_1: 1,
                tier_2: 0,
                tier_3: 0,
            },
            last_recovery_action: Some(super::RecoveryActionRecord {
                tier: 1,
                timestamp_unix_s: 1_745_000_060,
                outcome: String::from("restart-issued"),
            }),
            recovery_state: super::RecoveryState::InProgress,
        };
        let rendered = serde_json::to_value(&populated).expect("populated truth should serialize");
        assert_eq!(
            rendered["network_identity"]["identity"],
            serde_json::json!(
                "port-hosted://prod/nodes/aws-linux-cell-1/machines/cloud-aws-worker-2"
            )
        );
        assert_eq!(
            rendered["wedged_since_unix_s"],
            serde_json::json!(1_745_000_000)
        );
        assert_eq!(rendered["wedge_class"], serde_json::json!("guest"));
        assert_eq!(rendered["recovery_state"], serde_json::json!("in-progress"));
        assert_eq!(
            rendered["recovery_attempts"]["tier_1"],
            serde_json::json!(1)
        );
        let decoded: super::HostedK3sMachineTruth =
            serde_json::from_value(rendered).expect("populated truth should round-trip");
        assert_eq!(decoded, populated);

        let bare = super::HostedK3sMachineTruth {
            role: String::from("control-plane"),
            machine_name: String::from("cloud-aws"),
            node_name: Some(String::from("aws-linux-cell-0")),
            runtime_root: None,
            network_identity: super::HostedK3sGuestNetworkIdentity {
                identity: String::from(
                    "port-hosted://prod/nodes/aws-linux-cell-0/machines/cloud-aws",
                ),
                endpoint_ip: None,
                endpoint_scope: super::HostedK3sGuestNetworkEndpointScope::Unresolved,
                shared_with_machines: Vec::new(),
                detail: String::from("control-plane endpoint is unresolved"),
            },
            detail: String::from("control-plane placed on aws-linux-cell-0"),
            guest_refresh_age_seconds: None,
            wedged_since_unix_s: None,
            wedge_class: None,
            recovery_attempts: super::RecoveryAttemptCounters::default(),
            last_recovery_action: None,
            recovery_state: super::RecoveryState::default(),
        };
        let rendered = serde_json::to_value(&bare).expect("bare truth should serialize");
        let object = rendered
            .as_object()
            .expect("payload should be a JSON object");
        for absent in [
            "guest_refresh_age_seconds",
            "wedged_since_unix_s",
            "wedge_class",
            "recovery_attempts",
            "last_recovery_action",
            "recovery_state",
        ] {
            assert!(
                !object.contains_key(absent),
                "{absent} should be omitted from the wire when default; got: {rendered}"
            );
        }
        let decoded: super::HostedK3sMachineTruth =
            serde_json::from_value(rendered).expect("bare truth should round-trip");
        assert_eq!(decoded, bare);
    }

    #[test]
    fn hosted_k3s_machine_truth_leaves_wedge_fields_default_when_wedge_route_unreachable() {
        let temp = tempdir().expect("temp dir should create");
        let runtime_root = temp.path().join("runtime");
        let config = PortConfig::sample();
        let access = vec![super::HostedK3sMachineAccess {
            role: String::from("worker"),
            route: HostedRouteContext {
                control_plane: Some(String::from("prod")),
                machine_name: Some(String::from("nonexistent-machine")),
                node_name: Some(String::from("aws-linux-cell-1")),
                runtime_root: Some(runtime_root.clone()),
                ..HostedRouteContext::default()
            },
            network_identity: super::HostedK3sGuestNetworkIdentity {
                identity: String::from(
                    "port-hosted://prod/nodes/aws-linux-cell-1/machines/nonexistent-machine",
                ),
                endpoint_ip: None,
                endpoint_scope: super::HostedK3sGuestNetworkEndpointScope::Unresolved,
                shared_with_machines: Vec::new(),
                detail: String::from("placeholder unresolved identity"),
            },
            detail: String::from("worker placement detail"),
        }];

        // No live control plane in scope: hosted_control_plane_machine_wedge
        // returns Err and the row builds with wedge defaults — same shape
        // we expect when the dedicated wedge route has nothing to report
        // for that machine.
        let truth = super::hosted_k3s_machine_truth(&config, &access);

        assert_eq!(truth.len(), 1);
        let row = &truth[0];
        assert_eq!(row.role, "worker");
        assert_eq!(row.machine_name, "nonexistent-machine");
        assert_eq!(
            row.network_identity.identity,
            "port-hosted://prod/nodes/aws-linux-cell-1/machines/nonexistent-machine"
        );
        assert_eq!(
            row.network_identity.endpoint_scope,
            super::HostedK3sGuestNetworkEndpointScope::Unresolved
        );
        assert_eq!(row.detail, "worker placement detail");
        assert_eq!(row.guest_refresh_age_seconds, None);
        assert_eq!(row.wedged_since_unix_s, None);
        assert_eq!(row.wedge_class, None);
        assert_eq!(
            row.recovery_attempts,
            super::RecoveryAttemptCounters::default()
        );
        assert_eq!(row.last_recovery_action, None);
        assert_eq!(row.recovery_state, super::RecoveryState::default());
    }

    #[test]
    fn machine_wedge_status_serde_round_trips_with_defaults_skipped_on_wire() {
        let bare = super::MachineWedgeStatus {
            machine_name: String::from("cloud-aws"),
            guest_refresh_age_seconds: None,
            wedged_since_unix_s: None,
            wedge_class: None,
            wedge_signal: None,
            hosted_k3s_service: None,
            recovery_attempts: super::RecoveryAttemptCounters::default(),
            last_recovery_action: None,
            recovery_state: super::RecoveryState::default(),
        };
        let rendered = serde_json::to_value(&bare).expect("bare wedge status should serialize");
        let object = rendered
            .as_object()
            .expect("wedge status should serialize as object");
        for absent in [
            "guest_refresh_age_seconds",
            "wedged_since_unix_s",
            "wedge_class",
            "wedge_signal",
            "hosted_k3s_service",
            "recovery_attempts",
            "last_recovery_action",
            "recovery_state",
        ] {
            assert!(
                !object.contains_key(absent),
                "{absent} should be omitted on the wire when default; got: {rendered}"
            );
        }
        let decoded: super::MachineWedgeStatus =
            serde_json::from_value(rendered).expect("bare wedge status should round-trip");
        assert_eq!(decoded, bare);

        let populated = super::MachineWedgeStatus {
            machine_name: String::from("cloud-aws-worker-2"),
            guest_refresh_age_seconds: Some(248),
            wedged_since_unix_s: Some(1_745_000_000),
            wedge_class: Some(String::from("guest")),
            wedge_signal: Some(super::MachineWedgeSignal::HostedK3sServiceRuntime),
            hosted_k3s_service: Some(super::MachineWedgeServiceEvidence {
                name: String::from("k3s-agent"),
                state: super::ServiceRuntimeState::Running,
                health_state: super::ServiceHealthState::Unhealthy,
                pid: Some(4321),
                exit_code: None,
                last_exit_code: Some(1),
                health_detail: Some(String::from("lease has not renewed within threshold")),
                last_exit_detail: Some(String::from("healthcheck command exited 1")),
            }),
            recovery_attempts: super::RecoveryAttemptCounters {
                tier_1: 1,
                tier_2: 0,
                tier_3: 0,
            },
            last_recovery_action: Some(super::RecoveryActionRecord {
                tier: 1,
                timestamp_unix_s: 1_745_000_060,
                outcome: String::from("restart-issued"),
            }),
            recovery_state: super::RecoveryState::InProgress,
        };
        let rendered =
            serde_json::to_value(&populated).expect("populated wedge status should serialize");
        assert_eq!(
            rendered["guest_refresh_age_seconds"],
            serde_json::json!(248)
        );
        assert_eq!(
            rendered["wedged_since_unix_s"],
            serde_json::json!(1_745_000_000)
        );
        assert_eq!(
            rendered["wedge_signal"],
            serde_json::json!("hosted-k3s-service-runtime")
        );
        assert_eq!(
            rendered["hosted_k3s_service"]["name"],
            serde_json::json!("k3s-agent")
        );
        assert_eq!(rendered["recovery_state"], serde_json::json!("in-progress"));
        let decoded: super::MachineWedgeStatus =
            serde_json::from_value(rendered).expect("populated wedge status should round-trip");
        assert_eq!(decoded, populated);
    }
}
