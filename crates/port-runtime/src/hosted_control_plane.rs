use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::{get, post, put};
use port_hosted_protocol::{
    HostedArtifactTransferRequest, HostedArtifactTransferResult, HostedClientHeaders,
    HostedControlPlaneRoute, HostedDetachedForwardRoute, HostedDetachedForwardStartRequest,
    HostedError, HostedGuestRoute, HostedGuestStreamRoute, HostedGuestVerb, HostedMachineRoute,
    HostedNodeAgentHeaders, HostedNodeRegistrationRequest, HostedNodeRoute,
    HostedRegistrationRoute, HostedRouteContext, HostedServiceRoute, HostedSuccess,
    PORT_ARTIFACT_TRANSFER_HEADER,
};
use port_model::{
    ExecutionSubstrate, FirecrackerPvmLaneContract, HostConnection, HostProvider,
    HostedAuthTokenSource, HostedImportedNodeRecord, HostedMachineSummaryContract,
    HostedNodeRegistration, HostedRegisteredNodeContract, MachineArchitecture, PortConfig,
    ProtectionMode, hosted_artifact_store_path,
};
use port_sdk::{SecretPutRequest, ServiceApplyRequest};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::{
    DetachedForwardLaunchRequest, GuestCopyRequest, GuestForwardRequest, GuestRequest,
    HostedFleetFreshnessState, HostedFleetNodeStatus, HostedFleetRoutingEligibility,
    HostedStoredServicePlacement, LaunchMetadata, LaunchRequest, MachineRuntimeState,
    MachineStatus, RuntimePaths, ServiceApplyRequest as RuntimeServiceApplyRequest,
    ServiceSecretBinding, StopResult, apply_hosted_machine_service_live, copy_guest_file,
    copy_guest_via_endpoint, delete_machine_secret_local, execute_guest_operation,
    hosted_placeholder_runtime_root, hosted_stored_service_placements, launch_local_machine,
    list_detached_forwards, list_machine_secrets_local, machine_monitor as runtime_machine_monitor,
    machine_monitor_report, machine_status as runtime_machine_status,
    machine_top as runtime_machine_top, machine_top_report, prepare_guest_forward,
    put_machine_secret_local, refresh_hosted_machine_service_list,
    refresh_hosted_machine_service_runtime, start_detached_forward, stop_detached_forward,
    stop_hosted_machine_service_live, stop_machine as runtime_stop_machine,
};
use port_agent_protocol::{
    CopyRequest, ForwardResult, GuestOperation, OperationResult, RequestEnvelope, ResponseEnvelope,
    StreamKind, read_frame, write_frame,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedNodeBinding {
    pub node_name: String,
    pub endpoint: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct ControlPlaneServeRequest {
    pub control_plane: String,
    pub bind: String,
    pub node_bindings: Vec<HostedNodeBinding>,
}

#[derive(Clone)]
struct ControlPlaneState {
    inner: Arc<ControlPlaneStateInner>,
}

struct ControlPlaneStateInner {
    config: PortConfig,
    control_plane: String,
    auth_header: String,
    auth_value: String,
    static_node_bindings: BTreeMap<String, HostedNodeBinding>,
    registered_state_path: PathBuf,
    registered_state: RwLock<RegisteredNodeStateFile>,
    registered_nodes: RwLock<BTreeMap<String, RegisteredNodeRecord>>,
    #[allow(dead_code)]
    imported_inventory_path: PathBuf,
    #[allow(dead_code)]
    imported_inventory_state: RwLock<ImportedInventoryStateFile>,
    #[allow(dead_code)]
    imported_inventory: RwLock<BTreeMap<String, ImportedNodeRecord>>,
    machine_placement_state_path: PathBuf,
    machine_placement_state: RwLock<MachinePlacementStateFile>,
    machine_placements: RwLock<BTreeMap<String, HostedMachinePlacementRecord>>,
    client: Client,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RegisteredNodeStateFile {
    control_plane: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    nodes: BTreeMap<String, HostedNodeRegistration>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ImportedInventoryStateFile {
    control_plane: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    nodes: BTreeMap<String, HostedImportedNodeRecord>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct MachinePlacementStateFile {
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

#[derive(Debug, Clone)]
pub struct NodeAgentServeRequest {
    pub node_name: String,
    pub bind: String,
    pub token: String,
}

#[derive(Clone)]
struct NodeAgentState {
    inner: Arc<NodeAgentStateInner>,
}

struct NodeAgentStateInner {
    config: PortConfig,
    node_name: String,
    runtime_root: std::path::PathBuf,
    bind: String,
    token: String,
}

#[derive(Debug, Clone)]
struct RegisteredNodeRecord {
    binding: HostedNodeBinding,
    contract: HostedRegisteredNodeContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedNodeRecord {
    node_name: String,
    provider: HostProvider,
    provenance: String,
    imported_at: u64,
    capability_summary: port_model::HostedNodeCapabilities,
}

#[derive(Debug, Clone)]
struct NodeAgentRegistrationTarget {
    control_plane: String,
    endpoint: String,
    node_endpoint: String,
    auth_headers: HostedClientHeaders,
}

#[cfg(test)]
const NODE_AGENT_REGISTRATION_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
#[cfg(not(test))]
const NODE_AGENT_REGISTRATION_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

#[cfg(test)]
const NODE_AGENT_REGISTRATION_TTL_SECONDS: u64 = 3;
#[cfg(not(test))]
const NODE_AGENT_REGISTRATION_TTL_SECONDS: u64 = 15;

trait HostedMachineProjection {
    fn apply_hosted_route(self, route: &HostedRouteContext) -> Self;
}

impl HostedMachineProjection for MachineStatus {
    fn apply_hosted_route(mut self, route: &HostedRouteContext) -> Self {
        self.control = port_model::MachineControlContract::hosted_control_plane();
        self.detail = append_route_detail(self.detail, route);
        self
    }
}

impl HostedMachineProjection for StopResult {
    fn apply_hosted_route(mut self, route: &HostedRouteContext) -> Self {
        self.control = port_model::MachineControlContract::hosted_control_plane();
        self.detail = append_route_detail(self.detail, route);
        self
    }
}

impl HostedMachineProjection for crate::MachineMonitorReport {
    fn apply_hosted_route(mut self, route: &HostedRouteContext) -> Self {
        self.control = port_model::MachineControlContract::hosted_control_plane();
        self.control_plane = route.control_plane.clone();
        self.node_name = route.node_name.clone();
        self.host_groups = route.host_groups.clone();
        self.detail = append_route_detail(self.detail, route);
        self
    }
}

impl HostedMachineProjection for crate::MachineTopReport {
    fn apply_hosted_route(mut self, route: &HostedRouteContext) -> Self {
        self.control = port_model::MachineControlContract::hosted_control_plane();
        self.control_plane = route.control_plane.clone();
        self.node_name = route.node_name.clone();
        self.host_groups = route.host_groups.clone();
        self.detail = append_route_detail(self.detail, route);
        self
    }
}

impl HostedMachineProjection for LaunchMetadata {
    fn apply_hosted_route(self, _route: &HostedRouteContext) -> Self {
        self
    }
}

fn append_route_detail(detail: String, route: &HostedRouteContext) -> String {
    let Some(control_plane) = route.control_plane.as_deref() else {
        return detail;
    };
    let Some(node_name) = route.node_name.as_deref() else {
        return detail;
    };
    let mut output =
        format!("{detail} Routed through control plane '{control_plane}' and node '{node_name}'.");
    if let Some(placement_detail) = route.placement_detail.as_deref() {
        if !placement_detail.is_empty() {
            output.push(' ');
            output.push_str(placement_detail);
        }
    }
    output
}

#[allow(dead_code)]
fn registered_node_state_path(control_plane: &str) -> PathBuf {
    hosted_placeholder_runtime_root(control_plane).join("registered-nodes.json")
}

#[allow(dead_code)]
fn imported_inventory_state_path(control_plane: &str) -> PathBuf {
    hosted_placeholder_runtime_root(control_plane).join("imported-inventory.json")
}

fn load_registered_node_state(
    path: &PathBuf,
    control_plane: &str,
) -> Result<RegisteredNodeStateFile> {
    if !path.exists() {
        return Ok(RegisteredNodeStateFile {
            control_plane: control_plane.to_string(),
            ..RegisteredNodeStateFile::default()
        });
    }

    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "failed to read registered node state at '{}'",
            path.display()
        )
    })?;
    let state: RegisteredNodeStateFile = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode registered node state at '{}'",
            path.display()
        )
    })?;
    Ok(state)
}

fn persist_registered_node_state(path: &PathBuf, state: &RegisteredNodeStateFile) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "registered node state path '{}' has no parent directory",
            path.display()
        )
    })?;
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create registered node state directory '{}'",
            parent.display()
        )
    })?;
    std::fs::write(
        path,
        serde_json::to_vec_pretty(state).context("failed to encode registered node state")?,
    )
    .with_context(|| format!("failed to write registered node state '{}'", path.display()))?;
    Ok(())
}

fn load_imported_inventory_state(
    path: &PathBuf,
    control_plane: &str,
) -> Result<ImportedInventoryStateFile> {
    if !path.exists() {
        return Ok(ImportedInventoryStateFile {
            control_plane: control_plane.to_string(),
            ..ImportedInventoryStateFile::default()
        });
    }

    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "failed to read imported inventory state at '{}'",
            path.display()
        )
    })?;
    let state: ImportedInventoryStateFile = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode imported inventory state at '{}'",
            path.display()
        )
    })?;
    Ok(state)
}

#[allow(dead_code)]
fn persist_imported_inventory_state(
    path: &PathBuf,
    state: &ImportedInventoryStateFile,
) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "imported inventory state path '{}' has no parent directory",
            path.display()
        )
    })?;
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create imported inventory state directory '{}'",
            parent.display()
        )
    })?;
    std::fs::write(
        path,
        serde_json::to_vec_pretty(state).context("failed to encode imported inventory state")?,
    )
    .with_context(|| {
        format!(
            "failed to write imported inventory state '{}'",
            path.display()
        )
    })?;
    Ok(())
}

#[allow(dead_code)]
fn machine_placement_state_path(control_plane: &str) -> PathBuf {
    hosted_placeholder_runtime_root(control_plane).join("machine-placements.json")
}

fn load_machine_placement_state(
    path: &PathBuf,
    control_plane: &str,
) -> Result<MachinePlacementStateFile> {
    if !path.exists() {
        return Ok(MachinePlacementStateFile {
            control_plane: control_plane.to_string(),
            ..MachinePlacementStateFile::default()
        });
    }

    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "failed to read machine placement state at '{}'",
            path.display()
        )
    })?;
    let state: MachinePlacementStateFile = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode machine placement state at '{}'",
            path.display()
        )
    })?;
    Ok(state)
}

fn persist_machine_placement_state(
    path: &PathBuf,
    state: &MachinePlacementStateFile,
) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "machine placement state path '{}' has no parent directory",
            path.display()
        )
    })?;
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create machine placement state directory '{}'",
            parent.display()
        )
    })?;
    std::fs::write(
        path,
        serde_json::to_vec_pretty(state).context("failed to encode machine placement state")?,
    )
    .with_context(|| {
        format!(
            "failed to write machine placement state '{}'",
            path.display()
        )
    })?;
    Ok(())
}

#[allow(dead_code)]
fn validate_machine_placement_state(
    config: &PortConfig,
    state: &MachinePlacementStateFile,
) -> Result<BTreeMap<String, HostedMachinePlacementRecord>> {
    if state.control_plane.trim().is_empty() {
        bail!("machine placement state must declare a non-empty control plane");
    }
    if !config.control_planes.contains_key(&state.control_plane) {
        bail!(
            "machine placement state references unknown control plane '{}'",
            state.control_plane
        );
    }

    let inventory = config.hosted_inventory_contract().map_err(|error| {
        anyhow!("machine placement state could not load hosted inventory: {error}")
    })?;
    let mut placements = BTreeMap::new();
    for (machine_name, placement) in &state.machines {
        let summary = config
            .hosted_machine_summary_contract(machine_name)
            .map_err(|error| {
                anyhow!(
                    "machine placement state could not resolve hosted machine '{}': {error}",
                    machine_name
                )
            })?
            .ok_or_else(|| {
                anyhow!(
                    "machine placement state references unknown hosted machine '{}'",
                    machine_name
                )
            })?;
        if summary.control_plane != state.control_plane {
            bail!(
                "machine placement state for '{}' belongs to control plane '{}', not '{}'",
                machine_name,
                summary.control_plane,
                state.control_plane
            );
        }
        let node = inventory.nodes.get(&placement.node_name).ok_or_else(|| {
            anyhow!(
                "machine placement state for '{}' references unknown node '{}'",
                machine_name,
                placement.node_name
            )
        })?;
        if node.control_plane != state.control_plane {
            bail!(
                "machine placement state for '{}' references node '{}' on control plane '{}', not '{}'",
                machine_name,
                placement.node_name,
                node.control_plane,
                state.control_plane
            );
        }
        if node.runtime_root != placement.runtime_root {
            bail!(
                "machine placement state for '{}' records runtime root '{}' for node '{}', but inventory now declares '{}'",
                machine_name,
                placement.runtime_root.display(),
                placement.node_name,
                node.runtime_root.display()
            );
        }
        placements.insert(machine_name.clone(), placement.clone());
    }

    Ok(placements)
}

#[allow(dead_code)]
fn validate_registered_node_state(
    config: &PortConfig,
    state: &RegisteredNodeStateFile,
) -> Result<BTreeMap<String, HostedRegisteredNodeContract>> {
    if state.control_plane.trim().is_empty() {
        bail!("registered node state must declare a non-empty control plane");
    }
    if !config.control_planes.contains_key(&state.control_plane) {
        bail!(
            "registered node state references unknown control plane '{}'",
            state.control_plane
        );
    }

    let inventory = config.hosted_inventory_contract().map_err(|error| {
        anyhow!("registered node state could not load hosted inventory: {error}")
    })?;
    let mut contracts = BTreeMap::new();
    for (node_name, registration) in &state.nodes {
        let contract = inventory
            .hosted_registered_node_contract(&state.control_plane, node_name, registration)
            .map_err(|error| {
                anyhow!(
                    "registered node state for control plane '{}' is invalid: {}",
                    state.control_plane,
                    error
                )
            })?;
        contracts.insert(node_name.clone(), contract);
    }
    Ok(contracts)
}

fn registered_node_records(
    config: &PortConfig,
    state: &RegisteredNodeStateFile,
) -> Result<BTreeMap<String, RegisteredNodeRecord>> {
    let contracts = validate_registered_node_state(config, state)?;
    let mut records = BTreeMap::new();
    for (node_name, contract) in contracts {
        let registration = state.nodes.get(&node_name).with_context(|| {
            format!("registered node '{}' is missing persisted state", node_name)
        })?;
        records.insert(
            node_name.clone(),
            RegisteredNodeRecord {
                binding: HostedNodeBinding {
                    node_name: node_name.clone(),
                    endpoint: registration.endpoint.clone(),
                    token: registration.token.clone(),
                },
                contract,
            },
        );
    }
    Ok(records)
}

fn imported_provider_label(provider: HostProvider) -> String {
    serde_json::to_string(&provider)
        .unwrap_or_else(|_| format!("{provider:?}"))
        .trim_matches('"')
        .to_string()
}

fn imported_inventory_records(
    config: &PortConfig,
    path: &PathBuf,
    state: &ImportedInventoryStateFile,
) -> Result<BTreeMap<String, ImportedNodeRecord>> {
    if state.control_plane.trim().is_empty() {
        bail!(
            "imported inventory state at '{}' must declare a non-empty control plane",
            path.display()
        );
    }
    if !config.control_planes.contains_key(&state.control_plane) {
        bail!(
            "imported inventory state at '{}' references unknown control plane '{}'",
            path.display(),
            state.control_plane
        );
    }

    let inventory = config.hosted_inventory_contract().map_err(|error| {
        anyhow!(
            "imported inventory state at '{}' could not load hosted inventory: {error}",
            path.display()
        )
    })?;
    let mut records = BTreeMap::new();
    for (node_name, imported) in &state.nodes {
        let configured = inventory.nodes.get(node_name).ok_or_else(|| {
            anyhow!(
                "imported inventory state at '{}' for control plane '{}' references unknown configured node '{}'",
                path.display(),
                state.control_plane,
                node_name
            )
        })?;
        if configured.control_plane != state.control_plane {
            bail!(
                "imported inventory state at '{}' for node '{}' belongs to control plane '{}', not '{}'",
                path.display(),
                node_name,
                configured.control_plane,
                state.control_plane
            );
        }
        if !configured
            .capabilities
            .providers
            .contains(&imported.provider)
        {
            bail!(
                "imported inventory state at '{}' for node '{}' conflicts on provider: imported '{}' is not permitted by configured capabilities",
                path.display(),
                node_name,
                imported_provider_label(imported.provider)
            );
        }
        if !imported.capability_summary.is_populated() {
            bail!(
                "imported inventory state at '{}' for node '{}' must declare a populated capability summary",
                path.display(),
                node_name
            );
        }
        if !imported
            .capability_summary
            .is_subset_of(&configured.capabilities)
        {
            bail!(
                "imported inventory state at '{}' for node '{}' conflicts on capability_summary with the configured inventory contract",
                path.display(),
                node_name
            );
        }
        records.insert(
            node_name.clone(),
            ImportedNodeRecord {
                node_name: node_name.clone(),
                provider: imported.provider,
                provenance: imported.provenance.clone(),
                imported_at: imported.imported_at,
                capability_summary: imported.capability_summary.clone(),
            },
        );
    }
    Ok(records)
}

fn current_unix_timestamp_seconds() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the unix epoch")?
        .as_secs())
}

pub fn serve_control_plane(config: PortConfig, request: ControlPlaneServeRequest) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build control-plane runtime")?;

    runtime.block_on(async move {
        let bind = request.bind.clone();
        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind control plane on '{bind}'"))?;
        let state = build_state(config, request)?;
        axum::serve(listener, control_plane_router(state))
            .await
            .context("control-plane server exited unexpectedly")
    })
}

fn build_state(config: PortConfig, request: ControlPlaneServeRequest) -> Result<ControlPlaneState> {
    let control_plane = config
        .control_planes
        .get(&request.control_plane)
        .with_context(|| {
            format!(
                "unknown control plane '{}' for control-plane serve",
                request.control_plane
            )
        })?;

    let token = match &control_plane.auth.source {
        HostedAuthTokenSource::Env { variable } => std::env::var(variable).with_context(|| {
            format!(
                "control plane '{}' expects token in environment variable '{}'",
                request.control_plane, variable
            )
        })?,
    };

    let auth_value = format!("Bearer {token}");
    let static_node_bindings = request
        .node_bindings
        .into_iter()
        .map(|binding| (binding.node_name.clone(), binding))
        .collect();

    let registered_state_path = registered_node_state_path(&request.control_plane);
    let registered_state =
        load_registered_node_state(&registered_state_path, &request.control_plane).with_context(
            || {
                format!(
                    "control plane '{}' could not load registered node state",
                    request.control_plane
                )
            },
        )?;
    let registered_nodes =
        registered_node_records(&config, &registered_state).with_context(|| {
            format!(
                "control plane '{}' could not validate registered node state",
                request.control_plane
            )
        })?;
    let imported_inventory_path = imported_inventory_state_path(&request.control_plane);
    let imported_inventory_state =
        load_imported_inventory_state(&imported_inventory_path, &request.control_plane)
            .with_context(|| {
                format!(
                    "control plane '{}' could not load imported inventory state",
                    request.control_plane
                )
            })?;
    let imported_inventory =
        imported_inventory_records(&config, &imported_inventory_path, &imported_inventory_state)
            .with_context(|| {
                format!(
                    "control plane '{}' could not validate imported inventory state",
                    request.control_plane
                )
            })?;
    let machine_placement_state_path = machine_placement_state_path(&request.control_plane);
    let machine_placement_state =
        load_machine_placement_state(&machine_placement_state_path, &request.control_plane)
            .with_context(|| {
                format!(
                    "control plane '{}' could not load machine placement state",
                    request.control_plane
                )
            })?;
    let machine_placements = machine_placement_state.machines.clone();

    let auth_header = control_plane.auth.header.clone();

    Ok(ControlPlaneState {
        inner: Arc::new(ControlPlaneStateInner {
            config,
            control_plane: request.control_plane,
            auth_header,
            auth_value,
            static_node_bindings,
            registered_state_path,
            registered_state: RwLock::new(registered_state),
            registered_nodes: RwLock::new(registered_nodes),
            imported_inventory_path,
            imported_inventory_state: RwLock::new(imported_inventory_state),
            imported_inventory: RwLock::new(imported_inventory),
            machine_placement_state_path,
            machine_placement_state: RwLock::new(machine_placement_state),
            machine_placements: RwLock::new(machine_placements),
            client: Client::new(),
        }),
    })
}

fn control_plane_router(state: ControlPlaneState) -> Router {
    Router::new()
        .route(
            "/v1/nodes/{node}/registration",
            post(node_registration_refresh),
        )
        .route(
            "/v1/artifacts:push",
            post(artifact_push).layer(DefaultBodyLimit::disable()),
        )
        .route("/v1/artifacts:pull", post(artifact_pull))
        .route("/v1/machines", get(list_machines))
        .route(
            "/v1/machines/{machine}",
            get(machine_status).post(machine_command),
        )
        .route("/v1/machines/{machine}/monitor", get(machine_monitor))
        .route("/v1/machines/{machine}/top", get(machine_top))
        .route("/v1/machines/{machine}/guest:exec", post(guest_exec))
        .route("/v1/machines/{machine}/guest:copy", post(guest_copy))
        .route(
            "/v1/machines/{machine}/guest:copy:stream",
            post(guest_copy_stream),
        )
        .route("/v1/machines/{machine}/guest:pty", post(guest_pty))
        .route("/v1/machines/{machine}/guest:logs", post(guest_logs))
        .route("/v1/machines/{machine}/guest:forward", post(guest_forward))
        .route(
            "/v1/machines/{machine}/guest:forward:detached",
            get(guest_forward_detached_list).post(guest_forward_detached_start),
        )
        .route(
            "/v1/machines/{machine}/guest:forward:detached/{forward}/stop",
            post(guest_forward_detached_stop),
        )
        .route("/v1/machines/{machine}/secrets", get(service_secret_list))
        .route(
            "/v1/machines/{machine}/secrets/{secret}",
            put(service_secret_put).delete(service_secret_remove),
        )
        .route(
            "/v1/machines/{machine}/services",
            get(service_list).post(service_apply),
        )
        .route(
            "/v1/machines/{machine}/services/{service}",
            get(service_status).post(service_command),
        )
        .with_state(state)
}

async fn artifact_push(
    State(state): State<ControlPlaneState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let route = hosted_artifact_route_context(&state);
    let request = match decode_artifact_transfer_header(&state, &headers, &route) {
        Ok(request) => request,
        Err(response) => return response,
    };
    let request = match canonicalize_artifact_transfer_request(&state, request, &route) {
        Ok(request) => request,
        Err(response) => return response,
    };

    let parent = match request.store_path.parent() {
        Some(parent) => parent,
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "{} has no parent directory",
                    artifact_store_detail(&state, &request)
                ),
                Some(route),
            );
        }
    };
    if let Err(error) = std::fs::create_dir_all(parent) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "failed to create parent directory '{}' for {}: {error}",
                parent.display(),
                artifact_store_detail(&state, &request)
            ),
            Some(route),
        );
    }
    if let Err(error) = std::fs::write(&request.store_path, &body) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "failed to write {}: {error}",
                artifact_store_detail(&state, &request)
            ),
            Some(route),
        );
    }

    json_response(
        StatusCode::OK,
        &HostedSuccess {
            route,
            result: HostedArtifactTransferResult {
                artifact_name: request.artifact_name,
                reference: request.reference,
                selector: request.selector,
                store_path: request.store_path,
                bytes_copied: body.len() as u64,
            },
        },
    )
}

async fn artifact_pull(
    State(state): State<ControlPlaneState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let route = hosted_artifact_route_context(&state);
    let request: HostedArtifactTransferRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "control plane '{}' received invalid artifact transfer JSON: {error}",
                    state.inner.control_plane
                ),
                Some(route),
            );
        }
    };
    let request = match canonicalize_artifact_transfer_request(&state, request, &route) {
        Ok(request) => request,
        Err(response) => return response,
    };

    match std::fs::read(&request.store_path) {
        Ok(bytes) => raw_response(StatusCode::OK, bytes, "application/octet-stream"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => error_response(
            StatusCode::NOT_FOUND,
            format!("{} was not found", artifact_store_detail(&state, &request)),
            Some(route),
        ),
        Err(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "failed to read {}: {error}",
                artifact_store_detail(&state, &request)
            ),
            Some(route),
        ),
    }
}

async fn node_registration_refresh(
    State(state): State<ControlPlaneState>,
    Path(node_name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let route = HostedRouteContext {
        control_plane: Some(state.inner.control_plane.clone()),
        node_name: Some(node_name.clone()),
        runtime_root: state
            .inner
            .config
            .nodes
            .get(&node_name)
            .map(|node| node.runtime_root.clone()),
        ..HostedRouteContext::default()
    };
    let request: HostedNodeRegistrationRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "control plane '{}' received invalid registration JSON for node '{}': {error}",
                    state.inner.control_plane, node_name
                ),
                Some(route),
            );
        }
    };
    if request.control_plane != state.inner.control_plane {
        return error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "control plane '{}' rejected registration for control plane '{}'",
                state.inner.control_plane, request.control_plane
            ),
            Some(route),
        );
    }
    if request.node_name != node_name {
        return error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "control plane '{}' rejected registration for node '{}' on route '{}'",
                state.inner.control_plane, request.node_name, node_name
            ),
            Some(route),
        );
    }

    match store_registered_node_refresh(&state, &node_name, request.registration) {
        Ok(record) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route
                    .with_selected_node(node_name, record.contract.node.runtime_root.clone()),
                result: record.contract,
            },
        ),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error, Some(route)),
    }
}

fn hosted_artifact_route_context(state: &ControlPlaneState) -> HostedRouteContext {
    HostedRouteContext {
        control_plane: Some(state.inner.control_plane.clone()),
        runtime_root: Some(hosted_placeholder_runtime_root(&state.inner.control_plane)),
        ..HostedRouteContext::default()
    }
}

fn decode_artifact_transfer_header(
    state: &ControlPlaneState,
    headers: &HeaderMap,
    route: &HostedRouteContext,
) -> std::result::Result<HostedArtifactTransferRequest, Response> {
    let Some(value) = headers.get(PORT_ARTIFACT_TRANSFER_HEADER) else {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "control plane '{}' requires '{}' for hosted artifact upload",
                state.inner.control_plane, PORT_ARTIFACT_TRANSFER_HEADER
            ),
            Some(route.clone()),
        ));
    };
    let header = match value.to_str() {
        Ok(header) => header,
        Err(error) => {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "control plane '{}' received invalid '{}' header: {error}",
                    state.inner.control_plane, PORT_ARTIFACT_TRANSFER_HEADER
                ),
                Some(route.clone()),
            ));
        }
    };
    serde_json::from_str(header).map_err(|error| {
        error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "control plane '{}' received invalid artifact transfer metadata in '{}': {error}",
                state.inner.control_plane, PORT_ARTIFACT_TRANSFER_HEADER
            ),
            Some(route.clone()),
        )
    })
}

fn canonicalize_artifact_transfer_request(
    state: &ControlPlaneState,
    request: HostedArtifactTransferRequest,
    route: &HostedRouteContext,
) -> std::result::Result<HostedArtifactTransferRequest, Response> {
    let canonical_store_path = hosted_artifact_store_path(
        &state.inner.control_plane,
        &request.reference,
        request.selector,
        &request.filename,
    );
    if request.store_path != canonical_store_path {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "{} requested non-canonical store path '{}'; canonical control-plane store path is '{}'",
                artifact_store_detail(state, &request),
                request.store_path.display(),
                canonical_store_path.display()
            ),
            Some(route.clone()),
        ));
    }
    Ok(HostedArtifactTransferRequest {
        store_path: canonical_store_path,
        ..request
    })
}

fn artifact_store_detail(
    state: &ControlPlaneState,
    request: &HostedArtifactTransferRequest,
) -> String {
    let endpoint = state
        .inner
        .config
        .control_planes
        .get(&state.inner.control_plane)
        .map(|spec| spec.endpoint.as_str())
        .unwrap_or("<unknown>");
    format!(
        "hosted-api artifact '{}' ({}, selector '{}') for control plane '{}' endpoint '{}' at '{}'",
        request.artifact_name,
        request.reference,
        hosted_artifact_selector_label(request.selector),
        state.inner.control_plane,
        endpoint,
        request.store_path.display()
    )
}

fn hosted_artifact_selector_label(selector: port_model::ArtifactSelector) -> String {
    format!(
        "{}/{}/{}",
        hosted_artifact_architecture_label(selector.architecture),
        hosted_artifact_substrate_label(selector.substrate),
        hosted_artifact_protection_mode_label(selector.protection_mode)
    )
}

fn hosted_artifact_architecture_label(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => "native",
        MachineArchitecture::X86_64 => "x86_64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn hosted_artifact_substrate_label(substrate: ExecutionSubstrate) -> &'static str {
    match substrate {
        ExecutionSubstrate::Firecracker => "firecracker",
        ExecutionSubstrate::CloudHypervisor => "cloud-hypervisor",
        ExecutionSubstrate::Avf => "avf",
    }
}

fn hosted_artifact_protection_mode_label(mode: ProtectionMode) -> &'static str {
    match mode {
        ProtectionMode::Standard => "standard",
        ProtectionMode::Pvm => "pvm",
    }
}

async fn list_machines(State(state): State<ControlPlaneState>, headers: HeaderMap) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let mut machines = Vec::new();
    for machine_name in state.inner.config.machines.keys() {
        let Ok(Some(summary)) = state
            .inner
            .config
            .hosted_machine_summary_contract(machine_name)
        else {
            continue;
        };
        if summary.control_plane != state.inner.control_plane {
            continue;
        }
        match resolve_machine_binding(&state, &summary) {
            Ok((Some(binding), route, None)) => {
                let status_route = HostedNodeRoute::Machine(HostedMachineRoute::Status {
                    machine_name: machine_name.clone(),
                });
                match proxy_json::<HostedSuccess<MachineStatus>>(
                    &state,
                    &binding,
                    status_route,
                    Method::GET,
                    None,
                    route.clone(),
                )
                .await
                {
                    Ok(status) => match annotate_machine_status_with_fleet_state(
                        &state,
                        &summary,
                        &status.route,
                        status.result,
                    ) {
                        Ok(status) => machines.push(status),
                        Err(message) => {
                            return error_response(
                                StatusCode::BAD_GATEWAY,
                                message,
                                Some(HostedRouteContext::from_machine_summary(&summary)),
                            );
                        }
                    },
                    Err(message) => {
                        let status = malformed_machine_status(&summary, route.clone(), message);
                        match annotate_machine_status_with_fleet_state(
                            &state, &summary, &route, status,
                        ) {
                            Ok(status) => machines.push(status),
                            Err(message) => {
                                return error_response(
                                    StatusCode::BAD_GATEWAY,
                                    message,
                                    Some(HostedRouteContext::from_machine_summary(&summary)),
                                );
                            }
                        }
                    }
                }
            }
            Ok((None, route, Some(message))) => match annotate_machine_status_with_fleet_state(
                &state,
                &summary,
                &route,
                malformed_machine_status(&summary, route.clone(), message),
            ) {
                Ok(status) => machines.push(status),
                Err(message) => {
                    return error_response(
                        StatusCode::BAD_GATEWAY,
                        message,
                        Some(HostedRouteContext::from_machine_summary(&summary)),
                    );
                }
            },
            Ok((Some(_), _, Some(_))) | Ok((None, _, None)) => {
                let route = HostedRouteContext::from_machine_summary(&summary);
                let status = malformed_machine_status(
                    &summary,
                    route.clone(),
                    format!(
                        "control plane '{}' resolved an inconsistent routing state for machine '{}'",
                        state.inner.control_plane, machine_name
                    ),
                );
                match annotate_machine_status_with_fleet_state(&state, &summary, &route, status) {
                    Ok(status) => machines.push(status),
                    Err(message) => {
                        return error_response(StatusCode::BAD_GATEWAY, message, Some(route));
                    }
                }
            }
            Err(message) => {
                return error_response(StatusCode::BAD_GATEWAY, message, None);
            }
        }
    }

    json_response(
        StatusCode::OK,
        &HostedSuccess {
            route: HostedRouteContext {
                control_plane: Some(state.inner.control_plane.clone()),
                ..HostedRouteContext::default()
            },
            result: machines,
        },
    )
}

async fn machine_launch(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let (binding, route_context) = match resolve_node_binding(&state, &summary) {
        Ok(result) => result,
        Err((route_context, message)) => {
            return error_response(StatusCode::BAD_GATEWAY, message, Some(route_context));
        }
    };

    let launch_route = HostedNodeRoute::Machine(HostedMachineRoute::Launch {
        machine_name: machine.clone(),
    });
    match proxy_json::<HostedSuccess<LaunchMetadata>>(
        &state,
        &binding,
        launch_route,
        Method::POST,
        None,
        route_context.clone(),
    )
    .await
    {
        Ok(success) => {
            if let Err(message) = store_machine_placement(
                &state,
                &machine,
                &route_context,
                success.result.launched_at_unix_s,
            ) {
                return error_response(StatusCode::BAD_GATEWAY, message, Some(route_context));
            }
            json_response(
                StatusCode::OK,
                &HostedSuccess {
                    route: route_context,
                    result: success.result,
                },
            )
        }
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, Some(route_context)),
    }
}

async fn machine_status(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    match resolve_machine_binding(&state, &summary) {
        Ok((Some(binding), route, None)) => {
            let status_route = HostedNodeRoute::Machine(HostedMachineRoute::Status {
                machine_name: machine.clone(),
            });
            match proxy_json::<HostedSuccess<MachineStatus>>(
                &state,
                &binding,
                status_route,
                Method::GET,
                None,
                route.clone(),
            )
            .await
            {
                Ok(status) => match annotate_machine_status_with_fleet_state(
                    &state,
                    &summary,
                    &status.route,
                    status.result,
                ) {
                    Ok(result) => json_response(
                        StatusCode::OK,
                        &HostedSuccess {
                            route: status.route,
                            result,
                        },
                    ),
                    Err(message) => error_response(StatusCode::BAD_GATEWAY, message, Some(route)),
                },
                Err(message) => {
                    let status = malformed_machine_status(&summary, route.clone(), message);
                    match annotate_machine_status_with_fleet_state(&state, &summary, &route, status)
                    {
                        Ok(result) => json_response(
                            StatusCode::OK,
                            &HostedSuccess {
                                route: route.clone(),
                                result,
                            },
                        ),
                        Err(message) => {
                            error_response(StatusCode::BAD_GATEWAY, message, Some(route))
                        }
                    }
                }
            }
        }
        Ok((None, route, Some(message))) => {
            let status = malformed_machine_status(&summary, route.clone(), message);
            match annotate_machine_status_with_fleet_state(&state, &summary, &route, status) {
                Ok(result) => json_response(
                    StatusCode::OK,
                    &HostedSuccess {
                        route: route.clone(),
                        result,
                    },
                ),
                Err(message) => error_response(StatusCode::BAD_GATEWAY, message, Some(route)),
            }
        }
        Ok((Some(_), _, Some(_))) | Ok((None, _, None)) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' resolved an inconsistent routing state for machine '{}'",
                state.inner.control_plane, machine
            ),
            Some(HostedRouteContext::from_machine_summary(&summary)),
        ),
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, None),
    }
}

async fn machine_monitor(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    match resolve_machine_binding(&state, &summary) {
        Ok((Some(binding), route, None)) => {
            match proxy_json::<HostedSuccess<crate::MachineMonitorReport>>(
                &state,
                &binding,
                HostedNodeRoute::Machine(HostedMachineRoute::Monitor {
                    machine_name: machine.clone(),
                }),
                Method::GET,
                None,
                route.clone(),
            )
            .await
            {
                Ok(report) => json_response(StatusCode::OK, &report),
                Err(message) => render_unavailable_machine_monitor(&summary, route, message),
            }
        }
        Ok((None, route, Some(message))) => {
            render_unavailable_machine_monitor(&summary, route, message)
        }
        Ok((Some(_), _, Some(_))) | Ok((None, _, None)) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' resolved an inconsistent routing state for machine '{}'",
                state.inner.control_plane, machine
            ),
            Some(HostedRouteContext::from_machine_summary(&summary)),
        ),
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, None),
    }
}

async fn machine_top(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }

    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    match resolve_machine_binding(&state, &summary) {
        Ok((Some(binding), route, None)) => {
            match proxy_json::<HostedSuccess<crate::MachineTopReport>>(
                &state,
                &binding,
                HostedNodeRoute::Machine(HostedMachineRoute::Top {
                    machine_name: machine.clone(),
                }),
                Method::GET,
                None,
                route.clone(),
            )
            .await
            {
                Ok(report) => json_response(StatusCode::OK, &report),
                Err(message) => render_unavailable_machine_top(&summary, route, message),
            }
        }
        Ok((None, route, Some(message))) => {
            render_unavailable_machine_top(&summary, route, message)
        }
        Ok((Some(_), _, Some(_))) | Ok((None, _, None)) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' resolved an inconsistent routing state for machine '{}'",
                state.inner.control_plane, machine
            ),
            Some(HostedRouteContext::from_machine_summary(&summary)),
        ),
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, None),
    }
}

async fn machine_command(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(machine_name) = machine.strip_suffix(":launch") {
        return machine_launch(State(state), Path(machine_name.to_string()), headers).await;
    }

    if let Some(machine_name) = machine.strip_suffix(":stop") {
        if let Some(response) = authorize(&state, &headers) {
            return response;
        }
        let summary = match resolve_summary(&state, machine_name) {
            Ok(summary) => summary,
            Err(response) => return response,
        };
        return match resolve_machine_binding(&state, &summary) {
            Ok((Some(binding), route, None)) => match proxy_json::<HostedSuccess<StopResult>>(
                &state,
                &binding,
                HostedNodeRoute::Machine(HostedMachineRoute::Stop {
                    machine_name: machine_name.to_string(),
                }),
                Method::POST,
                None,
                route.clone(),
            )
            .await
            {
                Ok(result) => json_response(StatusCode::OK, &result),
                Err(message) => json_response(
                    StatusCode::OK,
                    &HostedSuccess {
                        route: route.clone(),
                        result: malformed_stop_result(&summary, route, message),
                    },
                ),
            },
            Ok((None, route, Some(message))) => json_response(
                StatusCode::OK,
                &HostedSuccess {
                    route: route.clone(),
                    result: malformed_stop_result(&summary, route, message),
                },
            ),
            Ok((Some(_), _, Some(_))) | Ok((None, _, None)) => error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "control plane '{}' resolved an inconsistent routing state for machine '{}'",
                    state.inner.control_plane, machine_name
                ),
                Some(HostedRouteContext::from_machine_summary(&summary)),
            ),
            Err(message) => error_response(StatusCode::BAD_GATEWAY, message, None),
        };
    }

    error_response(
        StatusCode::NOT_FOUND,
        format!(
            "control plane '{}' only serves launch and stop through '/v1/machines/{{machine}}:launch' and '/v1/machines/{{machine}}:stop'",
            state.inner.control_plane
        ),
        Some(HostedRouteContext {
            control_plane: Some(state.inner.control_plane.clone()),
            machine_name: Some(machine),
            ..HostedRouteContext::default()
        }),
    )
}

async fn guest_exec(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_route(&state, &headers, &machine, HostedGuestVerb::Exec, body).await
}

async fn guest_copy(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_route(&state, &headers, &machine, HostedGuestVerb::Copy, body).await
}

async fn guest_copy_stream(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_stream_route(&state, &headers, &machine, HostedGuestVerb::Copy, body).await
}

async fn guest_pty(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_route(&state, &headers, &machine, HostedGuestVerb::Pty, body).await
}

async fn guest_logs(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_route(&state, &headers, &machine, HostedGuestVerb::Logs, body).await
}

async fn guest_forward(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_guest_route(&state, &headers, &machine, HostedGuestVerb::Forward, body).await
}

async fn guest_forward_detached_start(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::DetachedForward(HostedDetachedForwardRoute::Start {
            machine_name: machine.clone(),
        }),
        Method::POST,
        Some(body),
    )
    .await
}

async fn guest_forward_detached_list(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::DetachedForward(HostedDetachedForwardRoute::List {
            machine_name: machine.clone(),
        }),
        Method::GET,
        None,
    )
    .await
}

async fn guest_forward_detached_stop(
    State(state): State<ControlPlaneState>,
    Path((machine, forward)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::DetachedForward(HostedDetachedForwardRoute::Stop {
            machine_name: machine.clone(),
            forward_name: forward,
        }),
        Method::POST,
        None,
    )
    .await
}

async fn service_secret_put(
    State(state): State<ControlPlaneState>,
    Path((machine, secret)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Service(HostedServiceRoute::SecretPut {
            machine_name: machine.clone(),
            secret_name: secret,
        }),
        Method::PUT,
        Some(body),
    )
    .await
}

async fn service_secret_list(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Service(HostedServiceRoute::SecretList {
            machine_name: machine.clone(),
        }),
        Method::GET,
        None,
    )
    .await
}

async fn service_secret_remove(
    State(state): State<ControlPlaneState>,
    Path((machine, secret)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Service(HostedServiceRoute::SecretRemove {
            machine_name: machine.clone(),
            secret_name: secret,
        }),
        Method::DELETE,
        None,
    )
    .await
}

async fn service_apply(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }
    let request: ServiceApplyRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "control plane '{}' received invalid service JSON for machine '{}': {error}",
                    state.inner.control_plane, machine
                ),
                Some(HostedRouteContext {
                    control_plane: Some(state.inner.control_plane.clone()),
                    machine_name: Some(machine),
                    ..HostedRouteContext::default()
                }),
            );
        }
    };
    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let (binding, route_context) =
        match resolve_service_apply_binding(&state, &summary, request.host_group.as_deref()) {
            Ok(result) => result,
            Err((route_context, message)) => {
                return error_response(StatusCode::BAD_GATEWAY, message, Some(route_context));
            }
        };

    proxy_raw(
        &state,
        &binding,
        HostedNodeRoute::Service(HostedServiceRoute::Apply {
            machine_name: machine,
        }),
        Method::POST,
        Some(body),
        route_context.with_service_name(request.name),
    )
    .await
}

async fn service_list(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }
    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let route = HostedRouteContext::from_machine_summary(&summary);
    let placements = match hosted_stored_service_placements(&state.inner.config, &machine, None) {
        Ok(placements) => placements,
        Err(error) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "control plane '{}' could not inspect stored service placement for machine '{}': {error}",
                    state.inner.control_plane, machine
                ),
                Some(route),
            );
        }
    };

    let mut services = Vec::new();
    for placement in placements {
        services.push(refresh_or_stored_service_status(&state, &machine, placement).await);
    }
    services.sort_by(|left, right| left.name.cmp(&right.name));
    json_response(
        StatusCode::OK,
        &HostedSuccess {
            route,
            result: services,
        },
    )
}

async fn service_status(
    State(state): State<ControlPlaneState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = authorize(&state, &headers) {
        return response;
    }
    let summary = match resolve_summary(&state, &machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let route =
        HostedRouteContext::from_machine_summary(&summary).with_service_name(service.clone());
    let placements = match hosted_stored_service_placements(
        &state.inner.config,
        &machine,
        Some(&service),
    ) {
        Ok(placements) => placements,
        Err(error) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "control plane '{}' could not inspect stored placement for service '{}' on machine '{}': {error}",
                    state.inner.control_plane, service, machine
                ),
                Some(route),
            );
        }
    };
    match placements.len() {
        0 => error_response(
            StatusCode::NOT_FOUND,
            format!(
                "control plane '{}' could not find stored placement for service '{}' on machine '{}'. {}",
                state.inner.control_plane, service, machine, summary.placement_detail
            ),
            Some(route),
        ),
        1 => {
            let placement = placements
                .into_iter()
                .next()
                .expect("single placement must exist");
            let response = refresh_or_stored_service_status(&state, &machine, placement).await;
            json_response(
                StatusCode::OK,
                &HostedSuccess {
                    route: stored_service_route_context(&response),
                    result: response,
                },
            )
        }
        _ => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' found multiple stored placements for service '{}' on machine '{}'",
                state.inner.control_plane, service, machine
            ),
            Some(route),
        ),
    }
}

async fn service_command(
    State(state): State<ControlPlaneState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(service_name) = service.strip_suffix(":stop") {
        if let Some(response) = authorize(&state, &headers) {
            return response;
        }
        let summary = match resolve_summary(&state, &machine) {
            Ok(summary) => summary,
            Err(response) => return response,
        };
        let route = HostedRouteContext::from_machine_summary(&summary)
            .with_service_name(service_name.to_string());
        let placements = match hosted_stored_service_placements(
            &state.inner.config,
            &machine,
            Some(service_name),
        ) {
            Ok(placements) => placements,
            Err(error) => {
                return error_response(
                    StatusCode::BAD_GATEWAY,
                    format!(
                        "control plane '{}' could not inspect stored placement for service '{}' on machine '{}': {error}",
                        state.inner.control_plane, service_name, machine
                    ),
                    Some(route),
                );
            }
        };
        if placements.is_empty() {
            return error_response(
                StatusCode::NOT_FOUND,
                format!(
                    "control plane '{}' could not find stored placement for service '{}' on machine '{}'. {}",
                    state.inner.control_plane, service_name, machine, summary.placement_detail
                ),
                Some(route),
            );
        }
        if placements.len() > 1 {
            return error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "control plane '{}' found multiple stored placements for service '{}' on machine '{}'",
                    state.inner.control_plane, service_name, machine
                ),
                Some(route),
            );
        }

        let placement = placements
            .into_iter()
            .next()
            .expect("single placement must exist");
        let Some(node_name) = placement.status.node_name.clone() else {
            return json_response(
                StatusCode::OK,
                &HostedSuccess {
                    route: stored_service_route_context(&placement.status),
                    result: placement.status,
                },
            );
        };
        let Some((binding, _)) = resolve_known_node_binding(&state, &node_name)
            .ok()
            .flatten()
        else {
            let mut status = placement.status;
            status.detail = format!(
                "{} Stop request could not reach node '{}' because the control plane has no live registered node-agent endpoint for it.",
                status.detail, node_name
            );
            return json_response(
                StatusCode::OK,
                &HostedSuccess {
                    route: stored_service_route_context(&status),
                    result: status,
                },
            );
        };

        let route_context = stored_service_route_context(&placement.status);
        return match proxy_json::<HostedSuccess<crate::ServiceDefinitionStatus>>(
            &state,
            &binding,
            HostedNodeRoute::Service(HostedServiceRoute::Stop {
                machine_name: machine.clone(),
                service_name: service_name.to_string(),
            }),
            Method::POST,
            None,
            route_context.clone(),
        )
        .await
        {
            Ok(success) => json_response(StatusCode::OK, &success),
            Err(message) => {
                let mut status = placement.status;
                status.detail = format!(
                    "{} Stop request could not refresh node '{}': {message}",
                    status.detail, node_name
                );
                json_response(
                    StatusCode::OK,
                    &HostedSuccess {
                        route: route_context,
                        result: status,
                    },
                )
            }
        };
    }

    error_response(
        StatusCode::NOT_FOUND,
        format!(
            "control plane '{}' only serves service stop through '/v1/machines/{{machine}}/services/{{service}}:stop'",
            state.inner.control_plane
        ),
        Some(
            HostedRouteContext {
                control_plane: Some(state.inner.control_plane.clone()),
                machine_name: Some(machine),
                ..HostedRouteContext::default()
            }
            .with_service_name(service),
        ),
    )
}

fn stored_service_route_context(service: &crate::ServiceDefinitionStatus) -> HostedRouteContext {
    HostedRouteContext {
        control_plane: service.control_plane.clone(),
        machine_name: Some(service.machine_name.clone()),
        forward_name: None,
        service_name: Some(service.name.clone()),
        node_name: service.node_name.clone(),
        candidate_nodes: Vec::new(),
        host_groups: service.host_groups.clone(),
        host_group_policies: service.host_group_policies.clone(),
        rejected_nodes: BTreeMap::new(),
        placement_detail: Some(service.detail.clone()),
        runtime_root: service
            .manifest_path
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf),
        inventory_owner: Some(service.control.inventory_owner),
        lifecycle_owner: Some(service.control.lifecycle_owner),
        guest_broker: Some(service.control.guest_broker),
    }
}

async fn refresh_or_stored_service_status(
    state: &ControlPlaneState,
    machine_name: &str,
    placement: HostedStoredServicePlacement,
) -> crate::ServiceDefinitionStatus {
    let Some(node_name) = placement.status.node_name.clone() else {
        return placement.status;
    };
    let Some((binding, _)) = resolve_known_node_binding(state, &node_name).ok().flatten() else {
        let mut status = placement.status;
        status.detail = format!(
            "{} Stored placement points at node '{}' but the control plane has no live registered node-agent endpoint for it.",
            status.detail, node_name
        );
        return status;
    };

    match proxy_json::<HostedSuccess<crate::ServiceDefinitionStatus>>(
        state,
        &binding,
        HostedNodeRoute::Service(HostedServiceRoute::Status {
            machine_name: machine_name.to_string(),
            service_name: placement.status.name.clone(),
        }),
        Method::GET,
        None,
        stored_service_route_context(&placement.status),
    )
    .await
    {
        Ok(success) => success.result,
        Err(message) => {
            let mut status = placement.status;
            status.detail = format!(
                "{} Stored placement on node '{}' could not be refreshed: {message}",
                status.detail, node_name
            );
            status
        }
    }
}

async fn proxy_guest_route(
    state: &ControlPlaneState,
    headers: &HeaderMap,
    machine: &str,
    verb: HostedGuestVerb,
    body: Bytes,
) -> Response {
    proxy_machine_route(
        state,
        headers,
        machine,
        HostedNodeRoute::Guest(HostedGuestRoute {
            machine_name: machine.to_string(),
            verb,
        }),
        Method::POST,
        Some(body),
    )
    .await
}

async fn proxy_guest_stream_route(
    state: &ControlPlaneState,
    headers: &HeaderMap,
    machine: &str,
    verb: HostedGuestVerb,
    body: Bytes,
) -> Response {
    if let Some(response) = authorize(state, headers) {
        return response;
    }

    let summary = match resolve_summary(state, machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let (binding, route_context) = match resolve_node_binding(state, &summary) {
        Ok(result) => result,
        Err((route_context, message)) => {
            return error_response(StatusCode::BAD_GATEWAY, message, Some(route_context));
        }
    };

    match proxy_bytes(
        state,
        &binding,
        HostedNodeRoute::GuestStream(HostedGuestStreamRoute {
            machine_name: machine.to_string(),
            verb,
        }),
        Method::POST,
        Some(body),
        Some("application/octet-stream"),
        route_context.clone(),
    )
    .await
    {
        Ok((status, bytes)) => raw_response(
            status,
            bytes,
            if status.is_success() {
                "application/octet-stream"
            } else {
                "application/json"
            },
        ),
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, Some(route_context)),
    }
}

async fn proxy_machine_route(
    state: &ControlPlaneState,
    headers: &HeaderMap,
    machine: &str,
    route: HostedNodeRoute,
    method: Method,
    body: Option<Bytes>,
) -> Response {
    if let Some(response) = authorize(state, headers) {
        return response;
    }

    let summary = match resolve_summary(state, machine) {
        Ok(summary) => summary,
        Err(response) => return response,
    };
    let (binding, route_context) = match resolve_node_binding(state, &summary) {
        Ok(result) => result,
        Err((route_context, message)) => {
            return error_response(StatusCode::BAD_GATEWAY, message, Some(route_context));
        }
    };

    proxy_raw(state, &binding, route, method, body, route_context).await
}

fn store_registered_node_refresh(
    state: &ControlPlaneState,
    node_name: &str,
    registration: HostedNodeRegistration,
) -> Result<RegisteredNodeRecord, String> {
    let current_state = state
        .inner
        .registered_state
        .read()
        .map_err(|_| {
            format!(
                "control plane '{}' could not inspect registered node state",
                state.inner.control_plane
            )
        })?
        .clone();
    if let Some(existing) = current_state.nodes.get(node_name)
        && registration.refreshed_at < existing.refreshed_at
    {
        return Err(format!(
            "control plane '{}' rejected stale registration refresh for node '{}': refreshed_at {} is older than current {}",
            state.inner.control_plane, node_name, registration.refreshed_at, existing.refreshed_at
        ));
    }

    let mut next_state = current_state;
    next_state.control_plane = state.inner.control_plane.clone();
    next_state.nodes.insert(node_name.to_string(), registration);
    let next_records = registered_node_records(&state.inner.config, &next_state)
        .map_err(|error| error.to_string())?;
    let record = next_records.get(node_name).cloned().ok_or_else(|| {
        format!(
            "control plane '{}' could not derive a registered-node record for '{}'",
            state.inner.control_plane, node_name
        )
    })?;
    persist_registered_node_state(&state.inner.registered_state_path, &next_state)
        .map_err(|error| error.to_string())?;

    *state.inner.registered_state.write().map_err(|_| {
        format!(
            "control plane '{}' could not update registered node state",
            state.inner.control_plane
        )
    })? = next_state;
    *state.inner.registered_nodes.write().map_err(|_| {
        format!(
            "control plane '{}' could not update registered node records",
            state.inner.control_plane
        )
    })? = next_records;
    Ok(record)
}

fn store_machine_placement(
    state: &ControlPlaneState,
    machine_name: &str,
    route_context: &HostedRouteContext,
    launched_at_unix_s: u64,
) -> Result<HostedMachinePlacementRecord, String> {
    let node_name = route_context.node_name.clone().ok_or_else(|| {
        format!(
            "control plane '{}' resolved machine '{}' without a selected node",
            state.inner.control_plane, machine_name
        )
    })?;
    let runtime_root = route_context.runtime_root.clone().ok_or_else(|| {
        format!(
            "control plane '{}' resolved machine '{}' without a selected runtime root",
            state.inner.control_plane, machine_name
        )
    })?;

    let current_state = state
        .inner
        .machine_placement_state
        .read()
        .map_err(|_| {
            format!(
                "control plane '{}' could not inspect machine placement state",
                state.inner.control_plane
            )
        })?
        .clone();
    let placement = HostedMachinePlacementRecord {
        node_name,
        runtime_root,
        placed_at_unix_s: launched_at_unix_s,
        placement_detail: route_context.placement_detail.clone(),
    };

    let mut next_state = current_state;
    next_state.control_plane = state.inner.control_plane.clone();
    next_state
        .machines
        .insert(machine_name.to_string(), placement.clone());
    let next_placements = next_state.machines.clone();
    persist_machine_placement_state(&state.inner.machine_placement_state_path, &next_state)
        .map_err(|error| error.to_string())?;

    *state.inner.machine_placement_state.write().map_err(|_| {
        format!(
            "control plane '{}' could not update machine placement state",
            state.inner.control_plane
        )
    })? = next_state;
    *state.inner.machine_placements.write().map_err(|_| {
        format!(
            "control plane '{}' could not update machine placement records",
            state.inner.control_plane
        )
    })? = next_placements;
    Ok(placement)
}

fn resolve_known_node_binding(
    state: &ControlPlaneState,
    node_name: &str,
) -> Result<Option<(HostedNodeBinding, PathBuf)>, String> {
    let now = current_unix_timestamp_seconds().map_err(|error| {
        format!(
            "control plane '{}' could not inspect node registration freshness: {error}",
            state.inner.control_plane
        )
    })?;
    if let Some(record) = state
        .inner
        .registered_nodes
        .read()
        .map_err(|_| {
            format!(
                "control plane '{}' could not inspect registered nodes",
                state.inner.control_plane
            )
        })?
        .get(node_name)
        .cloned()
    {
        if record.contract.freshness.fresh_until < now {
            return Err(format!(
                "node '{}' is registered but stale; last refresh {} with ttl {}s expired at {}",
                node_name,
                record.contract.freshness.refreshed_at,
                record.contract.freshness.ttl_seconds,
                record.contract.freshness.fresh_until
            ));
        }
        return Ok(Some((
            record.binding,
            record.contract.node.runtime_root.clone(),
        )));
    }
    if let Some(binding) = state.inner.static_node_bindings.get(node_name).cloned() {
        let runtime_root = state
            .inner
            .config
            .nodes
            .get(node_name)
            .map(|node| node.runtime_root.clone())
            .unwrap_or_else(|| hosted_placeholder_runtime_root(&state.inner.control_plane));
        return Ok(Some((binding, runtime_root)));
    }
    Ok(None)
}

fn resolve_machine_binding(
    state: &ControlPlaneState,
    summary: &HostedMachineSummaryContract,
) -> Result<
    (
        Option<HostedNodeBinding>,
        HostedRouteContext,
        Option<String>,
    ),
    String,
> {
    if let Some(placement) = refresh_machine_placements(state)?
        .get(&summary.machine_name)
        .cloned()
    {
        let route_context = stored_machine_route_context(summary, &placement);
        return match resolve_known_node_binding(state, &placement.node_name) {
            Ok(Some((binding, _))) => Ok((Some(binding), route_context, None)),
            Ok(None) => Ok((
                None,
                route_context.clone(),
                Some(machine_placement_detail(
                    &route_context,
                    format!(
                        "stored placement points at node '{}' but the control plane has no live registered node-agent endpoint for it.",
                        placement.node_name
                    ),
                )),
            )),
            Err(message) => Ok((
                None,
                route_context.clone(),
                Some(machine_placement_detail(
                    &route_context,
                    format!(
                        "stored placement on node '{}' is not currently usable: {message}",
                        placement.node_name
                    ),
                )),
            )),
        };
    }

    match resolve_node_binding(state, summary) {
        Ok((binding, route)) => Ok((Some(binding), route, None)),
        Err((route, message)) => Ok((None, route, Some(message))),
    }
}

fn refresh_machine_placements(
    state: &ControlPlaneState,
) -> Result<BTreeMap<String, HostedMachinePlacementRecord>, String> {
    let placement_state = load_machine_placement_state(
        &state.inner.machine_placement_state_path,
        &state.inner.control_plane,
    )
    .map_err(|error| {
        format!(
            "control plane '{}' could not refresh machine placement state: {error}",
            state.inner.control_plane
        )
    })?;
    let placements = placement_state.machines.clone();
    *state.inner.machine_placement_state.write().map_err(|_| {
        format!(
            "control plane '{}' could not update machine placement state",
            state.inner.control_plane
        )
    })? = placement_state;
    *state.inner.machine_placements.write().map_err(|_| {
        format!(
            "control plane '{}' could not update machine placement records",
            state.inner.control_plane
        )
    })? = placements.clone();
    Ok(placements)
}

fn stored_machine_route_context(
    summary: &HostedMachineSummaryContract,
    placement: &HostedMachinePlacementRecord,
) -> HostedRouteContext {
    let mut route = HostedRouteContext::from_machine_summary(summary)
        .with_selected_node(placement.node_name.clone(), placement.runtime_root.clone());
    if placement.placement_detail.is_some() {
        route.placement_detail = placement.placement_detail.clone();
    }
    route
}

fn machine_placement_detail(route_context: &HostedRouteContext, message: String) -> String {
    match route_context.placement_detail.as_deref() {
        Some(detail) if !detail.is_empty() => format!("{message} {detail}"),
        _ => message,
    }
}

fn authorize(state: &ControlPlaneState, headers: &HeaderMap) -> Option<Response> {
    match headers
        .get(&state.inner.auth_header)
        .and_then(|value| value.to_str().ok())
    {
        Some(value) if value == state.inner.auth_value => None,
        _ => Some(error_response(
            StatusCode::UNAUTHORIZED,
            format!(
                "control plane '{}' expects a bearer token in the '{}' header",
                state.inner.control_plane, state.inner.auth_header
            ),
            Some(HostedRouteContext {
                control_plane: Some(state.inner.control_plane.clone()),
                ..HostedRouteContext::default()
            }),
        )),
    }
}

fn resolve_summary(
    state: &ControlPlaneState,
    machine: &str,
) -> Result<HostedMachineSummaryContract, Response> {
    let summary = state
        .inner
        .config
        .hosted_machine_summary_contract(machine)
        .map_err(|error| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "control plane '{}' could not resolve hosted machine '{}': {error}",
                    state.inner.control_plane, machine
                ),
                Some(HostedRouteContext {
                    control_plane: Some(state.inner.control_plane.clone()),
                    machine_name: Some(machine.to_string()),
                    ..HostedRouteContext::default()
                }),
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!(
                    "machine '{}' does not resolve to hosted control plane '{}'",
                    machine, state.inner.control_plane
                ),
                Some(HostedRouteContext {
                    control_plane: Some(state.inner.control_plane.clone()),
                    machine_name: Some(machine.to_string()),
                    ..HostedRouteContext::default()
                }),
            )
        })?;

    if summary.control_plane != state.inner.control_plane {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            format!(
                "machine '{}' belongs to control plane '{}', not '{}'",
                machine, summary.control_plane, state.inner.control_plane
            ),
            Some(HostedRouteContext::from_machine_summary(&summary)),
        ));
    }

    Ok(summary)
}

fn resolve_node_binding(
    state: &ControlPlaneState,
    summary: &HostedMachineSummaryContract,
) -> Result<(HostedNodeBinding, HostedRouteContext), (HostedRouteContext, String)> {
    let route_context = HostedRouteContext::from_machine_summary(summary);
    if summary.candidate_nodes.is_empty() {
        return Err((
            route_context,
            format!(
                "control plane '{}' cannot place machine '{}': {}",
                state.inner.control_plane, summary.machine_name, summary.placement_detail
            ),
        ));
    }

    let mut stale_registrations = Vec::new();
    for node_name in &summary.candidate_nodes {
        match resolve_known_node_binding(state, node_name) {
            Ok(Some((binding, runtime_root))) => {
                return Ok((
                    binding,
                    route_context
                        .clone()
                        .with_selected_node(node_name.clone(), runtime_root),
                ));
            }
            Ok(None) => {}
            Err(detail) => stale_registrations.push(detail),
        }
    }

    let mut detail = format!(
        "control plane '{}' could not route machine '{}' because none of the candidate nodes {:?} have a live registered node-agent endpoint.",
        state.inner.control_plane, summary.machine_name, summary.candidate_nodes
    );
    if !stale_registrations.is_empty() {
        detail.push_str(&format!(
            " Stale registrations: {}.",
            stale_registrations.join(", ")
        ));
    }
    detail.push_str(&format!(" {}", summary.placement_detail));
    Err((route_context, detail))
}

fn resolve_service_apply_binding(
    state: &ControlPlaneState,
    summary: &HostedMachineSummaryContract,
    host_group: Option<&str>,
) -> Result<(HostedNodeBinding, HostedRouteContext), (HostedRouteContext, String)> {
    let route_context = HostedRouteContext::from_machine_summary(summary);
    let Some(host_group) = host_group else {
        return Err((
            route_context,
            format!(
                "control plane '{}' requires a host group for hosted service placement on machine '{}'; available groups: {}",
                state.inner.control_plane,
                summary.machine_name,
                if summary.host_groups.is_empty() {
                    String::from("(none)")
                } else {
                    summary.host_groups.join(", ")
                }
            ),
        ));
    };
    if !summary.host_groups.iter().any(|group| group == host_group) {
        return Err((
            route_context,
            format!(
                "control plane '{}' cannot place service for machine '{}' in host group '{}'; available groups: {}. {}",
                state.inner.control_plane,
                summary.machine_name,
                host_group,
                if summary.host_groups.is_empty() {
                    String::from("(none)")
                } else {
                    summary.host_groups.join(", ")
                },
                summary.placement_detail
            ),
        ));
    }
    let inventory = state
        .inner
        .config
        .hosted_inventory_contract()
        .map_err(|error| {
            (
                route_context.clone(),
                format!(
                    "control plane '{}' could not inspect hosted inventory for machine '{}': {error}",
                    state.inner.control_plane, summary.machine_name
                ),
            )
        })?;
    let Some(group) = inventory.host_groups.get(host_group) else {
        return Err((
            route_context,
            format!(
                "control plane '{}' does not declare host group '{}' for machine '{}'",
                state.inner.control_plane, host_group, summary.machine_name
            ),
        ));
    };

    let mut eligible = Vec::new();
    let mut missing_bindings = Vec::new();
    let mut stale_bindings = Vec::new();
    for node_name in &group.nodes {
        if !summary
            .candidate_nodes
            .iter()
            .any(|candidate| candidate == node_name)
        {
            continue;
        }
        match resolve_known_node_binding(state, node_name) {
            Ok(Some((binding, runtime_root))) => {
                eligible.push((node_name.clone(), binding, runtime_root));
            }
            Ok(None) => missing_bindings.push(node_name.clone()),
            Err(detail) => stale_bindings.push(detail),
        }
    }
    eligible.sort_by(|left, right| left.0.cmp(&right.0));
    if let Some((node_name, binding, runtime_root)) = eligible.into_iter().next() {
        return Ok((
            binding,
            route_context
                .clone()
                .with_selected_node(node_name, runtime_root),
        ));
    }

    let rejected = group
        .nodes
        .iter()
        .filter_map(|node_name| {
            summary
                .rejected_nodes
                .get(node_name)
                .map(|reason| format!("{node_name} ({reason})"))
        })
        .collect::<Vec<_>>();
    let mut detail = format!(
        "control plane '{}' cannot place service for machine '{}' in host group '{}' with scheduler '{:?}'",
        state.inner.control_plane, summary.machine_name, host_group, group.scheduler
    );
    if !rejected.is_empty() {
        detail.push_str(&format!("; rejected nodes: {}", rejected.join(", ")));
    }
    if !missing_bindings.is_empty() {
        detail.push_str(&format!(
            "; eligible nodes without live registrations: {}",
            missing_bindings.join(", ")
        ));
    }
    if !stale_bindings.is_empty() {
        detail.push_str(&format!(
            "; stale registrations: {}",
            stale_bindings.join(", ")
        ));
    }
    if rejected.is_empty() && missing_bindings.is_empty() {
        detail.push_str("; no candidate nodes in the requested host group are eligible");
    }
    detail.push_str(&format!("; {}", summary.placement_detail));
    Err((route_context, detail))
}

async fn proxy_raw(
    state: &ControlPlaneState,
    binding: &HostedNodeBinding,
    route: HostedNodeRoute,
    method: Method,
    body: Option<Bytes>,
    route_context: HostedRouteContext,
) -> Response {
    match proxy_bytes(
        state,
        binding,
        route,
        method,
        body,
        Some("application/json"),
        route_context.clone(),
    )
    .await
    {
        Ok((status, bytes)) => bytes_response(status, bytes),
        Err(message) => error_response(StatusCode::BAD_GATEWAY, message, Some(route_context)),
    }
}

async fn proxy_json<T: DeserializeOwned>(
    state: &ControlPlaneState,
    binding: &HostedNodeBinding,
    route: HostedNodeRoute,
    method: Method,
    body: Option<Bytes>,
    route_context: HostedRouteContext,
) -> Result<T, String> {
    let (status, bytes) = proxy_bytes(
        state,
        binding,
        route,
        method,
        body,
        Some("application/json"),
        route_context,
    )
    .await?;
    if !status.is_success() {
        if let Ok(error) = serde_json::from_slice::<HostedError>(&bytes) {
            return Err(error.message);
        }
        return Err(format!(
            "node agent '{}' returned status {}",
            binding.node_name, status
        ));
    }

    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "node agent '{}' returned invalid JSON payload: {error}",
            binding.node_name
        )
    })
}

async fn proxy_bytes(
    state: &ControlPlaneState,
    binding: &HostedNodeBinding,
    route: HostedNodeRoute,
    method: Method,
    body: Option<Bytes>,
    content_type: Option<&str>,
    route_context: HostedRouteContext,
) -> Result<(StatusCode, Bytes), String> {
    let url = format!("{}{}", binding.endpoint.trim_end_matches('/'), route.path());
    let method = reqwest::Method::from_bytes(method.as_str().as_bytes()).map_err(|error| {
        format!(
            "control plane '{}' could not convert request method for node '{}': {error}",
            state.inner.control_plane, binding.node_name
        )
    })?;
    let mut request = state.inner.client.request(method, url);
    for (name, value) in HostedNodeAgentHeaders::new(binding.token.clone()).to_header_map() {
        request = request.header(name, value);
    }
    if let Some(body) = body {
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE.as_str(), content_type);
        }
        request = request.body(body.to_vec());
    }
    let response = request.send().await.map_err(|error| {
        format!(
            "control plane '{}' could not reach node '{}' for machine '{}': {error}",
            state.inner.control_plane,
            binding.node_name,
            route_context.machine_name.as_deref().unwrap_or("<unknown>")
        )
    })?;
    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let bytes = response.bytes().await.map_err(|error| {
        format!(
            "control plane '{}' received an unreadable response from node '{}': {error}",
            state.inner.control_plane, binding.node_name
        )
    })?;
    Ok((status, bytes))
}

fn malformed_machine_status(
    summary: &HostedMachineSummaryContract,
    route_context: HostedRouteContext,
    detail: String,
) -> MachineStatus {
    let runtime_root = route_context
        .runtime_root
        .clone()
        .unwrap_or_else(|| hosted_placeholder_runtime_root(&summary.control_plane));
    let paths = RuntimePaths::for_machine(runtime_root, &summary.machine_name);
    MachineStatus {
        machine_name: summary.machine_name.clone(),
        state: MachineRuntimeState::Malformed,
        pid: None,
        control: summary.control.clone(),
        runtime_dir: paths.runtime_dir,
        config_path: paths.config_path,
        manifest_path: paths.manifest_path,
        pid_path: paths.pid_path,
        firecracker_log: paths.firecracker_log,
        stdout_log: paths.stdout_log,
        stderr_log: paths.stderr_log,
        hosted_fleet_nodes: Vec::new(),
        detail,
    }
}

fn annotate_machine_status_with_fleet_state(
    state: &ControlPlaneState,
    summary: &HostedMachineSummaryContract,
    route_context: &HostedRouteContext,
    mut status: MachineStatus,
) -> Result<MachineStatus, String> {
    status.hosted_fleet_nodes = hosted_fleet_node_statuses(state, summary, route_context)?;
    Ok(status)
}

fn hosted_fleet_node_statuses(
    state: &ControlPlaneState,
    summary: &HostedMachineSummaryContract,
    route_context: &HostedRouteContext,
) -> Result<Vec<HostedFleetNodeStatus>, String> {
    let now = current_unix_timestamp_seconds().map_err(|error| {
        format!(
            "control plane '{}' could not inspect hosted fleet state for machine '{}': {error}",
            state.inner.control_plane, summary.machine_name
        )
    })?;
    let registered_nodes = state.inner.registered_nodes.read().map_err(|_| {
        format!(
            "control plane '{}' could not inspect hosted fleet state for machine '{}': registered-node state lock poisoned",
            state.inner.control_plane, summary.machine_name
        )
    })?;
    let imported_inventory = state.inner.imported_inventory.read().map_err(|_| {
        format!(
            "control plane '{}' could not inspect hosted fleet state for machine '{}': imported-inventory state lock poisoned",
            state.inner.control_plane, summary.machine_name
        )
    })?;

    let mut relevant_nodes = BTreeSet::new();
    relevant_nodes.extend(summary.candidate_nodes.iter().cloned());
    relevant_nodes.extend(summary.rejected_nodes.keys().cloned());
    if let Some(node_name) = route_context.node_name.as_ref() {
        relevant_nodes.insert(node_name.clone());
    }

    let mut statuses = Vec::new();
    for node_name in relevant_nodes {
        let configured = state.inner.config.nodes.get(&node_name).ok_or_else(|| {
            format!(
                "control plane '{}' could not merge hosted fleet state for machine '{}': affected node '{}' is not present in the configured hosted inventory",
                state.inner.control_plane, summary.machine_name, node_name
            )
        })?;
        let configured_provider = state
            .inner
            .config
            .hosts
            .get(&configured.host)
            .map(|host| host.provider)
            .ok_or_else(|| {
                format!(
                    "control plane '{}' could not merge hosted fleet state for machine '{}': affected node '{}' references unknown host '{}'",
                    state.inner.control_plane, summary.machine_name, node_name, configured.host
                )
            })?;
        let imported = imported_inventory.get(&node_name);
        if let Some(imported) = imported {
            if imported.provider != configured_provider {
                return Err(format!(
                    "control plane '{}' could not merge hosted fleet state for machine '{}': affected node '{}' has imported provider '{:?}' but configured provider '{:?}'",
                    state.inner.control_plane,
                    summary.machine_name,
                    node_name,
                    imported.provider,
                    configured_provider
                ));
            }
            if !imported.capability_summary.is_populated() {
                return Err(format!(
                    "control plane '{}' could not merge hosted fleet state for machine '{}': affected node '{}' has an empty imported capability summary",
                    state.inner.control_plane, summary.machine_name, node_name
                ));
            }
            if !imported
                .capability_summary
                .is_subset_of(&configured.capabilities)
            {
                return Err(format!(
                    "control plane '{}' could not merge hosted fleet state for machine '{}': affected node '{}' has imported capabilities outside the configured node contract",
                    state.inner.control_plane, summary.machine_name, node_name
                ));
            }
        }

        let selected = route_context.node_name.as_deref() == Some(node_name.as_str());
        let registration = registered_nodes.get(&node_name);
        let (registered, freshness, refreshed_at_unix_s, ttl_seconds, fresh_until_unix_s) =
            match registration {
                Some(record) if record.contract.freshness.fresh_until >= now => (
                    true,
                    HostedFleetFreshnessState::Live,
                    Some(record.contract.freshness.refreshed_at),
                    Some(record.contract.freshness.ttl_seconds),
                    Some(record.contract.freshness.fresh_until),
                ),
                Some(record) => (
                    true,
                    HostedFleetFreshnessState::Stale,
                    Some(record.contract.freshness.refreshed_at),
                    Some(record.contract.freshness.ttl_seconds),
                    Some(record.contract.freshness.fresh_until),
                ),
                None => (
                    false,
                    HostedFleetFreshnessState::MissingRegistration,
                    None,
                    None,
                    None,
                ),
            };

        let mut detail_parts = Vec::new();
        if selected {
            detail_parts.push(String::from("Selected by the current control-plane route."));
        }
        if let Some(reason) = summary.rejected_nodes.get(&node_name) {
            detail_parts.push(format!("Rejected for routing: {reason}"));
        }
        if let Some(imported) = imported {
            detail_parts.push(format!(
                "Imported from '{}' at {}.",
                imported.provenance, imported.imported_at
            ));
        } else {
            detail_parts.push(String::from("No imported inventory record."));
        }
        match registration {
            Some(record) if record.contract.freshness.fresh_until >= now => {
                detail_parts.push(format!(
                    "Registered with a live node-agent refresh at {} and ttl {}s (fresh until {}).",
                    record.contract.freshness.refreshed_at,
                    record.contract.freshness.ttl_seconds,
                    record.contract.freshness.fresh_until
                ))
            }
            Some(record) => detail_parts.push(format!(
                "Registered node-agent refresh at {} with ttl {}s expired at {}.",
                record.contract.freshness.refreshed_at,
                record.contract.freshness.ttl_seconds,
                record.contract.freshness.fresh_until
            )),
            None => detail_parts.push(String::from("No registered node-agent endpoint.")),
        }

        let routing_eligibility = if summary.rejected_nodes.contains_key(&node_name) {
            HostedFleetRoutingEligibility::Rejected
        } else {
            match freshness {
                HostedFleetFreshnessState::Live => HostedFleetRoutingEligibility::Eligible,
                HostedFleetFreshnessState::Stale => {
                    HostedFleetRoutingEligibility::StaleRegistration
                }
                HostedFleetFreshnessState::MissingRegistration => {
                    HostedFleetRoutingEligibility::MissingRegistration
                }
            }
        };

        statuses.push(HostedFleetNodeStatus {
            node_name,
            configured: true,
            imported: imported.is_some(),
            registered,
            selected,
            freshness,
            routing_eligibility,
            import_provenance: imported.map(|record| record.provenance.clone()),
            imported_at_unix_s: imported.map(|record| record.imported_at),
            refreshed_at_unix_s,
            ttl_seconds,
            fresh_until_unix_s,
            detail: detail_parts.join(" "),
        });
    }

    Ok(statuses)
}

fn malformed_stop_result(
    summary: &HostedMachineSummaryContract,
    route_context: HostedRouteContext,
    detail: String,
) -> StopResult {
    let status = malformed_machine_status(summary, route_context, detail);
    StopResult {
        machine_name: status.machine_name,
        previous_state: status.state,
        current_state: status.state,
        pid: status.pid,
        control: status.control,
        runtime_dir: status.runtime_dir,
        detail: status.detail,
    }
}

fn render_unavailable_machine_monitor(
    summary: &HostedMachineSummaryContract,
    route: HostedRouteContext,
    message: String,
) -> Response {
    let route_context = route.clone();
    let status = malformed_machine_status(summary, route, message);
    match machine_monitor_report(
        status,
        Some(summary.control_plane.clone()),
        route_context.node_name.clone(),
        route_context.host_groups.clone(),
    ) {
        Ok(result) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route_context,
                result,
            },
        ),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' could not synthesize monitor output for machine '{}': {error}",
                summary.control_plane, summary.machine_name
            ),
            Some(route_context),
        ),
    }
}

fn render_unavailable_machine_top(
    summary: &HostedMachineSummaryContract,
    route: HostedRouteContext,
    message: String,
) -> Response {
    let route_context = route.clone();
    let status = malformed_machine_status(summary, route, message);
    match machine_top_report(
        status,
        Some(summary.control_plane.clone()),
        route_context.node_name.clone(),
        route_context.host_groups.clone(),
    ) {
        Ok(result) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route_context,
                result,
            },
        ),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "control plane '{}' could not synthesize top output for machine '{}': {error}",
                summary.control_plane, summary.machine_name
            ),
            Some(route_context),
        ),
    }
}

fn json_response<T: Serialize>(status: StatusCode, value: &T) -> Response {
    match serde_json::to_vec(value) {
        Ok(bytes) => bytes_response(status, bytes),
        Err(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to encode JSON response: {error}"),
            None,
        ),
    }
}

fn bytes_response(status: StatusCode, bytes: impl Into<Bytes>) -> Response {
    raw_response(status, bytes, "application/json")
}

fn raw_response(
    status: StatusCode,
    bytes: impl Into<Bytes>,
    content_type: &'static str,
) -> Response {
    let mut response = Response::new(Body::from(bytes.into()));
    *response.status_mut() = status;
    response.headers_mut().insert(
        CONTENT_TYPE,
        axum::http::HeaderValue::from_static(content_type),
    );
    response
}

fn error_response(
    status: StatusCode,
    message: String,
    route: Option<HostedRouteContext>,
) -> Response {
    json_response(status, &HostedError { route, message })
}

pub fn serve_node_agent(config: PortConfig, request: NodeAgentServeRequest) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build node-agent runtime")?;

    runtime.block_on(async move {
        let bind = request.bind.clone();
        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind node agent on '{bind}'"))?;
        let state = build_node_agent_state(config, request)?;
        let registration_target = build_node_agent_registration_target(&state)?;
        let registered_at = register_node_agent_once(&state, &registration_target, None)
            .await
            .map_err(|error| {
                anyhow!(
                    "{}: {error}",
                    format!(
                        "node agent '{}' registration failed against control plane '{}'",
                        state.inner.node_name, registration_target.control_plane
                    )
                )
            })?;
        let refresh_state = state.clone();
        let refresh_target = registration_target.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(NODE_AGENT_REGISTRATION_REFRESH_INTERVAL).await;
                if let Err(error) = register_node_agent_once(
                    &refresh_state,
                    &refresh_target,
                    Some(registered_at),
                )
                .await
                {
                    eprintln!(
                        "node agent '{}' registration refresh against control plane '{}' failed: {}",
                        refresh_state.inner.node_name, refresh_target.control_plane, error
                    );
                }
            }
        });
        axum::serve(listener, node_agent_router(state))
            .await
            .context("node-agent server exited unexpectedly")
    })
}

fn build_node_agent_state(
    config: PortConfig,
    request: NodeAgentServeRequest,
) -> Result<NodeAgentState> {
    let runtime_root = config
        .nodes
        .get(&request.node_name)
        .with_context(|| {
            format!(
                "unknown hosted node '{}' for node-agent serve",
                request.node_name
            )
        })?
        .runtime_root
        .clone();

    Ok(NodeAgentState {
        inner: Arc::new(NodeAgentStateInner {
            config,
            node_name: request.node_name,
            runtime_root,
            bind: request.bind,
            token: request.token,
        }),
    })
}

fn build_node_agent_registration_target(
    state: &NodeAgentState,
) -> Result<NodeAgentRegistrationTarget> {
    let node = state
        .inner
        .config
        .nodes
        .get(&state.inner.node_name)
        .with_context(|| {
            format!(
                "unknown hosted node '{}' for node-agent registration",
                state.inner.node_name
            )
        })?;
    let host = state.inner.config.hosts.get(&node.host).with_context(|| {
        format!(
            "node '{}' references unknown host '{}'",
            state.inner.node_name, node.host
        )
    })?;
    let control_plane = match &host.connection {
        HostConnection::HostedControlPlane { control_plane } => control_plane.clone(),
        HostConnection::Local => {
            bail!(
                "node '{}' does not target a hosted control plane",
                state.inner.node_name
            )
        }
    };
    let spec = state
        .inner
        .config
        .control_planes
        .get(&control_plane)
        .with_context(|| {
            format!(
                "node '{}' references unknown control plane '{}'",
                state.inner.node_name, control_plane
            )
        })?;
    let token = match &spec.auth.source {
        HostedAuthTokenSource::Env { variable } => std::env::var(variable).with_context(|| {
            format!(
                "control plane '{}' expects token in environment variable '{}'",
                control_plane, variable
            )
        })?,
    };

    Ok(NodeAgentRegistrationTarget {
        control_plane,
        endpoint: spec.endpoint.clone(),
        node_endpoint: normalize_node_agent_endpoint(&state.inner.bind),
        auth_headers: HostedClientHeaders::new(
            spec.auth.header.clone(),
            format!("Bearer {token}"),
            spec.audience.clone(),
        ),
    })
}

fn normalize_node_agent_endpoint(bind: &str) -> String {
    if bind.starts_with("http://") || bind.starts_with("https://") {
        bind.to_string()
    } else {
        format!("http://{bind}")
    }
}

async fn register_node_agent_once(
    state: &NodeAgentState,
    target: &NodeAgentRegistrationTarget,
    registered_at: Option<u64>,
) -> Result<u64> {
    let refreshed_at = current_unix_timestamp_seconds()?;
    let registered_at = registered_at.unwrap_or(refreshed_at);
    let route = HostedControlPlaneRoute::Registration(HostedRegistrationRoute::Refresh {
        node_name: state.inner.node_name.clone(),
    });
    let request_body = HostedNodeRegistrationRequest {
        control_plane: target.control_plane.clone(),
        node_name: state.inner.node_name.clone(),
        registration: HostedNodeRegistration {
            endpoint: target.node_endpoint.clone(),
            token: state.inner.token.clone(),
            registered_at,
            refreshed_at,
            ttl_seconds: NODE_AGENT_REGISTRATION_TTL_SECONDS,
        },
    };
    let mut request = Client::new().post(format!(
        "{}{}",
        target.endpoint.trim_end_matches('/'),
        route.path()
    ));
    for (name, value) in target.auth_headers.to_header_map() {
        request = request.header(name, value);
    }
    let response = request
        .header(CONTENT_TYPE.as_str(), "application/json")
        .body(serde_json::to_vec(&request_body).context("failed to encode registration request")?)
        .send()
        .await
        .with_context(|| {
            format!(
                "could not reach control plane '{}' at '{}'",
                target.control_plane, target.endpoint
            )
        })?;
    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let bytes = response
        .bytes()
        .await
        .context("failed to read control-plane registration response")?;
    if !status.is_success() {
        if let Ok(error) = serde_json::from_slice::<HostedError>(&bytes) {
            bail!("{}", error.message);
        }
        bail!(
            "control plane '{}' rejected node '{}' registration with status {}",
            target.control_plane,
            state.inner.node_name,
            status
        );
    }
    let _: HostedSuccess<HostedRegisteredNodeContract> = serde_json::from_slice(&bytes)
        .context("control plane returned invalid registration JSON")?;
    Ok(registered_at)
}

fn node_agent_router(state: NodeAgentState) -> Router {
    Router::new()
        .route(
            "/v1/node/machines/{machine}",
            get(node_machine_status).post(node_machine_command),
        )
        .route(
            "/v1/node/machines/{machine}/monitor",
            get(node_machine_monitor),
        )
        .route("/v1/node/machines/{machine}/top", get(node_machine_top))
        .route(
            "/v1/node/machines/{machine}/guest:exec",
            post(node_guest_exec),
        )
        .route(
            "/v1/node/machines/{machine}/guest:copy",
            post(node_guest_copy),
        )
        .route(
            "/v1/node/machines/{machine}/guest:copy:stream",
            post(node_guest_copy_stream),
        )
        .route(
            "/v1/node/machines/{machine}/guest:pty",
            post(node_guest_pty),
        )
        .route(
            "/v1/node/machines/{machine}/guest:logs",
            post(node_guest_logs),
        )
        .route(
            "/v1/node/machines/{machine}/guest:forward",
            post(node_guest_forward),
        )
        .route(
            "/v1/node/machines/{machine}/guest:forward:detached",
            get(node_guest_forward_detached_list).post(node_guest_forward_detached_start),
        )
        .route(
            "/v1/node/machines/{machine}/guest:forward:detached/{forward}/stop",
            post(node_guest_forward_detached_stop),
        )
        .route(
            "/v1/node/machines/{machine}/secrets",
            get(node_service_secret_list),
        )
        .route(
            "/v1/node/machines/{machine}/secrets/{secret}",
            put(node_service_secret_put).delete(node_service_secret_remove),
        )
        .route(
            "/v1/node/machines/{machine}/services",
            get(node_service_list).post(node_service_apply),
        )
        .route(
            "/v1/node/machines/{machine}/services/{service}",
            get(node_service_status).post(node_service_command),
        )
        .with_state(state)
}

async fn node_machine_status(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    node_machine_response(
        &state,
        &headers,
        &machine,
        |config, runtime_root, machine_name| {
            runtime_machine_status(config, runtime_root, machine_name)
        },
    )
}

async fn node_machine_monitor(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    node_machine_response(
        &state,
        &headers,
        &machine,
        |config, runtime_root, machine_name| {
            runtime_machine_monitor(config, runtime_root, machine_name)
        },
    )
}

async fn node_machine_top(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    node_machine_response(
        &state,
        &headers,
        &machine,
        |config, runtime_root, machine_name| {
            runtime_machine_top(config, runtime_root, machine_name)
        },
    )
}

async fn node_machine_command(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(machine_name) = machine.strip_suffix(":launch") {
        return node_machine_response(
            &state,
            &headers,
            machine_name,
            |config, runtime_root, machine_name| {
                launch_local_machine(
                    config,
                    &LaunchRequest {
                        machine_name,
                        runtime_root,
                        boot_wait: Duration::from_secs(3),
                    },
                )
            },
        );
    }
    if let Some(machine_name) = machine.strip_suffix(":stop") {
        return node_machine_response(
            &state,
            &headers,
            machine_name,
            |config, runtime_root, machine_name| {
                runtime_stop_machine(config, runtime_root, machine_name, Duration::from_secs(3))
            },
        );
    }

    node_agent_error(
        &state,
        Some(machine),
        format!(
            "node '{}' only serves launch and stop through '/v1/node/machines/{{machine}}:launch' and '/v1/node/machines/{{machine}}:stop'",
            state.inner.node_name
        ),
    )
}

async fn node_guest_exec(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_operation_response(&state, &headers, &machine, body, HostedGuestVerb::Exec)
}

async fn node_guest_copy(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_operation_response(&state, &headers, &machine, body, HostedGuestVerb::Copy)
}

async fn node_guest_copy_stream(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_copy_stream_response(&state, &headers, &machine, body)
}

async fn node_guest_pty(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_operation_response(&state, &headers, &machine, body, HostedGuestVerb::Pty)
}

async fn node_guest_logs(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_operation_response(&state, &headers, &machine, body, HostedGuestVerb::Logs)
}

async fn node_guest_forward(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    node_guest_operation_response(&state, &headers, &machine, body, HostedGuestVerb::Forward)
}

async fn node_guest_forward_detached_start(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }

    let request: HostedDetachedForwardStartRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' received invalid detached forward JSON: {error}",
                    state.inner.node_name
                ),
                Some(node_route_context(&state, Some(machine))),
            );
        }
    };

    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok((localized, route)) => (
            localized,
            route.with_forward_name(
                request
                    .name
                    .clone()
                    .unwrap_or_else(|| String::from("(generated)")),
            ),
        ),
        Err(response) => return response,
    };

    match start_detached_forward(
        &localized,
        DetachedForwardLaunchRequest {
            machine_name: &machine,
            runtime_root: &state.inner.runtime_root,
            listen: &request.listen,
            target: &request.target,
            name: request.name.as_deref(),
        },
    ) {
        Ok(result) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route.with_forward_name(result.name.clone()),
                result,
            },
        ),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to start detached forward '{}' for machine '{}': {error}",
                state.inner.node_name,
                route.forward_name.as_deref().unwrap_or("(generated)"),
                machine
            ),
            Some(route),
        ),
    }
}

async fn node_guest_forward_detached_list(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }

    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok(value) => value,
        Err(response) => return response,
    };

    match list_detached_forwards(&localized, &machine, &state.inner.runtime_root) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to list detached forwards for machine '{}': {error}",
                state.inner.node_name, machine
            ),
            Some(route),
        ),
    }
}

async fn node_guest_forward_detached_stop(
    State(state): State<NodeAgentState>,
    Path((machine, forward)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }

    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok((localized, route)) => (localized, route.with_forward_name(forward.clone())),
        Err(response) => return response,
    };

    match stop_detached_forward(&localized, &machine, &state.inner.runtime_root, &forward) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to stop detached forward '{}' for machine '{}': {error}",
                state.inner.node_name, forward, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_secret_put(
    State(state): State<NodeAgentState>,
    Path((machine, secret)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let request: SecretPutRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' received invalid service secret JSON: {error}",
                    state.inner.node_name
                ),
                Some(node_route_context(&state, Some(machine))),
            );
        }
    };
    if request.name != secret {
        return error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "node '{}' received mismatched secret name '{}' for path '{}'",
                state.inner.node_name, request.name, secret
            ),
            Some(node_route_context(&state, Some(machine))),
        );
    }

    let (_localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match put_machine_secret_local(
        &state.inner.config,
        crate::SecretPutRequest {
            machine_name: &machine,
            runtime_root: &state.inner.runtime_root,
            name: &request.name,
            value: &request.value,
        },
    ) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to store secret '{}' for machine '{}': {error}",
                state.inner.node_name, request.name, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_secret_list(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let (_localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match list_machine_secrets_local(&state.inner.config, &state.inner.runtime_root, &machine) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to list secrets for machine '{}': {error}",
                state.inner.node_name, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_secret_remove(
    State(state): State<NodeAgentState>,
    Path((machine, secret)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let (_localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match delete_machine_secret_local(
        &state.inner.config,
        &state.inner.runtime_root,
        &machine,
        &secret,
    ) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to remove secret '{}' for machine '{}': {error}",
                state.inner.node_name, secret, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_apply(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let request: ServiceApplyRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' received invalid service JSON: {error}",
                    state.inner.node_name
                ),
                Some(node_route_context(&state, Some(machine))),
            );
        }
    };
    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok((localized, route)) => (localized, route.with_service_name(request.name.clone())),
        Err(response) => return response,
    };
    let host_group = request.host_group.clone();
    let runtime_request = RuntimeServiceApplyRequest {
        machine_name: &machine,
        runtime_root: &state.inner.runtime_root,
        name: &request.name,
        kind: match request.kind {
            port_sdk::ServiceKind::Service => crate::ServiceKind::Service,
            port_sdk::ServiceKind::Sandbox => crate::ServiceKind::Sandbox,
        },
        host_group: host_group.as_deref(),
        command: request.command,
        secret_bindings: request
            .secret_bindings
            .into_iter()
            .map(|binding| ServiceSecretBinding {
                env: binding.env,
                secret: binding.secret,
            })
            .collect(),
    };
    match apply_hosted_machine_service_live(&state.inner.config, &localized, runtime_request) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to apply service for machine '{}': {error}",
                state.inner.node_name, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_list(
    State(state): State<NodeAgentState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match refresh_hosted_machine_service_list(
        &state.inner.config,
        &localized,
        &state.inner.runtime_root,
        &machine,
    ) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to list services for machine '{}': {error}",
                state.inner.node_name, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_status(
    State(state): State<NodeAgentState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = node_authorize(&state, &headers) {
        return response;
    }
    let (localized, route) = match localize_machine_for_node(&state, &machine) {
        Ok((localized, route)) => (localized, route.with_service_name(service.clone())),
        Err(response) => return response,
    };
    match refresh_hosted_machine_service_runtime(
        &state.inner.config,
        &localized,
        &state.inner.runtime_root,
        &machine,
        &service,
    ) {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to load service '{}' for machine '{}': {error}",
                state.inner.node_name, service, machine
            ),
            Some(route),
        ),
    }
}

async fn node_service_command(
    State(state): State<NodeAgentState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(service_name) = service.strip_suffix(":stop") {
        if let Some(response) = node_authorize(&state, &headers) {
            return response;
        }
        let (localized, route) = match localize_machine_for_node(&state, &machine) {
            Ok((localized, route)) => {
                (localized, route.with_service_name(service_name.to_string()))
            }
            Err(response) => return response,
        };
        return match stop_hosted_machine_service_live(
            &state.inner.config,
            &localized,
            &state.inner.runtime_root,
            &machine,
            service_name,
        ) {
            Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
            Err(error) => error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "node '{}' failed to stop service '{}' for machine '{}': {error}",
                    state.inner.node_name, service_name, machine
                ),
                Some(route),
            ),
        };
    }

    node_agent_error(
        &state,
        Some(machine),
        format!(
            "node '{}' only serves service stop through '/v1/node/machines/{{machine}}/services/{{service}}:stop'",
            state.inner.node_name
        ),
    )
}

fn node_authorize(state: &NodeAgentState, headers: &HeaderMap) -> Option<Response> {
    match headers
        .get("x-port-node-agent-token")
        .and_then(|value| value.to_str().ok())
    {
        Some(value) if value == state.inner.token => None,
        _ => Some(node_agent_error(
            state,
            None,
            format!(
                "node '{}' expects an auth token in the '{}' header",
                state.inner.node_name, "x-port-node-agent-token"
            ),
        )),
    }
}

fn node_route_context(state: &NodeAgentState, machine_name: Option<String>) -> HostedRouteContext {
    HostedRouteContext {
        control_plane: None,
        machine_name,
        node_name: Some(state.inner.node_name.clone()),
        runtime_root: Some(state.inner.runtime_root.clone()),
        ..HostedRouteContext::default()
    }
}

fn node_agent_error(
    state: &NodeAgentState,
    machine_name: Option<String>,
    message: String,
) -> Response {
    error_response(
        StatusCode::BAD_GATEWAY,
        message,
        Some(node_route_context(state, machine_name)),
    )
}

fn localize_machine_for_node(
    state: &NodeAgentState,
    machine_name: &str,
) -> Result<(PortConfig, HostedRouteContext), Response> {
    let summary = state
        .inner
        .config
        .hosted_machine_summary_contract(machine_name)
        .map_err(|error| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' could not resolve hosted machine '{}': {error}",
                    state.inner.node_name, machine_name
                ),
                Some(node_route_context(state, Some(machine_name.to_string()))),
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!(
                    "machine '{}' does not resolve to a hosted machine owned by node '{}'",
                    machine_name, state.inner.node_name
                ),
                Some(node_route_context(state, Some(machine_name.to_string()))),
            )
        })?;

    if !summary
        .candidate_nodes
        .iter()
        .any(|node| node == &state.inner.node_name)
    {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            format!(
                "machine '{}' is not routed to node '{}' (candidate nodes: {:?})",
                machine_name, state.inner.node_name, summary.candidate_nodes
            ),
            Some(
                HostedRouteContext::from_machine_summary(&summary)
                    .with_selected_node(&state.inner.node_name, state.inner.runtime_root.clone()),
            ),
        ));
    }

    let mut localized = state.inner.config.clone();
    let host_name = localized
        .machines
        .get(machine_name)
        .map(|machine| machine.host.clone())
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("unknown machine '{}'", machine_name),
                Some(node_route_context(state, Some(machine_name.to_string()))),
            )
        })?;
    let machine = localized
        .machines
        .get(machine_name)
        .expect("machine should exist after summary resolution")
        .clone();
    let node = localized
        .nodes
        .get(&state.inner.node_name)
        .expect("selected node should exist after summary resolution")
        .clone();
    let host = localized
        .hosts
        .get_mut(&host_name)
        .expect("machine host should exist after summary resolution");
    host.connection = HostConnection::Local;
    host.firecracker.local_launch = true;
    if machine.substrate == ExecutionSubstrate::Firecracker
        && machine.protection_mode == ProtectionMode::Pvm
    {
        let Some(node_lane) = node.capabilities.pvm_lane_for(machine.architecture) else {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' does not advertise a PVM lane for machine '{}'",
                    state.inner.node_name, machine_name
                ),
                Some(
                    HostedRouteContext::from_machine_summary(&summary).with_selected_node(
                        &state.inner.node_name,
                        state.inner.runtime_root.clone(),
                    ),
                ),
            ));
        };
        let Some(host_kit) = node_lane.host_kit.clone() else {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' does not declare a prepared PVM host-kit contract for machine '{}'",
                    state.inner.node_name, machine_name
                ),
                Some(
                    HostedRouteContext::from_machine_summary(&summary).with_selected_node(
                        &state.inner.node_name,
                        state.inner.runtime_root.clone(),
                    ),
                ),
            ));
        };
        let mut contract = FirecrackerPvmLaneContract::for_architecture(node_lane.architecture);
        contract.host_kit = Some(host_kit);
        host.firecracker
            .pvm_lanes
            .retain(|lane| lane.architecture != contract.architecture);
        host.firecracker.pvm_lanes.push(contract);
    }
    localized.machines.retain(|name, _| name == machine_name);
    localized.hosts.retain(|name, _| name == &host_name);
    localized.nodes.clear();
    localized.host_groups.clear();
    localized.control_planes.clear();

    let route = HostedRouteContext::from_machine_summary(&summary)
        .with_selected_node(&state.inner.node_name, state.inner.runtime_root.clone());
    Ok((localized, route))
}

fn node_machine_response<T, F>(
    state: &NodeAgentState,
    headers: &HeaderMap,
    machine_name: &str,
    operation: F,
) -> Response
where
    T: Serialize + HostedMachineProjection,
    F: FnOnce(&PortConfig, &std::path::Path, &str) -> Result<T>,
{
    if let Some(response) = node_authorize(state, headers) {
        return response;
    }

    let (localized, route) = match localize_machine_for_node(state, machine_name) {
        Ok(value) => value,
        Err(response) => return response,
    };

    match operation(&localized, &state.inner.runtime_root, machine_name) {
        Ok(result) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route.clone(),
                result: result.apply_hosted_route(&route),
            },
        ),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to serve machine '{}': {error}",
                state.inner.node_name, machine_name
            ),
            Some(route),
        ),
    }
}

fn node_guest_operation_response(
    state: &NodeAgentState,
    headers: &HeaderMap,
    machine_name: &str,
    body: Bytes,
    expected_verb: HostedGuestVerb,
) -> Response {
    if let Some(response) = node_authorize(state, headers) {
        return response;
    }

    let operation: GuestOperation = match serde_json::from_slice(&body) {
        Ok(operation) => operation,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' received invalid guest JSON: {error}",
                    state.inner.node_name
                ),
                Some(node_route_context(state, Some(machine_name.to_string()))),
            );
        }
    };

    let (localized, route) = match localize_machine_for_node(state, machine_name) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let result = match (expected_verb, operation) {
        (HostedGuestVerb::Exec, operation @ GuestOperation::Exec(_))
        | (HostedGuestVerb::Pty, operation @ GuestOperation::Pty(_))
        | (HostedGuestVerb::Logs, operation @ GuestOperation::Logs(_)) => execute_guest_operation(
            &localized,
            GuestRequest {
                machine_name,
                runtime_root: &state.inner.runtime_root,
                operation,
            },
        ),
        (HostedGuestVerb::Copy, GuestOperation::Copy(request)) => copy_guest_file(
            &localized,
            GuestCopyRequest {
                machine_name,
                runtime_root: &state.inner.runtime_root,
                source: std::path::Path::new(&request.source),
                destination: std::path::Path::new(&request.destination),
                direction: request.direction,
            },
        )
        .map(OperationResult::Copy),
        (HostedGuestVerb::Forward, GuestOperation::Forward(request)) => {
            match prepare_guest_forward(
                &localized,
                GuestForwardRequest {
                    machine_name,
                    runtime_root: &state.inner.runtime_root,
                    listen: &request.listen,
                    target: &request.target,
                },
            ) {
                Ok(session) => {
                    let listen = session.listen_addr();
                    let target = session.target().to_string();
                    thread::spawn(move || {
                        let _ = session.serve();
                    });
                    Ok(OperationResult::Forward(ForwardResult { listen, target }))
                }
                Err(error) => Err(error),
            }
        }
        (verb, _) => Err(anyhow::anyhow!(
            "node '{}' received a guest payload that does not match the '{}' route",
            state.inner.node_name,
            verb.as_str()
        )),
    };

    match result {
        Ok(result) => json_response(StatusCode::OK, &HostedSuccess { route, result }),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to serve guest operation for machine '{}': {error}",
                state.inner.node_name, machine_name
            ),
            Some(route),
        ),
    }
}

fn node_guest_copy_stream_response(
    state: &NodeAgentState,
    headers: &HeaderMap,
    machine_name: &str,
    body: Bytes,
) -> Response {
    if let Some(response) = node_authorize(state, headers) {
        return response;
    }

    let (localized, route) = match localize_machine_for_node(state, machine_name) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let mut request_reader = BufReader::new(Cursor::new(body.to_vec()));
    let request: RequestEnvelope = match read_frame(&mut request_reader) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "node '{}' received an invalid guest copy stream payload: {error}",
                    state.inner.node_name
                ),
                Some(route),
            );
        }
    };
    let GuestOperation::Copy(copy_request) = request.operation else {
        return error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "node '{}' received a streamed guest payload that does not match the 'copy' route",
                state.inner.node_name
            ),
            Some(route),
        );
    };

    match relay_guest_copy_stream(
        &localized,
        machine_name,
        &state.inner.runtime_root,
        copy_request,
        &mut request_reader,
    ) {
        Ok(bytes) => raw_response(StatusCode::OK, bytes, "application/octet-stream"),
        Err(error) => error_response(
            StatusCode::BAD_GATEWAY,
            format!(
                "node '{}' failed to serve guest copy stream for machine '{}': {error}",
                state.inner.node_name, machine_name
            ),
            Some(route),
        ),
    }
}

fn relay_guest_copy_stream(
    config: &PortConfig,
    machine_name: &str,
    runtime_root: &std::path::Path,
    copy_request: CopyRequest,
    request_reader: &mut dyn std::io::Read,
) -> Result<Vec<u8>> {
    let mut response_bytes = Vec::new();
    match copy_request.direction {
        port_agent_protocol::CopyDirection::HostToGuest => {
            let result = copy_guest_via_endpoint(
                config,
                machine_name,
                runtime_root,
                copy_request.clone(),
                Some(request_reader),
                None,
            )?;
            write_frame(
                &mut response_bytes,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: None,
                },
            )
            .map_err(|error| anyhow::anyhow!("protocol error: {error}"))?;
            write_frame(
                &mut response_bytes,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Copy(result),
                },
            )
            .map_err(|error| anyhow::anyhow!("protocol error: {error}"))?;
        }
        port_agent_protocol::CopyDirection::GuestToHost => {
            let mut downloaded = Vec::new();
            let result = copy_guest_via_endpoint(
                config,
                machine_name,
                runtime_root,
                copy_request,
                None,
                Some(&mut downloaded),
            )?;
            write_frame(
                &mut response_bytes,
                &ResponseEnvelope::Accepted {
                    id: 1,
                    stream: StreamKind::Bytes,
                    size_bytes: Some(result.bytes_copied),
                },
            )
            .map_err(|error| anyhow::anyhow!("protocol error: {error}"))?;
            response_bytes.extend_from_slice(&downloaded);
            write_frame(
                &mut response_bytes,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Copy(result),
                },
            )
            .map_err(|error| anyhow::anyhow!("protocol error: {error}"))?;
        }
    }

    Ok(response_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router};
    use port_agent_protocol::{
        CopyDirection, CopyRequest, ExecRequest, ExecResult, GuestOperation, OperationResult,
        RequestEnvelope, ResponseEnvelope, StreamKind, read_frame, write_frame,
    };
    use port_hosted_protocol::{
        HostedArtifactTransferRequest, HostedArtifactTransferResult, HostedClientHeaders,
        HostedError, HostedNodeRegistrationRequest, HostedSuccess, PORT_ARTIFACT_TRANSFER_HEADER,
    };
    use port_model::hosted_artifact_store_path;
    use std::io::{BufReader, Cursor, Read, Write};
    use std::net::SocketAddr;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[derive(Clone)]
    struct MockNodeState {
        node_name: String,
        runtime_root: PathBuf,
        pid: u32,
        headers: Arc<Mutex<Vec<String>>>,
        bodies: Arc<Mutex<Vec<String>>>,
    }

    fn sample_artifact_transfer_request(
        config: &PortConfig,
        control_plane: &str,
    ) -> HostedArtifactTransferRequest {
        let artifact = config
            .artifacts
            .kernels
            .get("demo-kernel")
            .expect("demo-kernel should exist");
        let variant = artifact
            .variants
            .iter()
            .find(|variant| {
                variant.selector.architecture == port_model::MachineArchitecture::X86_64
                    && variant.selector.substrate == port_model::ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == port_model::ProtectionMode::Standard
            })
            .expect("demo-kernel standard variant should exist");
        let filename = variant
            .path
            .file_name()
            .expect("variant filename should exist")
            .to_string_lossy()
            .to_string();
        HostedArtifactTransferRequest {
            artifact_name: String::from("demo-kernel"),
            reference: artifact.reference.clone(),
            selector: variant.selector,
            filename: filename.clone(),
            store_path: hosted_artifact_store_path(
                control_plane,
                &artifact.reference,
                variant.selector,
                &filename,
            ),
        }
    }

    #[test]
    fn registered_node_state_path_is_scoped_under_control_plane_runtime_root() {
        assert_eq!(
            registered_node_state_path("demo"),
            PathBuf::from(".port/hosted/demo/registered-nodes.json")
        );
    }

    #[test]
    fn machine_placement_state_path_is_scoped_under_control_plane_runtime_root() {
        assert_eq!(
            machine_placement_state_path("demo"),
            PathBuf::from(".port/hosted/demo/machine-placements.json")
        );
    }

    #[tokio::test]
    async fn hosted_artifact_routes_persist_and_stream_selected_variant() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("artifact-routes");
        let token_var = unique_test_env("PORT_TEST_ARTIFACT_ROUTE_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let request = sample_artifact_transfer_request(&config, &control_plane);
        let identity = config
            .hosted_api_identity_contract("cloud-aws")
            .expect("hosted identity should resolve")
            .expect("cloud-aws should use hosted control plane");
        let headers = HostedClientHeaders::from_identity(&identity, "demo-token").to_header_map();
        let control_plane_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;

        let upload_body = vec![b'k'; 3 * 1024 * 1024 + 17];
        let mut upload =
            Client::new().post(format!("http://{control_plane_addr}/v1/artifacts:push"));
        for (name, value) in &headers {
            upload = upload.header(name, value);
        }
        let upload = upload
            .header(
                PORT_ARTIFACT_TRANSFER_HEADER,
                serde_json::to_string(&request).expect("artifact metadata should encode"),
            )
            .body(upload_body.clone())
            .send()
            .await
            .expect("artifact upload should complete");
        assert_eq!(upload.status(), StatusCode::OK);
        let uploaded: HostedSuccess<HostedArtifactTransferResult> =
            upload.json().await.expect("upload response should decode");
        assert_eq!(uploaded.result.store_path, request.store_path);
        assert_eq!(uploaded.result.bytes_copied, upload_body.len() as u64);
        assert_eq!(
            std::fs::read(&request.store_path).expect("uploaded artifact should persist"),
            upload_body
        );

        let mut download =
            Client::new().post(format!("http://{control_plane_addr}/v1/artifacts:pull"));
        for (name, value) in &headers {
            download = download.header(name, value);
        }
        let download = download
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request).expect("artifact metadata should encode"))
            .send()
            .await
            .expect("artifact download should complete");
        assert_eq!(download.status(), StatusCode::OK);
        assert_eq!(
            download
                .bytes()
                .await
                .expect("downloaded artifact should decode")
                .as_ref(),
            std::fs::read(&request.store_path)
                .expect("stored artifact should exist")
                .as_slice()
        );

        cleanup_registered_state(&control_plane);
    }

    #[tokio::test]
    async fn hosted_artifact_pull_failure_includes_backend_and_store_context() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("artifact-miss");
        let token_var = unique_test_env("PORT_TEST_ARTIFACT_MISS_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let request = sample_artifact_transfer_request(&config, &control_plane);
        let identity = config
            .hosted_api_identity_contract("cloud-aws")
            .expect("hosted identity should resolve")
            .expect("cloud-aws should use hosted control plane");
        let headers = HostedClientHeaders::from_identity(&identity, "demo-token").to_header_map();
        let control_plane_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;

        let mut download =
            Client::new().post(format!("http://{control_plane_addr}/v1/artifacts:pull"));
        for (name, value) in &headers {
            download = download.header(name, value);
        }
        let download = download
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request).expect("artifact metadata should encode"))
            .send()
            .await
            .expect("artifact download should complete");
        assert_eq!(download.status(), StatusCode::NOT_FOUND);
        let error: HostedError = download.json().await.expect("error response should decode");
        assert!(error.message.contains("hosted-api artifact 'demo-kernel'"));
        assert!(error.message.contains("demo-fs/port/demo-kernel:v1"));
        assert!(error.message.contains("x86_64/firecracker/standard"));
        assert!(
            error
                .message
                .contains(&request.store_path.display().to_string())
        );

        cleanup_registered_state(&control_plane);
    }

    #[test]
    fn registered_node_state_validates_into_registered_contracts() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        let state = RegisteredNodeStateFile {
            control_plane: String::from("demo"),
            nodes: BTreeMap::from([(
                String::from("aws-linux-node"),
                HostedNodeRegistration {
                    endpoint: String::from("http://127.0.0.1:9001"),
                    token: String::from("node-secret"),
                    registered_at: 10,
                    refreshed_at: 25,
                    ttl_seconds: 30,
                },
            )]),
        };

        let contracts =
            validate_registered_node_state(&config, &state).expect("state should validate");
        let contract = &contracts["aws-linux-node"];
        assert_eq!(contract.node_name, "aws-linux-node");
        assert_eq!(contract.endpoint, "http://127.0.0.1:9001");
        assert_eq!(contract.freshness.fresh_until, 55);
        assert!(contract.host_groups.contains(&String::from("aws-builders")));
        assert!(contract.host_groups.contains(&String::from("remote-linux")));
    }

    #[test]
    fn registered_node_state_rejects_invalid_registration_detail() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        let state = RegisteredNodeStateFile {
            control_plane: String::from("demo"),
            nodes: BTreeMap::from([(
                String::from("aws-linux-node"),
                HostedNodeRegistration {
                    endpoint: String::from("http://127.0.0.1:9001"),
                    token: String::from(" "),
                    registered_at: 10,
                    refreshed_at: 25,
                    ttl_seconds: 30,
                },
            )]),
        };

        let error =
            validate_registered_node_state(&config, &state).expect_err("blank token should fail");
        assert!(
            error.to_string().contains("must declare a non-empty token"),
            "{}",
            error
        );
    }

    #[test]
    fn hosted_imported_inventory_persists_and_loads_imported_node_records() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("imported-inventory");
        let token_var = unique_test_env("PORT_TEST_IMPORTED_INVENTORY_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let imported_state = ImportedInventoryStateFile {
            control_plane: control_plane.clone(),
            nodes: BTreeMap::from([(
                String::from("aws-linux-node"),
                HostedImportedNodeRecord {
                    provider: HostProvider::Aws,
                    provenance: String::from("inventory-sync"),
                    imported_at: 123,
                    capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                },
            )]),
        };
        let imported_path = imported_inventory_state_path(&control_plane);
        persist_imported_inventory_state(&imported_path, &imported_state)
            .expect("imported inventory should persist");

        let state = build_state(
            config.clone(),
            ControlPlaneServeRequest {
                control_plane: control_plane.clone(),
                bind: reserve_test_addr(),
                node_bindings: Vec::new(),
            },
        )
        .expect("control-plane state should load imported inventory");

        assert_eq!(state.inner.imported_inventory_path, imported_path);
        assert_eq!(
            *state
                .inner
                .imported_inventory_state
                .read()
                .expect("imported inventory state lock"),
            imported_state
        );
        let imported_contract = state
            .inner
            .imported_inventory
            .read()
            .expect("imported inventory lock")
            .get("aws-linux-node")
            .expect("imported node contract should exist")
            .clone();
        assert_eq!(imported_contract.node_name, "aws-linux-node");
        assert_eq!(imported_contract.provider, HostProvider::Aws);
        assert_eq!(imported_contract.provenance, "inventory-sync");
        assert_eq!(imported_contract.imported_at, 123);
        assert_eq!(
            imported_contract.capability_summary,
            config.nodes["aws-linux-node"].capabilities
        );

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[test]
    fn hosted_imported_inventory_rejects_unknown_runtime_only_nodes() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("unknown-import");
        let token_var = unique_test_env("PORT_TEST_UNKNOWN_IMPORT_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let imported_path = imported_inventory_state_path(&control_plane);
        persist_imported_inventory_state(
            &imported_path,
            &ImportedInventoryStateFile {
                control_plane: control_plane.clone(),
                nodes: BTreeMap::from([(
                    String::from("runtime-only-node"),
                    HostedImportedNodeRecord {
                        provider: HostProvider::Aws,
                        provenance: String::from("inventory-sync"),
                        imported_at: 456,
                        capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                    },
                )]),
            },
        )
        .expect("imported inventory should persist");

        let error = build_state(
            config,
            ControlPlaneServeRequest {
                control_plane: control_plane.clone(),
                bind: reserve_test_addr(),
                node_bindings: Vec::new(),
            },
        )
        .err()
        .expect("unknown imported node should fail");
        let message = format!("{error:#}");
        assert!(message.contains("runtime-only-node"), "{message}");
        assert!(
            message.contains(imported_path.to_string_lossy().as_ref()),
            "{message}"
        );
        assert!(message.contains(&control_plane), "{message}");

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[test]
    fn hosted_imported_inventory_surfaces_import_path_and_node_on_conflict() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("conflicting-import");
        let token_var = unique_test_env("PORT_TEST_CONFLICTING_IMPORT_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let imported_path = imported_inventory_state_path(&control_plane);
        persist_imported_inventory_state(
            &imported_path,
            &ImportedInventoryStateFile {
                control_plane: control_plane.clone(),
                nodes: BTreeMap::from([(
                    String::from("aws-linux-node"),
                    HostedImportedNodeRecord {
                        provider: HostProvider::Gcp,
                        provenance: String::from("inventory-sync"),
                        imported_at: 789,
                        capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                    },
                )]),
            },
        )
        .expect("conflicting imported inventory should persist");

        let error = build_state(
            config,
            ControlPlaneServeRequest {
                control_plane: control_plane.clone(),
                bind: reserve_test_addr(),
                node_bindings: Vec::new(),
            },
        )
        .err()
        .expect("conflicting imported node should fail");
        let message = format!("{error:#}");
        assert!(message.contains("aws-linux-node"), "{message}");
        assert!(
            message.contains(imported_path.to_string_lossy().as_ref()),
            "{message}"
        );
        assert!(message.contains("provider"), "{message}");
        assert!(message.contains("gcp"), "{message}");

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[tokio::test]
    async fn hosted_registry_persistence_control_plane_registration_route_persists_and_refreshes_state()
     {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("registration-route");
        let token_var = unique_test_env("PORT_TEST_CP_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let control_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;
        let client = Client::new();
        let path = registered_node_state_path(&control_plane);
        let url = format!("http://{control_addr}/v1/nodes/aws-linux-node/registration");

        let initial = HostedNodeRegistrationRequest {
            control_plane: control_plane.clone(),
            node_name: String::from("aws-linux-node"),
            registration: HostedNodeRegistration {
                endpoint: String::from("http://127.0.0.1:9234"),
                token: String::from("node-secret"),
                registered_at: 10,
                refreshed_at: 10,
                ttl_seconds: 30,
            },
        };
        let response = client
            .post(&url)
            .header("authorization", "Bearer demo-token")
            .header("x-port-audience", "port-hosted-demo")
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&initial).expect("registration request should encode"))
            .send()
            .await
            .expect("registration request should complete");
        let status = response.status();
        let body = response.text().await.expect("response body should decode");
        assert_eq!(status, StatusCode::OK, "{body}");
        let success: HostedSuccess<HostedRegisteredNodeContract> =
            serde_json::from_str(&body).expect("registration success should decode");
        assert_eq!(success.result.endpoint, "http://127.0.0.1:9234");
        assert!(path.exists(), "registered node state should persist");

        let state_after_initial: RegisteredNodeStateFile = serde_json::from_slice(
            &std::fs::read(&path).expect("registered node state should read"),
        )
        .expect("registered node state should decode");
        assert_eq!(state_after_initial.nodes["aws-linux-node"].refreshed_at, 10);

        let refresh = HostedNodeRegistrationRequest {
            control_plane: control_plane.clone(),
            node_name: String::from("aws-linux-node"),
            registration: HostedNodeRegistration {
                endpoint: String::from("http://127.0.0.1:9234"),
                token: String::from("node-secret"),
                registered_at: 10,
                refreshed_at: 25,
                ttl_seconds: 30,
            },
        };
        let response = client
            .post(&url)
            .header("authorization", "Bearer demo-token")
            .header("x-port-audience", "port-hosted-demo")
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&refresh).expect("refresh request should encode"))
            .send()
            .await
            .expect("refresh request should complete");
        let status = response.status();
        let body = response.text().await.expect("refresh body should decode");
        assert_eq!(status, StatusCode::OK, "{body}");

        let state_after_refresh: RegisteredNodeStateFile = serde_json::from_slice(
            &std::fs::read(&path).expect("registered node state should read"),
        )
        .expect("registered node state should decode");
        assert_eq!(state_after_refresh.nodes["aws-linux-node"].refreshed_at, 25);

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[test]
    fn hosted_registry_persistence_node_agent_registers_and_refreshes_against_control_plane() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("node-refresh");
        let token_var = unique_test_env("PORT_TEST_NODE_REFRESH_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        let control_bind = reserve_test_addr();
        config
            .control_planes
            .get_mut(&control_plane)
            .expect("control plane should exist")
            .endpoint = format!("http://{control_bind}");
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let control_config = config.clone();
        let control_plane_name = control_plane.clone();
        let control_bind_for_thread = control_bind.clone();
        std::thread::spawn(move || {
            let _ = serve_control_plane(
                control_config,
                ControlPlaneServeRequest {
                    control_plane: control_plane_name,
                    bind: control_bind_for_thread,
                    node_bindings: Vec::new(),
                },
            );
        });

        wait_for(Duration::from_secs(5), Duration::from_millis(50), || {
            let response = reqwest::blocking::Client::new()
                .get(format!("http://{control_bind}/v1/machines"))
                .header("authorization", "Bearer demo-token")
                .send();
            matches!(response, Ok(response) if response.status() == StatusCode::OK)
        });

        let node_bind = reserve_test_addr();
        let node_config = config.clone();
        std::thread::spawn(move || {
            let _ = serve_node_agent(
                node_config,
                NodeAgentServeRequest {
                    node_name: String::from("aws-linux-node"),
                    bind: node_bind,
                    token: String::from("node-secret"),
                },
            );
        });

        let state_path = registered_node_state_path(&control_plane);
        wait_for(Duration::from_secs(5), Duration::from_millis(100), || {
            std::fs::read(&state_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<RegisteredNodeStateFile>(&bytes).ok())
                .and_then(|state| state.nodes.get("aws-linux-node").cloned())
                .is_some()
        });
        let first: RegisteredNodeStateFile = serde_json::from_slice(
            &std::fs::read(&state_path).expect("registered state should read"),
        )
        .expect("registered state should decode");
        let first_seen = first.nodes["aws-linux-node"].refreshed_at;

        wait_for(Duration::from_secs(4), Duration::from_millis(250), || {
            std::fs::read(&state_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<RegisteredNodeStateFile>(&bytes).ok())
                .and_then(|state| {
                    state
                        .nodes
                        .get("aws-linux-node")
                        .map(|registration| registration.refreshed_at > first_seen)
                })
                .unwrap_or(false)
        });

        let refreshed: RegisteredNodeStateFile = serde_json::from_slice(
            &std::fs::read(&state_path).expect("registered state should read"),
        )
        .expect("registered state should decode");
        assert!(
            refreshed.nodes["aws-linux-node"].refreshed_at > first_seen,
            "expected a later refresh than {first_seen}, got {}",
            refreshed.nodes["aws-linux-node"].refreshed_at
        );

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[tokio::test]
    async fn hosted_registry_persistence_reconstructs_routes_from_durable_state_after_restart() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("restart-recovery");
        let token_var = unique_test_env("PORT_TEST_RESTART_RECOVERY_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let node_addr =
            serve_test_node_agent(config.clone(), "aws-linux-node", "node-secret").await;
        let now = current_unix_timestamp_seconds().expect("unix timestamp should resolve");
        persist_registered_node_state(
            &registered_node_state_path(&control_plane),
            &RegisteredNodeStateFile {
                control_plane: control_plane.clone(),
                nodes: BTreeMap::from([(
                    String::from("aws-linux-node"),
                    HostedNodeRegistration {
                        endpoint: format!("http://{node_addr}"),
                        token: String::from("node-secret"),
                        registered_at: now.saturating_sub(5),
                        refreshed_at: now,
                        ttl_seconds: 60,
                    },
                )]),
            },
        )
        .expect("registered state should persist");

        let control_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;
        let response = Client::new()
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws/guest:exec"
            ))
            .header("authorization", "Bearer demo-token")
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_vec(&GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/true")],
                    cwd: None,
                    env: BTreeMap::new(),
                }))
                .expect("guest exec request should encode"),
            )
            .send()
            .await
            .expect("guest exec request should complete");
        let status = response.status();
        let body = response
            .text()
            .await
            .expect("guest exec body should decode");
        assert_eq!(status, StatusCode::BAD_GATEWAY, "{body}");
        let error: HostedError =
            serde_json::from_str(&body).expect("guest exec body should decode");
        assert!(
            error.message.contains("guest agent socket"),
            "{}",
            error.message
        );
        assert_eq!(
            error
                .route
                .as_ref()
                .and_then(|route| route.node_name.as_deref()),
            Some("aws-linux-node")
        );

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[test]
    fn hosted_registry_persistence_surfaces_control_plane_and_path_context_on_decode_failure() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("decode-failure");
        let token_var = unique_test_env("PORT_TEST_DECODE_FAILURE_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let state_path = registered_node_state_path(&control_plane);
        std::fs::create_dir_all(
            state_path
                .parent()
                .expect("registered state path should have a parent"),
        )
        .expect("registered state dir should exist");
        std::fs::write(&state_path, b"{not-valid-json")
            .expect("corrupt registered state should write");

        let error = match build_state(
            config,
            ControlPlaneServeRequest {
                control_plane: control_plane.clone(),
                bind: reserve_test_addr(),
                node_bindings: Vec::new(),
            },
        ) {
            Ok(_) => panic!("corrupt registered state should fail to load"),
            Err(error) => error,
        };
        let message = format!("{error:#}");
        assert!(message.contains(&format!("control plane '{}'", control_plane)));
        assert!(
            message.contains(state_path.to_string_lossy().as_ref()),
            "{message}"
        );
        assert!(message.contains("registered node state"), "{message}");

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[test]
    fn node_agent_surfaces_explicit_registration_failures() {
        let tempdir = TempDir::new().expect("tempdir should be created");

        {
            let mut config = sample_control_plane_config(tempdir.path());
            let control_plane = unique_test_control_plane("auth-mismatch");
            let token_var = unique_test_env("PORT_TEST_NODE_AUTH_MISMATCH");
            retarget_demo_control_plane(&mut config, &control_plane, &token_var);
            cleanup_registered_state(&control_plane);
            let control_bind = reserve_test_addr();
            config
                .control_planes
                .get_mut(&control_plane)
                .expect("control plane should exist")
                .endpoint = format!("http://{control_bind}");
            unsafe {
                std::env::set_var(&token_var, "demo-token");
            }

            let control_config = config.clone();
            let control_plane_name = control_plane.clone();
            let control_bind_for_thread = control_bind.clone();
            std::thread::spawn(move || {
                let _ = serve_control_plane(
                    control_config,
                    ControlPlaneServeRequest {
                        control_plane: control_plane_name,
                        bind: control_bind_for_thread,
                        node_bindings: Vec::new(),
                    },
                );
            });

            wait_for(Duration::from_secs(5), Duration::from_millis(50), || {
                let response = reqwest::blocking::Client::new()
                    .get(format!("http://{control_bind}/v1/machines"))
                    .header("authorization", "Bearer demo-token")
                    .send();
                matches!(response, Ok(response) if response.status() == StatusCode::OK)
            });

            unsafe {
                std::env::set_var(&token_var, "wrong-token");
            }
            let error = serve_node_agent(
                config,
                NodeAgentServeRequest {
                    node_name: String::from("aws-linux-node"),
                    bind: reserve_test_addr(),
                    token: String::from("node-secret"),
                },
            )
            .expect_err("auth mismatch should fail registration");
            let error_text = error.to_string();
            assert!(error_text.contains("registration"));
            assert!(
                error_text.contains("authorization")
                    || error_text.contains("status 401")
                    || error_text.contains("expects a bearer token"),
                "{error_text}"
            );
            cleanup_registered_state(&control_plane);
            unsafe {
                std::env::remove_var(&token_var);
            }
        }

        {
            let mut config = sample_control_plane_config(tempdir.path());
            let control_plane = unique_test_control_plane("unreachable");
            let token_var = unique_test_env("PORT_TEST_NODE_UNREACHABLE");
            retarget_demo_control_plane(&mut config, &control_plane, &token_var);
            cleanup_registered_state(&control_plane);
            config
                .control_planes
                .get_mut(&control_plane)
                .expect("control plane should exist")
                .endpoint = String::from("http://127.0.0.1:9");
            unsafe {
                std::env::set_var(&token_var, "demo-token");
            }
            let error = serve_node_agent(
                config,
                NodeAgentServeRequest {
                    node_name: String::from("aws-linux-node"),
                    bind: reserve_test_addr(),
                    token: String::from("node-secret"),
                },
            )
            .expect_err("unreachable control plane should fail registration");
            let error_text = error.to_string();
            assert!(error_text.contains("registration"));
            assert!(error_text.contains("could not reach"), "{error_text}");
            cleanup_registered_state(&control_plane);
            unsafe {
                std::env::remove_var(&token_var);
            }
        }

        {
            let mut config = sample_control_plane_config(tempdir.path());
            let control_plane = unique_test_control_plane("stale");
            let token_var = unique_test_env("PORT_TEST_NODE_STALE");
            retarget_demo_control_plane(&mut config, &control_plane, &token_var);
            cleanup_registered_state(&control_plane);
            let control_bind = reserve_test_addr();
            config
                .control_planes
                .get_mut(&control_plane)
                .expect("control plane should exist")
                .endpoint = format!("http://{control_bind}");
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix time should exist")
                .as_secs();
            let stale_path = registered_node_state_path(&control_plane);
            std::fs::create_dir_all(
                stale_path
                    .parent()
                    .expect("registered state path should have a parent"),
            )
            .expect("registered state dir should exist");
            std::fs::write(
                &stale_path,
                serde_json::to_vec_pretty(&RegisteredNodeStateFile {
                    control_plane: control_plane.clone(),
                    nodes: BTreeMap::from([(
                        String::from("aws-linux-node"),
                        HostedNodeRegistration {
                            endpoint: String::from("http://127.0.0.1:9234"),
                            token: String::from("node-secret"),
                            registered_at: now + 60,
                            refreshed_at: now + 60,
                            ttl_seconds: 30,
                        },
                    )]),
                })
                .expect("stale registered state should encode"),
            )
            .expect("stale registered state should write");
            unsafe {
                std::env::set_var(&token_var, "demo-token");
            }

            let control_config = config.clone();
            let control_plane_name = control_plane.clone();
            let control_bind_for_thread = control_bind.clone();
            std::thread::spawn(move || {
                let _ = serve_control_plane(
                    control_config,
                    ControlPlaneServeRequest {
                        control_plane: control_plane_name,
                        bind: control_bind_for_thread,
                        node_bindings: Vec::new(),
                    },
                );
            });

            wait_for(Duration::from_secs(5), Duration::from_millis(50), || {
                let response = reqwest::blocking::Client::new()
                    .get(format!("http://{control_bind}/v1/machines"))
                    .header("authorization", "Bearer demo-token")
                    .send();
                matches!(response, Ok(response) if response.status() == StatusCode::OK)
            });

            let error = serve_node_agent(
                config,
                NodeAgentServeRequest {
                    node_name: String::from("aws-linux-node"),
                    bind: reserve_test_addr(),
                    token: String::from("node-secret"),
                },
            )
            .expect_err("stale registration should fail");
            assert!(error.to_string().contains("stale"));
            assert!(error.to_string().contains("aws-linux-node"));
            cleanup_registered_state(&control_plane);
            unsafe {
                std::env::remove_var(&token_var);
            }
        }
    }

    #[tokio::test]
    async fn control_plane_rejects_invalid_client_token() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let state = build_state(
            config,
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
                bind: addr.to_string(),
                node_bindings: Vec::new(),
            },
        )
        .expect("state should build");
        let server = tokio::spawn(async move {
            let _ = axum::serve(listener, control_plane_router(state)).await;
        });

        let response = Client::new()
            .get(format!("http://{addr}/v1/machines/cloud-aws"))
            .header("authorization", "Bearer wrong")
            .send()
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let error: HostedError = response.json().await.expect("error body should decode");
        assert!(error.message.contains("control plane 'demo'"));
        assert!(error.message.contains("authorization"));

        server.abort();
    }

    #[tokio::test]
    async fn control_plane_proxies_machine_and_guest_routes_to_node_agent() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let mock_state = MockNodeState {
            node_name: String::from("aws-linux-node"),
            runtime_root: PathBuf::from("runtime/hosted/aws-linux-node"),
            pid: 9876,
            headers: Arc::new(Mutex::new(Vec::new())),
            bodies: Arc::new(Mutex::new(Vec::new())),
        };
        let node_addr = serve_mock_node_agent(mock_state.clone()).await;
        let control_addr = serve_test_control_plane(
            config,
            vec![HostedNodeBinding {
                node_name: String::from("aws-linux-node"),
                endpoint: format!("http://{node_addr}"),
                token: String::from("node-secret"),
            }],
        )
        .await;

        let client = Client::new();
        let status = client
            .get(format!("http://{control_addr}/v1/machines/cloud-aws"))
            .header("authorization", "Bearer demo-token")
            .send()
            .await
            .expect("status request should complete");
        assert_eq!(status.status(), StatusCode::OK);
        let status_body: HostedSuccess<MachineStatus> =
            status.json().await.expect("status body should decode");
        assert_eq!(status_body.result.machine_name, "cloud-aws");

        let launch = client
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws:launch"
            ))
            .header("authorization", "Bearer demo-token")
            .send()
            .await
            .expect("launch request should complete");
        assert_eq!(launch.status(), StatusCode::OK);
        let launch_body: HostedSuccess<LaunchMetadata> =
            launch.json().await.expect("launch body should decode");
        assert_eq!(launch_body.result.machine_name, "cloud-aws");
        assert_eq!(launch_body.result.pid, 9876);
        assert_eq!(
            launch_body.route.node_name.as_deref(),
            Some("aws-linux-node")
        );

        let guest = client
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws/guest:exec"
            ))
            .header("authorization", "Bearer demo-token")
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_vec(&GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("hello")],
                    cwd: None,
                    env: BTreeMap::new(),
                }))
                .expect("guest request should encode"),
            )
            .send()
            .await
            .expect("guest request should complete");
        assert_eq!(guest.status(), StatusCode::OK);
        let guest_body: HostedSuccess<OperationResult> =
            guest.json().await.expect("guest body should decode");
        match guest_body.result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "node-ok\n"),
            other => panic!("unexpected guest result: {other:?}"),
        }

        let recorded_headers = mock_state.headers.lock().expect("headers lock");
        assert!(recorded_headers.iter().all(|value| value == "node-secret"));
        let recorded_bodies = mock_state.bodies.lock().expect("bodies lock");
        assert!(
            recorded_bodies
                .iter()
                .any(|body| body.contains("\"type\":\"exec\""))
        );
    }

    #[tokio::test]
    async fn control_plane_launch_selects_registered_node_deterministically_and_persists_placement()
    {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("placement");
        let token_var = unique_test_env("PORT_TEST_PLACEMENT_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let mut preferred_node = config
            .nodes
            .get("aws-linux-node")
            .expect("aws node should exist")
            .clone();
        preferred_node.runtime_root = tempdir.path().join("hosted/aaa-linux-node");
        config
            .nodes
            .insert(String::from("aaa-linux-node"), preferred_node.clone());

        let preferred_state = MockNodeState {
            node_name: String::from("aaa-linux-node"),
            runtime_root: preferred_node.runtime_root.clone(),
            pid: 1111,
            headers: Arc::new(Mutex::new(Vec::new())),
            bodies: Arc::new(Mutex::new(Vec::new())),
        };
        let fallback_state = MockNodeState {
            node_name: String::from("aws-linux-node"),
            runtime_root: config.nodes["aws-linux-node"].runtime_root.clone(),
            pid: 2222,
            headers: Arc::new(Mutex::new(Vec::new())),
            bodies: Arc::new(Mutex::new(Vec::new())),
        };
        let preferred_addr = serve_mock_node_agent_named(preferred_state.clone()).await;
        let fallback_addr = serve_mock_node_agent_named(fallback_state.clone()).await;
        let now = current_unix_timestamp_seconds().expect("unix timestamp should resolve");
        persist_registered_node_state(
            &registered_node_state_path(&control_plane),
            &RegisteredNodeStateFile {
                control_plane: control_plane.clone(),
                nodes: BTreeMap::from([
                    (
                        String::from("aaa-linux-node"),
                        HostedNodeRegistration {
                            endpoint: format!("http://{preferred_addr}"),
                            token: String::from("node-secret"),
                            registered_at: now,
                            refreshed_at: now,
                            ttl_seconds: 30,
                        },
                    ),
                    (
                        String::from("aws-linux-node"),
                        HostedNodeRegistration {
                            endpoint: format!("http://{fallback_addr}"),
                            token: String::from("node-secret"),
                            registered_at: now,
                            refreshed_at: now,
                            ttl_seconds: 30,
                        },
                    ),
                ]),
            },
        )
        .expect("registered state should persist");

        let control_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;
        let client = Client::new();
        for _ in 0..2 {
            let launch = client
                .post(format!(
                    "http://{control_addr}/v1/machines/cloud-aws:launch"
                ))
                .header("authorization", "Bearer demo-token")
                .send()
                .await
                .expect("launch request should complete");
            assert_eq!(launch.status(), StatusCode::OK);
            let body: HostedSuccess<LaunchMetadata> =
                launch.json().await.expect("launch body should decode");
            assert_eq!(body.route.node_name.as_deref(), Some("aaa-linux-node"));
            assert_eq!(body.result.pid, 1111);
        }

        assert_eq!(
            preferred_state.headers.lock().expect("headers lock").len(),
            2
        );
        assert!(
            fallback_state
                .headers
                .lock()
                .expect("headers lock")
                .is_empty()
        );

        let placements: MachinePlacementStateFile = serde_json::from_slice(
            &std::fs::read(machine_placement_state_path(&control_plane))
                .expect("machine placement state should read"),
        )
        .expect("machine placement state should decode");
        let placement = placements
            .machines
            .get("cloud-aws")
            .expect("cloud-aws placement should persist");
        assert_eq!(placement.node_name, "aaa-linux-node");
        assert_eq!(placement.runtime_root, preferred_node.runtime_root);
        assert!(
            placement
                .placement_detail
                .as_deref()
                .unwrap_or_default()
                .contains("aaa-linux-node")
        );

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[tokio::test]
    async fn hosted_registry_persistence_control_plane_launch_rejects_stale_registered_node_with_explicit_detail()
     {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        let control_plane = unique_test_control_plane("stale-placement");
        let token_var = unique_test_env("PORT_TEST_STALE_PLACEMENT_TOKEN");
        retarget_demo_control_plane(&mut config, &control_plane, &token_var);
        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::set_var(&token_var, "demo-token");
        }

        let now = current_unix_timestamp_seconds().expect("unix timestamp should resolve");
        persist_registered_node_state(
            &registered_node_state_path(&control_plane),
            &RegisteredNodeStateFile {
                control_plane: control_plane.clone(),
                nodes: BTreeMap::from([(
                    String::from("aws-linux-node"),
                    HostedNodeRegistration {
                        endpoint: String::from("http://127.0.0.1:9"),
                        token: String::from("node-secret"),
                        registered_at: now.saturating_sub(10),
                        refreshed_at: now.saturating_sub(10),
                        ttl_seconds: 1,
                    },
                )]),
            },
        )
        .expect("registered state should persist");

        let control_addr =
            serve_test_control_plane_named(&config, &control_plane, Vec::new()).await;
        let response = Client::new()
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws:launch"
            ))
            .header("authorization", "Bearer demo-token")
            .send()
            .await
            .expect("launch request should complete");
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let error: HostedError = response.json().await.expect("error body should decode");
        assert!(error.message.contains("stale"), "{}", error.message);
        assert!(error.message.contains("aws-linux-node"));
        assert!(error.message.contains("cloud-aws"));
        let route = error.route.expect("route context should exist");
        assert_eq!(route.control_plane.as_deref(), Some(control_plane.as_str()));
        assert_eq!(route.machine_name.as_deref(), Some("cloud-aws"));
        assert_eq!(route.candidate_nodes, vec![String::from("aws-linux-node")]);

        cleanup_registered_state(&control_plane);
        unsafe {
            std::env::remove_var(&token_var);
        }
    }

    #[tokio::test]
    async fn control_plane_proxies_copy_stream_through_node_agent_guest_transport() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        std::fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest socket should bind");
        let server = std::thread::spawn(move || {
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
                        path: String::from("/client/downloaded.txt"),
                        direction: CopyDirection::GuestToHost,
                    }),
                },
            )
            .expect("download completion should encode");
        });

        let node_addr =
            serve_test_node_agent(config.clone(), "aws-linux-node", "node-secret").await;
        let control_addr = serve_test_control_plane(
            config,
            vec![HostedNodeBinding {
                node_name: String::from("aws-linux-node"),
                endpoint: format!("http://{node_addr}"),
                token: String::from("node-secret"),
            }],
        )
        .await;

        let client = Client::new();
        let mut upload_body = Vec::new();
        write_frame(
            &mut upload_body,
            &RequestEnvelope {
                id: 1,
                operation: GuestOperation::Copy(CopyRequest {
                    source: String::from("/client/source.txt"),
                    destination: String::from("/workspace/copied.txt"),
                    direction: CopyDirection::HostToGuest,
                    size_bytes: Some(7),
                }),
            },
        )
        .expect("upload request should encode");
        upload_body.extend_from_slice(b"copy-ok");

        let upload = client
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws/guest:copy:stream"
            ))
            .header("authorization", "Bearer demo-token")
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(upload_body)
            .send()
            .await
            .expect("upload request should complete");
        assert_eq!(upload.status(), StatusCode::OK);
        let upload_bytes = upload.bytes().await.expect("upload body should read");
        let mut upload_reader = BufReader::new(Cursor::new(upload_bytes.to_vec()));
        let upload_accepted: ResponseEnvelope =
            read_frame(&mut upload_reader).expect("upload accepted should decode");
        assert!(matches!(
            upload_accepted,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                ..
            }
        ));
        let upload_completed: ResponseEnvelope =
            read_frame(&mut upload_reader).expect("upload completion should decode");
        match upload_completed {
            ResponseEnvelope::Completed {
                result: OperationResult::Copy(result),
                ..
            } => {
                assert_eq!(result.bytes_copied, 7);
                assert_eq!(result.path, "/workspace/copied.txt");
            }
            other => panic!("unexpected upload completion: {other:?}"),
        }

        let mut download_body = Vec::new();
        write_frame(
            &mut download_body,
            &RequestEnvelope {
                id: 1,
                operation: GuestOperation::Copy(CopyRequest {
                    source: String::from("/workspace/copied.txt"),
                    destination: String::from("/client/downloaded.txt"),
                    direction: CopyDirection::GuestToHost,
                    size_bytes: None,
                }),
            },
        )
        .expect("download request should encode");
        let download = client
            .post(format!(
                "http://{control_addr}/v1/machines/cloud-aws/guest:copy:stream"
            ))
            .header("authorization", "Bearer demo-token")
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(download_body)
            .send()
            .await
            .expect("download request should complete");
        assert_eq!(download.status(), StatusCode::OK);
        let download_bytes = download.bytes().await.expect("download body should read");
        let mut download_reader = BufReader::new(Cursor::new(download_bytes.to_vec()));
        let download_accepted: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download accepted should decode");
        let size_bytes = match download_accepted {
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                size_bytes: Some(size_bytes),
                ..
            } => size_bytes,
            other => panic!("unexpected download accepted frame: {other:?}"),
        };
        let mut downloaded = vec![0_u8; size_bytes as usize];
        download_reader
            .read_exact(&mut downloaded)
            .expect("download bytes should read");
        assert_eq!(&downloaded, b"copy-ok");
        let download_completed: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download completion should decode");
        match download_completed {
            ResponseEnvelope::Completed {
                result: OperationResult::Copy(result),
                ..
            } => {
                assert_eq!(result.bytes_copied, 7);
                assert_eq!(result.path, "/client/downloaded.txt");
            }
            other => panic!("unexpected download completion: {other:?}"),
        }

        server.join().expect("guest copy server should complete");
    }

    #[tokio::test]
    async fn control_plane_reports_missing_node_binding_with_route_context() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let control_addr = serve_test_control_plane(config, Vec::new()).await;
        let response = Client::new()
            .get(format!("http://{control_addr}/v1/machines/cloud-aws"))
            .header("authorization", "Bearer demo-token")
            .send()
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::OK);
        let success: HostedSuccess<MachineStatus> =
            response.json().await.expect("success body should decode");
        assert_eq!(success.result.state, MachineRuntimeState::Malformed);
        assert!(success.result.detail.contains("aws-linux-node"));
        assert_eq!(success.route.control_plane, Some(String::from("demo")));
        assert_eq!(success.result.hosted_fleet_nodes.len(), 1);
        assert_eq!(
            success.result.hosted_fleet_nodes[0].routing_eligibility,
            crate::HostedFleetRoutingEligibility::MissingRegistration
        );
    }

    #[tokio::test]
    async fn hosted_fleet_state_reports_merge_failures_with_control_plane_and_node_detail() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        cleanup_registered_state("demo");
        unsafe {
            std::env::set_var("PORT_DEMO_TOKEN", "demo-token");
        }

        let state = build_state(
            config.clone(),
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
                bind: reserve_test_addr(),
                node_bindings: Vec::new(),
            },
        )
        .expect("state should build");
        state
            .inner
            .imported_inventory
            .write()
            .expect("imported inventory lock")
            .insert(
                String::from("aws-linux-node"),
                ImportedNodeRecord {
                    node_name: String::from("aws-linux-node"),
                    provider: HostProvider::Gcp,
                    provenance: String::from("inventory-sync"),
                    imported_at: 123,
                    capability_summary: config.nodes["aws-linux-node"].capabilities.clone(),
                },
            );

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer demo-token".parse().expect("header should parse"),
        );
        let response = machine_status(State(state), Path(String::from("cloud-aws")), headers).await;
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let error: HostedError = serde_json::from_slice(&body).expect("error should decode");
        assert!(
            error.message.contains("control plane 'demo'"),
            "{}",
            error.message
        );
        assert!(error.message.contains("cloud-aws"), "{}", error.message);
        assert!(
            error.message.contains("aws-linux-node"),
            "{}",
            error.message
        );
        let route = error.route.expect("route context should exist");
        assert_eq!(route.control_plane.as_deref(), Some("demo"));
        assert_eq!(route.machine_name.as_deref(), Some("cloud-aws"));
        cleanup_registered_state("demo");
    }

    #[tokio::test]
    async fn node_agent_rejects_invalid_token() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());

        let node_addr = serve_test_node_agent(config, "aws-linux-node", "node-secret").await;
        let response = Client::new()
            .get(format!("http://{node_addr}/v1/node/machines/cloud-aws"))
            .header("x-port-node-agent-token", "wrong")
            .send()
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let error: HostedError = response.json().await.expect("error body should decode");
        assert!(error.message.contains("aws-linux-node"));
        assert!(error.message.contains("x-port-node-agent-token"));
    }

    #[tokio::test]
    async fn node_agent_serves_status_and_guest_exec_from_runtime_root() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        let runtime_root = config.nodes["aws-linux-node"].runtime_root.clone();
        let paths = RuntimePaths::for_machine(&runtime_root, "cloud-aws");
        write_manifest(&paths, "cloud-aws", 424242);
        std::fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let listener =
            UnixListener::bind(&paths.guest_agent_socket).expect("guest socket should bind");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("guest socket should accept");
            let reader_stream = stream.try_clone().expect("stream should clone");
            let mut reader = BufReader::new(reader_stream);
            let request: RequestEnvelope = read_frame(&mut reader).expect("request should decode");
            match request.operation {
                GuestOperation::Exec(request) => {
                    assert_eq!(
                        request.command,
                        vec![String::from("/bin/echo"), String::from("node-ok")]
                    );
                }
                other => panic!("unexpected operation: {other:?}"),
            }

            write_frame(
                &mut stream,
                &ResponseEnvelope::Completed {
                    id: 1,
                    exit_code: 0,
                    result: OperationResult::Exec(ExecResult {
                        stdout: String::from("node-ok\n"),
                        stderr: String::new(),
                    }),
                },
            )
            .expect("response should encode");
        });

        let node_addr = serve_test_node_agent(config, "aws-linux-node", "node-secret").await;
        let client = Client::new();

        let status = client
            .get(format!("http://{node_addr}/v1/node/machines/cloud-aws"))
            .header("x-port-node-agent-token", "node-secret")
            .send()
            .await
            .expect("status request should complete");
        assert_eq!(status.status(), StatusCode::OK);
        let status_body: HostedSuccess<MachineStatus> =
            status.json().await.expect("status body should decode");
        assert_eq!(status_body.result.machine_name, "cloud-aws");
        assert_eq!(
            status_body.route.node_name.as_deref(),
            Some("aws-linux-node")
        );

        let guest = client
            .post(format!(
                "http://{node_addr}/v1/node/machines/cloud-aws/guest:exec"
            ))
            .header("x-port-node-agent-token", "node-secret")
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_vec(&GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("node-ok")],
                    cwd: None,
                    env: BTreeMap::new(),
                }))
                .expect("guest request should encode"),
            )
            .send()
            .await
            .expect("guest request should complete");
        assert_eq!(guest.status(), StatusCode::OK);
        let guest_body: HostedSuccess<OperationResult> =
            guest.json().await.expect("guest body should decode");
        match guest_body.result {
            OperationResult::Exec(result) => assert_eq!(result.stdout, "node-ok\n"),
            other => panic!("unexpected result: {other:?}"),
        }

        server.join().expect("guest server thread should complete");
    }

    #[tokio::test]
    async fn node_agent_launches_pvm_machine_from_prepared_host_kit() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let mut config = sample_control_plane_config(tempdir.path());
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws machine should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let kernel_path = tempdir.path().join("pvm-vmlinux");
        let guest_path = tempdir.path().join("pvm-rootfs.ext4");
        std::fs::write(&kernel_path, b"fake-kernel").expect("kernel variant should write");
        std::fs::write(&guest_path, b"fake-rootfs").expect("guest variant should write");
        config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .expect("demo-kernel should exist")
            .variants
            .iter_mut()
            .find(|variant| {
                variant.selector.architecture == port_model::MachineArchitecture::X86_64
                    && variant.selector.substrate == port_model::ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == port_model::ProtectionMode::Pvm
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
                variant.selector.architecture == port_model::MachineArchitecture::X86_64
                    && variant.selector.substrate == port_model::ExecutionSubstrate::Firecracker
                    && variant.selector.protection_mode == port_model::ProtectionMode::Pvm
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
            .expect("aws node should declare a host-kit contract");
        host_kit.requires_custom_host_kernel = false;
        host_kit.host_boot_args.clear();
        host_kit.firecracker_binary_env = Some(String::from("PORT_TEST_NODE_PVM_FIRECRACKER"));
        let fake_binary = write_fake_firecracker(tempdir.path(), "firecracker-pvm");
        unsafe {
            std::env::set_var("PORT_TEST_NODE_PVM_FIRECRACKER", &fake_binary);
        }

        let node_addr = serve_test_node_agent(config, "aws-linux-node", "node-secret").await;
        let response = Client::new()
            .post(format!(
                "http://{node_addr}/v1/node/machines/cloud-aws:launch"
            ))
            .header("x-port-node-agent-token", "node-secret")
            .send()
            .await
            .expect("launch request should complete");
        let status = response.status();
        let body = response.text().await.expect("launch body should decode");
        assert_eq!(status, StatusCode::OK, "{body}");

        let success: serde_json::Value =
            serde_json::from_str(&body).expect("launch body should decode");
        let result = &success["result"];
        assert_eq!(result["machine_name"], "cloud-aws");
        assert_eq!(
            result["firecracker_binary"].as_str(),
            Some(fake_binary.to_string_lossy().as_ref())
        );
        assert_eq!(
            success["route"]["node_name"].as_str(),
            Some("aws-linux-node")
        );

        let config_path = PathBuf::from(
            result["config_path"]
                .as_str()
                .expect("config path should be present"),
        );
        let manifest_path = PathBuf::from(
            result["manifest_path"]
                .as_str()
                .expect("manifest path should be present"),
        );
        let config_json = std::fs::read_to_string(&config_path).expect("config should exist");
        assert!(config_json.contains(kernel_path.to_string_lossy().as_ref()));
        assert!(config_json.contains(guest_path.to_string_lossy().as_ref()));
        assert!(manifest_path.exists());

        let pid = result["pid"].as_u64().expect("pid should be present");
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
    }

    #[tokio::test]
    async fn node_agent_reports_missing_guest_socket_with_runtime_context() {
        let tempdir = TempDir::new().expect("tempdir should be created");
        let config = sample_control_plane_config(tempdir.path());
        let node_addr = serve_test_node_agent(config, "aws-linux-node", "node-secret").await;

        let response = Client::new()
            .post(format!(
                "http://{node_addr}/v1/node/machines/cloud-aws/guest:exec"
            ))
            .header("x-port-node-agent-token", "node-secret")
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_vec(&GuestOperation::Exec(ExecRequest {
                    command: vec![String::from("/bin/true")],
                    cwd: None,
                    env: BTreeMap::new(),
                }))
                .expect("guest request should encode"),
            )
            .send()
            .await
            .expect("guest request should complete");
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let error: HostedError = response.json().await.expect("error body should decode");
        assert!(error.message.contains("guest agent socket"));
        let route = error.route.expect("route context should exist");
        assert_eq!(route.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            route.runtime_root,
            Some(tempdir.path().join("hosted/aws-linux-node"))
        );
    }

    async fn serve_test_control_plane(
        config: PortConfig,
        node_bindings: Vec<HostedNodeBinding>,
    ) -> SocketAddr {
        cleanup_registered_state("demo");
        serve_test_control_plane_named(&config, "demo", node_bindings).await
    }

    async fn serve_test_control_plane_named(
        config: &PortConfig,
        control_plane: &str,
        node_bindings: Vec<HostedNodeBinding>,
    ) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let state = build_state(
            config.clone(),
            ControlPlaneServeRequest {
                control_plane: control_plane.to_string(),
                bind: addr.to_string(),
                node_bindings,
            },
        )
        .expect("state should build");
        tokio::spawn(async move {
            let _ = axum::serve(listener, control_plane_router(state)).await;
        });
        addr
    }

    async fn serve_mock_node_agent(state: MockNodeState) -> SocketAddr {
        serve_mock_node_agent_named(state).await
    }

    async fn serve_mock_node_agent_named(state: MockNodeState) -> SocketAddr {
        async fn ready_handler() -> StatusCode {
            StatusCode::OK
        }

        async fn status_handler(
            State(state): State<MockNodeState>,
            headers: HeaderMap,
            Path(machine): Path<String>,
        ) -> Json<HostedSuccess<MachineStatus>> {
            state.headers.lock().expect("headers lock").push(
                headers
                    .get("x-port-node-agent-token")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
            );
            Json(HostedSuccess {
                route: HostedRouteContext {
                    control_plane: Some(String::from("demo")),
                    machine_name: Some(machine.clone()),
                    node_name: Some(state.node_name.clone()),
                    ..HostedRouteContext::default()
                },
                result: MachineStatus {
                    machine_name: machine,
                    state: MachineRuntimeState::Running,
                    pid: Some(4321),
                    control: port_model::MachineControlContract::hosted_control_plane(),
                    runtime_dir: state.runtime_root.join("cloud-aws"),
                    config_path: state.runtime_root.join("cloud-aws/firecracker-config.json"),
                    manifest_path: state.runtime_root.join("cloud-aws/manifest.json"),
                    pid_path: state.runtime_root.join("cloud-aws/firecracker.pid"),
                    firecracker_log: state.runtime_root.join("cloud-aws/firecracker.log"),
                    stdout_log: state.runtime_root.join("cloud-aws/console.stdout.log"),
                    stderr_log: state.runtime_root.join("cloud-aws/console.stderr.log"),
                    hosted_fleet_nodes: Vec::new(),
                    detail: String::from("mock status"),
                },
            })
        }

        async fn guest_handler(
            State(state): State<MockNodeState>,
            headers: HeaderMap,
            body: Bytes,
        ) -> Json<HostedSuccess<OperationResult>> {
            state.headers.lock().expect("headers lock").push(
                headers
                    .get("x-port-node-agent-token")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
            );
            state
                .bodies
                .lock()
                .expect("bodies lock")
                .push(String::from_utf8(body.to_vec()).expect("body should be utf8"));
            Json(HostedSuccess {
                route: HostedRouteContext {
                    control_plane: Some(String::from("demo")),
                    machine_name: Some(String::from("cloud-aws")),
                    node_name: Some(state.node_name.clone()),
                    ..HostedRouteContext::default()
                },
                result: OperationResult::Exec(ExecResult {
                    stdout: String::from("node-ok\n"),
                    stderr: String::new(),
                }),
            })
        }

        async fn launch_handler(
            State(state): State<MockNodeState>,
            headers: HeaderMap,
            Path(machine): Path<String>,
        ) -> Json<HostedSuccess<LaunchMetadata>> {
            state.headers.lock().expect("headers lock").push(
                headers
                    .get("x-port-node-agent-token")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
            );
            let machine_name = machine
                .strip_suffix(":launch")
                .expect("launch route should preserve machine suffix")
                .to_string();
            Json(HostedSuccess {
                route: HostedRouteContext {
                    control_plane: Some(String::from("demo")),
                    machine_name: Some(machine_name.clone()),
                    node_name: Some(state.node_name.clone()),
                    ..HostedRouteContext::default()
                },
                result: LaunchMetadata {
                    machine_name,
                    pid: state.pid,
                    launched_at_unix_s: 1,
                    runtime_dir: state.runtime_root.join("cloud-aws"),
                    firecracker_binary: PathBuf::from("/usr/bin/firecracker-pvm"),
                    config_path: state.runtime_root.join("cloud-aws/firecracker-config.json"),
                    log_path: state.runtime_root.join("cloud-aws/firecracker.log"),
                    stdout_path: state.runtime_root.join("cloud-aws/console.stdout.log"),
                    stderr_path: state.runtime_root.join("cloud-aws/console.stderr.log"),
                    manifest_path: state.runtime_root.join("cloud-aws/manifest.json"),
                },
            })
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let router = Router::new()
            .route("/__ready", get(ready_handler))
            .route(
                "/v1/node/machines/{machine}",
                get(status_handler).post(launch_handler),
            )
            .route(
                "/v1/node/machines/{machine}/guest:exec",
                post(guest_handler),
            )
            .with_state(state);
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        wait_for_http_ready(addr, "/__ready", &[], true).await;
        addr
    }

    fn retarget_demo_control_plane(config: &mut PortConfig, control_plane: &str, token_var: &str) {
        let mut spec = config
            .control_planes
            .remove("demo")
            .expect("demo control plane should exist");
        spec.auth.source = port_model::HostedAuthTokenSource::Env {
            variable: token_var.to_string(),
        };
        config
            .control_planes
            .insert(control_plane.to_string(), spec);
        for host in config.hosts.values_mut() {
            if let HostConnection::HostedControlPlane {
                control_plane: current,
            } = &mut host.connection
            {
                *current = control_plane.to_string();
            }
        }
    }

    fn unique_test_control_plane(label: &str) -> String {
        format!("demo-{label}-{}", std::process::id())
    }

    fn unique_test_env(prefix: &str) -> String {
        format!("{prefix}_{}", std::process::id())
    }

    fn reserve_test_addr() -> String {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("temporary listener should bind")
            .local_addr()
            .expect("temporary listener should have an addr")
            .to_string()
    }

    fn wait_for(timeout: Duration, poll: Duration, mut predicate: impl FnMut() -> bool) {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if predicate() {
                return;
            }
            std::thread::sleep(poll);
        }
        panic!("timed out after {:?}", timeout);
    }

    async fn wait_for_http_ready(
        addr: SocketAddr,
        path: &str,
        headers: &[(&str, &str)],
        require_success: bool,
    ) {
        let client = Client::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let mut request = client.get(format!("http://{addr}{path}"));
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
            if let Ok(response) = request.send().await {
                if !require_success || response.status().is_success() {
                    return;
                }
            }
            if std::time::Instant::now() >= deadline {
                panic!("timed out after {:?}", Duration::from_secs(5));
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    fn cleanup_registered_state(control_plane: &str) {
        let path = registered_node_state_path(control_plane);
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    async fn serve_test_node_agent(config: PortConfig, node_name: &str, token: &str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let state = build_node_agent_state(
            config,
            NodeAgentServeRequest {
                node_name: node_name.to_string(),
                bind: addr.to_string(),
                token: token.to_string(),
            },
        )
        .expect("node-agent state should build");
        tokio::spawn(async move {
            let _ = axum::serve(listener, node_agent_router(state)).await;
        });
        wait_for_http_ready(
            addr,
            "/v1/node/machines/cloud-aws",
            &[("x-port-node-agent-token", token)],
            false,
        )
        .await;
        addr
    }

    fn write_fake_firecracker(root: &std::path::Path, name: &str) -> PathBuf {
        let path = root.join(name);
        std::fs::write(&path, "#!/usr/bin/env bash\nsleep 30\n")
            .expect("fake firecracker should write");
        let mut permissions = std::fs::metadata(&path)
            .expect("fake firecracker metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions)
            .expect("fake firecracker permissions should update");
        path
    }

    fn write_manifest(paths: &RuntimePaths, machine_name: &str, pid: u32) {
        std::fs::create_dir_all(&paths.runtime_dir).expect("runtime dir should exist");
        let manifest = crate::LaunchMetadata {
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
        std::fs::write(
            &paths.manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");
    }

    fn sample_control_plane_config(root: &std::path::Path) -> PortConfig {
        let mut config = PortConfig::sample();
        config
            .control_planes
            .get_mut("demo")
            .expect("demo control plane")
            .endpoint = String::from("http://127.0.0.1:0");
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws node should exist")
            .runtime_root = root.join("hosted/aws-linux-node");
        config
    }
}
