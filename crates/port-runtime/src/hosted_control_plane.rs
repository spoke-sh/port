use std::collections::BTreeMap;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::{get, post, put};
use port_hosted_protocol::{
    HostedDetachedForwardRoute, HostedDetachedForwardStartRequest, HostedError, HostedGuestRoute,
    HostedGuestStreamRoute, HostedGuestVerb, HostedMachineRoute, HostedNodeAgentHeaders,
    HostedNodeRoute, HostedRouteContext, HostedServiceRoute, HostedSuccess,
};
use port_model::{
    ExecutionSubstrate, FirecrackerPvmLaneContract, HostConnection, HostedAuthTokenSource,
    HostedMachineSummaryContract, PortConfig, ProtectionMode,
};
use port_sdk::{SecretPutRequest, ServiceApplyRequest};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;

use crate::{
    DetachedForwardLaunchRequest, GuestCopyRequest, GuestForwardRequest, GuestRequest,
    LaunchMetadata, LaunchRequest, MachineRuntimeState, MachineStatus, RuntimePaths,
    ServiceApplyRequest as RuntimeServiceApplyRequest, ServiceSecretBinding, StopResult,
    apply_hosted_machine_service_live, copy_guest_file, copy_guest_via_endpoint,
    delete_machine_secret_local, execute_guest_operation, hosted_placeholder_runtime_root,
    launch_local_machine, list_detached_forwards, list_machine_secrets_local,
    machine_monitor as runtime_machine_monitor, machine_status as runtime_machine_status,
    machine_top as runtime_machine_top, prepare_guest_forward, put_machine_secret_local,
    refresh_hosted_machine_service_list, refresh_hosted_machine_service_runtime,
    start_detached_forward, stop_detached_forward, stop_hosted_machine_service_live,
    stop_machine as runtime_stop_machine,
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
    node_bindings: BTreeMap<String, HostedNodeBinding>,
    client: Client,
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
    token: String,
}

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
    format!("{detail} Routed through control plane '{control_plane}' and node '{node_name}'.")
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
    let node_bindings = request
        .node_bindings
        .into_iter()
        .map(|binding| (binding.node_name.clone(), binding))
        .collect();

    let auth_header = control_plane.auth.header.clone();

    Ok(ControlPlaneState {
        inner: Arc::new(ControlPlaneStateInner {
            config,
            control_plane: request.control_plane,
            auth_header,
            auth_value,
            node_bindings,
            client: Client::new(),
        }),
    })
}

fn control_plane_router(state: ControlPlaneState) -> Router {
    Router::new()
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
        match resolve_node_binding(&state, &summary) {
            Ok((binding, route)) => {
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
                    Ok(status) => machines.push(status.result),
                    Err(message) => {
                        machines.push(malformed_machine_status(&summary, route, message))
                    }
                }
            }
            Err((route, message)) => {
                machines.push(malformed_machine_status(&summary, route, message))
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
    match resolve_node_binding(&state, &summary) {
        Ok((binding, route)) => {
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
                Ok(status) => json_response(StatusCode::OK, &status),
                Err(message) => json_response(
                    StatusCode::OK,
                    &HostedSuccess {
                        route: route.clone(),
                        result: malformed_machine_status(&summary, route, message),
                    },
                ),
            }
        }
        Err((route, message)) => json_response(
            StatusCode::OK,
            &HostedSuccess {
                route: route.clone(),
                result: malformed_machine_status(&summary, route, message),
            },
        ),
    }
}

async fn machine_monitor(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Machine(HostedMachineRoute::Monitor {
            machine_name: machine.clone(),
        }),
        Method::GET,
        None,
    )
    .await
}

async fn machine_top(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Machine(HostedMachineRoute::Top {
            machine_name: machine.clone(),
        }),
        Method::GET,
        None,
    )
    .await
}

async fn machine_command(
    State(state): State<ControlPlaneState>,
    Path(machine): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(machine_name) = machine.strip_suffix(":launch") {
        return proxy_machine_route(
            &state,
            &headers,
            machine_name,
            HostedNodeRoute::Machine(HostedMachineRoute::Launch {
                machine_name: machine_name.to_string(),
            }),
            Method::POST,
            None,
        )
        .await;
    }

    if let Some(machine_name) = machine.strip_suffix(":stop") {
        return proxy_machine_route(
            &state,
            &headers,
            machine_name,
            HostedNodeRoute::Machine(HostedMachineRoute::Stop {
                machine_name: machine_name.to_string(),
            }),
            Method::POST,
            None,
        )
        .await;
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
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Service(HostedServiceRoute::List {
            machine_name: machine.clone(),
        }),
        Method::GET,
        None,
    )
    .await
}

async fn service_status(
    State(state): State<ControlPlaneState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    proxy_machine_route(
        &state,
        &headers,
        &machine,
        HostedNodeRoute::Service(HostedServiceRoute::Status {
            machine_name: machine.clone(),
            service_name: service,
        }),
        Method::GET,
        None,
    )
    .await
}

async fn service_command(
    State(state): State<ControlPlaneState>,
    Path((machine, service)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(service_name) = service.strip_suffix(":stop") {
        return proxy_machine_route(
            &state,
            &headers,
            &machine,
            HostedNodeRoute::Service(HostedServiceRoute::Stop {
                machine_name: machine.clone(),
                service_name: service_name.to_string(),
            }),
            Method::POST,
            None,
        )
        .await;
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

    for node_name in &summary.candidate_nodes {
        if let Some(binding) = state.inner.node_bindings.get(node_name) {
            let runtime_root = state
                .inner
                .config
                .nodes
                .get(node_name)
                .map(|node| node.runtime_root.clone())
                .unwrap_or_else(|| hosted_placeholder_runtime_root(&summary.control_plane));
            return Ok((
                binding.clone(),
                route_context
                    .clone()
                    .with_selected_node(node_name.clone(), runtime_root),
            ));
        }
    }

    Err((
        route_context,
        format!(
            "control plane '{}' could not route machine '{}' because none of the candidate nodes {:?} have a bound node-agent endpoint. {}",
            state.inner.control_plane,
            summary.machine_name,
            summary.candidate_nodes,
            summary.placement_detail
        ),
    ))
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
    for node_name in &group.nodes {
        if !summary
            .candidate_nodes
            .iter()
            .any(|candidate| candidate == node_name)
        {
            continue;
        }
        if let Some(binding) = state.inner.node_bindings.get(node_name) {
            eligible.push((node_name.clone(), binding.clone()));
        } else {
            missing_bindings.push(node_name.clone());
        }
    }
    eligible.sort_by(|left, right| left.0.cmp(&right.0));
    if let Some((node_name, binding)) = eligible.into_iter().next() {
        let runtime_root = state
            .inner
            .config
            .nodes
            .get(&node_name)
            .map(|node| node.runtime_root.clone())
            .unwrap_or_else(|| hosted_placeholder_runtime_root(&summary.control_plane));
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
            "; eligible nodes without node-agent bindings: {}",
            missing_bindings.join(", ")
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
        detail,
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
            token: request.token,
        }),
    })
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
    use std::io::{BufReader, Cursor, Read, Write};
    use std::net::SocketAddr;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    #[derive(Clone)]
    struct MockNodeState {
        headers: Arc<Mutex<Vec<String>>>,
        bodies: Arc<Mutex<Vec<String>>>,
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
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let state = build_state(
            config,
            ControlPlaneServeRequest {
                control_plane: String::from("demo"),
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
                    node_name: Some(String::from("aws-linux-node")),
                    ..HostedRouteContext::default()
                },
                result: MachineStatus {
                    machine_name: machine,
                    state: MachineRuntimeState::Running,
                    pid: Some(4321),
                    control: port_model::MachineControlContract::hosted_control_plane(),
                    runtime_dir: PathBuf::from("runtime/hosted/aws-linux-node/cloud-aws"),
                    config_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/firecracker-config.json",
                    ),
                    manifest_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/manifest.json",
                    ),
                    pid_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/firecracker.pid",
                    ),
                    firecracker_log: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/firecracker.log",
                    ),
                    stdout_log: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/console.stdout.log",
                    ),
                    stderr_log: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/console.stderr.log",
                    ),
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
                    node_name: Some(String::from("aws-linux-node")),
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
                    node_name: Some(String::from("aws-linux-node")),
                    ..HostedRouteContext::default()
                },
                result: LaunchMetadata {
                    machine_name,
                    pid: 9876,
                    launched_at_unix_s: 1,
                    runtime_dir: PathBuf::from("runtime/hosted/aws-linux-node/cloud-aws"),
                    firecracker_binary: PathBuf::from("/usr/bin/firecracker-pvm"),
                    config_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/firecracker-config.json",
                    ),
                    log_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/firecracker.log",
                    ),
                    stdout_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/console.stdout.log",
                    ),
                    stderr_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/console.stderr.log",
                    ),
                    manifest_path: PathBuf::from(
                        "runtime/hosted/aws-linux-node/cloud-aws/manifest.json",
                    ),
                },
            })
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("addr should exist");
        let router = Router::new()
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
        addr
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
