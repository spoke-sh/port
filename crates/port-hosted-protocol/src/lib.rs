use std::collections::BTreeMap;
use std::path::PathBuf;

use port_model::{
    ArtifactReference, ArtifactSelector, GuestCommandVerb, HostedApiIdentityContract,
    HostedAuthScheme, HostedGuestAttachContract, HostedGuestProtocolContract,
    HostedMachineSummaryContract, HostedNodeRegistration, HostedSchedulerPolicy,
    MachineArchitecture, MachineCommandRoute, MachineGuestBroker, MachineInventoryOwner,
    MachineLifecycleOwner, PvmHostKitPackage,
};
use serde::{Deserialize, Serialize};

pub const PORT_AUDIENCE_HEADER: &str = "x-port-audience";
pub const PORT_NODE_AGENT_TOKEN_HEADER: &str = "x-port-node-agent-token";
pub const PORT_ARTIFACT_TRANSFER_HEADER: &str = "x-port-artifact-transfer";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedClientHeaders {
    pub auth_header: String,
    pub auth_value: String,
    pub audience_value: String,
}

impl HostedClientHeaders {
    #[must_use]
    pub fn new(
        auth_header: impl Into<String>,
        auth_value: impl Into<String>,
        audience_value: impl Into<String>,
    ) -> Self {
        Self {
            auth_header: auth_header.into(),
            auth_value: auth_value.into(),
            audience_value: audience_value.into(),
        }
    }

    #[must_use]
    pub fn from_identity(contract: &HostedApiIdentityContract, token: impl Into<String>) -> Self {
        let token = token.into();
        let auth_value = match contract.auth.scheme {
            HostedAuthScheme::Bearer => format!("Bearer {token}"),
        };
        Self::new(
            contract.auth.header.clone(),
            auth_value,
            contract.audience.clone(),
        )
    }

    #[must_use]
    pub fn to_header_map(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (self.auth_header.clone(), self.auth_value.clone()),
            (
                String::from(PORT_AUDIENCE_HEADER),
                self.audience_value.clone(),
            ),
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeAgentHeaders {
    pub token: String,
}

impl HostedNodeAgentHeaders {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    #[must_use]
    pub fn to_header_map(&self) -> BTreeMap<String, String> {
        BTreeMap::from([(
            String::from(PORT_NODE_AGENT_TOKEN_HEADER),
            self.token.clone(),
        )])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedArtifactTransferRequest {
    pub artifact_name: String,
    pub reference: ArtifactReference,
    pub selector: ArtifactSelector,
    pub filename: String,
    pub store_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedArtifactTransferResult {
    pub artifact_name: String,
    pub reference: ArtifactReference,
    pub selector: ArtifactSelector,
    pub store_path: PathBuf,
    pub bytes_copied: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedGuestVerb {
    Exec,
    Copy,
    Pty,
    Logs,
    Forward,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedMachineRoute {
    List,
    Launch { machine_name: String },
    Status { machine_name: String },
    Monitor { machine_name: String },
    Top { machine_name: String },
    Stop { machine_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedGuestRoute {
    pub machine_name: String,
    pub verb: HostedGuestVerb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedGuestStreamRoute {
    pub machine_name: String,
    pub verb: HostedGuestVerb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedDetachedForwardRoute {
    Start {
        machine_name: String,
    },
    List {
        machine_name: String,
    },
    Stop {
        machine_name: String,
        forward_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedDetachedForwardStartRequest {
    pub listen: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedDetachedForwardState {
    Running,
    Stale,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedDetachedForwardStatus {
    pub name: String,
    pub state: HostedDetachedForwardState,
    pub pid: Option<u32>,
    pub listen: String,
    pub target: String,
    pub manifest_path: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedDetachedForwardStopResult {
    pub name: String,
    pub state: HostedDetachedForwardState,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedGuestStreamProtocol {
    PortAgentStreamV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedServiceRoute {
    SecretPut {
        machine_name: String,
        secret_name: String,
    },
    SecretList {
        machine_name: String,
    },
    SecretRemove {
        machine_name: String,
        secret_name: String,
    },
    Apply {
        machine_name: String,
    },
    List {
        machine_name: String,
    },
    Status {
        machine_name: String,
        service_name: String,
    },
    Stop {
        machine_name: String,
        service_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedRegistrationRoute {
    Refresh { node_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedPreparationRoute {
    PreparePvm { node_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedArtifactRoute {
    Push,
    Pull,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedControlPlaneRoute {
    Artifact(HostedArtifactRoute),
    Machine(HostedMachineRoute),
    Guest(HostedGuestRoute),
    GuestStream(HostedGuestStreamRoute),
    DetachedForward(HostedDetachedForwardRoute),
    Service(HostedServiceRoute),
    Registration(HostedRegistrationRoute),
    Preparation(HostedPreparationRoute),
}

impl HostedControlPlaneRoute {
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::Artifact(route) => artifact_route_path(*route),
            Self::Machine(route) => machine_route_path(route),
            Self::Guest(route) => guest_route_path(route),
            Self::GuestStream(route) => guest_stream_route_path(route),
            Self::DetachedForward(route) => detached_forward_route_path(route),
            Self::Service(route) => service_route_path(route),
            Self::Registration(route) => registration_route_path(route),
            Self::Preparation(route) => preparation_route_path(route),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedNodeRoute {
    Machine(HostedMachineRoute),
    Guest(HostedGuestRoute),
    GuestStream(HostedGuestStreamRoute),
    DetachedForward(HostedDetachedForwardRoute),
    Service(HostedServiceRoute),
}

impl HostedNodeRoute {
    #[must_use]
    pub fn path(&self) -> String {
        let suffix = match self {
            Self::Machine(route) => machine_node_route_suffix(route),
            Self::Guest(route) => guest_node_route_suffix(route),
            Self::GuestStream(route) => guest_stream_node_route_suffix(route),
            Self::DetachedForward(route) => detached_forward_node_route_suffix(route),
            Self::Service(route) => service_node_route_suffix(route),
        };
        format!("/v1/node{suffix}")
    }
}

fn artifact_route_path(route: HostedArtifactRoute) -> String {
    match route {
        HostedArtifactRoute::Push => String::from("/v1/artifacts:push"),
        HostedArtifactRoute::Pull => String::from("/v1/artifacts:pull"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HostedRouteContext {
    pub control_plane: Option<String>,
    pub machine_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    pub node_name: Option<String>,
    pub candidate_nodes: Vec<String>,
    pub host_groups: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rejected_nodes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_detail: Option<String>,
    pub runtime_root: Option<PathBuf>,
    pub inventory_owner: Option<MachineInventoryOwner>,
    pub lifecycle_owner: Option<MachineLifecycleOwner>,
    pub guest_broker: Option<MachineGuestBroker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_session: Option<HostedGuestSessionContract>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedGuestSessionScope {
    Machine,
}

impl std::fmt::Display for HostedGuestSessionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Machine => f.write_str("machine"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedShellDriverContract {
    pub id: String,
    pub route: MachineCommandRoute,
    pub broker: MachineGuestBroker,
    pub protocol: HostedGuestProtocolContract,
    pub command_surface: Vec<GuestCommandVerb>,
}

impl HostedShellDriverContract {
    #[must_use]
    pub fn canonical(route: MachineCommandRoute, broker: MachineGuestBroker) -> Self {
        Self {
            id: String::from("port-guest-shell-driver-v1"),
            route,
            broker,
            protocol: HostedGuestProtocolContract::PortAgentProtocol,
            command_surface: vec![
                GuestCommandVerb::Exec,
                GuestCommandVerb::Copy,
                GuestCommandVerb::Pty,
                GuestCommandVerb::Logs,
                GuestCommandVerb::Forward,
            ],
        }
    }

    #[must_use]
    pub fn from_guest_attach(contract: &HostedGuestAttachContract) -> Self {
        Self {
            route: contract.guest_route,
            broker: contract.guest_broker,
            protocol: contract.protocol,
            command_surface: contract.command_surface.clone(),
            ..Self::canonical(contract.guest_route, contract.guest_broker)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedGuestSessionContract {
    pub id: String,
    pub scope: HostedGuestSessionScope,
    pub driver: HostedShellDriverContract,
}

impl HostedGuestSessionContract {
    #[must_use]
    pub fn machine_scoped(
        control_plane: impl Into<String>,
        machine_name: impl Into<String>,
        driver: HostedShellDriverContract,
    ) -> Self {
        Self {
            id: format!(
                "port-hosted://{}/machines/{}/guest-session",
                control_plane.into(),
                machine_name.into()
            ),
            scope: HostedGuestSessionScope::Machine,
            driver,
        }
    }

    #[must_use]
    pub fn from_guest_attach(contract: &HostedGuestAttachContract) -> Self {
        Self::machine_scoped(
            contract.machine.control_plane.clone(),
            contract.machine.machine_name.clone(),
            HostedShellDriverContract::from_guest_attach(contract),
        )
    }
}

impl HostedRouteContext {
    #[must_use]
    pub fn from_machine_summary(summary: &HostedMachineSummaryContract) -> Self {
        Self {
            control_plane: Some(summary.control_plane.clone()),
            machine_name: Some(summary.machine_name.clone()),
            forward_name: None,
            service_name: None,
            node_name: None,
            candidate_nodes: summary.candidate_nodes.clone(),
            host_groups: summary.host_groups.clone(),
            host_group_policies: summary.host_group_policies.clone(),
            rejected_nodes: summary.rejected_nodes.clone(),
            placement_detail: (!summary.placement_detail.trim().is_empty())
                .then(|| summary.placement_detail.clone()),
            runtime_root: None,
            inventory_owner: Some(summary.control.inventory_owner),
            lifecycle_owner: Some(summary.control.lifecycle_owner),
            guest_broker: Some(summary.control.guest_broker),
            guest_session: None,
        }
    }

    #[must_use]
    pub fn from_guest_attach(contract: &HostedGuestAttachContract) -> Self {
        let context = Self::from_machine_summary(&contract.machine);
        context.with_guest_session_contract(contract)
    }

    #[must_use]
    pub fn with_guest_session_contract(mut self, contract: &HostedGuestAttachContract) -> Self {
        self.guest_broker = Some(contract.guest_broker);
        self.guest_session = Some(HostedGuestSessionContract::from_guest_attach(contract));
        self
    }

    #[must_use]
    pub fn with_selected_node(
        mut self,
        node_name: impl Into<String>,
        runtime_root: impl Into<PathBuf>,
    ) -> Self {
        self.node_name = Some(node_name.into());
        self.runtime_root = Some(runtime_root.into());
        self
    }

    #[must_use]
    pub fn with_forward_name(mut self, forward_name: impl Into<String>) -> Self {
        self.forward_name = Some(forward_name.into());
        self
    }

    #[must_use]
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedSuccess<T> {
    pub route: HostedRouteContext,
    pub result: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedError {
    pub route: Option<HostedRouteContext>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeRegistrationRequest {
    pub control_plane: String,
    pub node_name: String,
    pub registration: HostedNodeRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedPreparePvmNodeRequest {
    pub control_plane: String,
    pub node_name: String,
    pub architecture: MachineArchitecture,
    pub provenance: String,
    pub package: PvmHostKitPackage,
}

fn machine_route_path(route: &HostedMachineRoute) -> String {
    match route {
        HostedMachineRoute::List => String::from("/v1/machines"),
        HostedMachineRoute::Launch { machine_name } => {
            format!("/v1/machines/{machine_name}:launch")
        }
        HostedMachineRoute::Status { machine_name } => {
            format!("/v1/machines/{machine_name}")
        }
        HostedMachineRoute::Monitor { machine_name } => {
            format!("/v1/machines/{machine_name}/monitor")
        }
        HostedMachineRoute::Top { machine_name } => {
            format!("/v1/machines/{machine_name}/top")
        }
        HostedMachineRoute::Stop { machine_name } => {
            format!("/v1/machines/{machine_name}:stop")
        }
    }
}

fn guest_route_path(route: &HostedGuestRoute) -> String {
    format!(
        "/v1/machines/{}/guest:{}",
        route.machine_name,
        route.verb.as_str()
    )
}

fn guest_stream_route_path(route: &HostedGuestStreamRoute) -> String {
    format!(
        "/v1/machines/{}/guest:{}:stream",
        route.machine_name,
        route.verb.as_str()
    )
}

fn detached_forward_route_path(route: &HostedDetachedForwardRoute) -> String {
    match route {
        HostedDetachedForwardRoute::Start { machine_name }
        | HostedDetachedForwardRoute::List { machine_name } => {
            format!("/v1/machines/{machine_name}/guest:forward:detached")
        }
        HostedDetachedForwardRoute::Stop {
            machine_name,
            forward_name,
        } => {
            format!("/v1/machines/{machine_name}/guest:forward:detached/{forward_name}/stop")
        }
    }
}

fn service_route_path(route: &HostedServiceRoute) -> String {
    match route {
        HostedServiceRoute::SecretPut {
            machine_name,
            secret_name,
        } => format!("/v1/machines/{machine_name}/secrets/{secret_name}"),
        HostedServiceRoute::SecretList { machine_name } => {
            format!("/v1/machines/{machine_name}/secrets")
        }
        HostedServiceRoute::SecretRemove {
            machine_name,
            secret_name,
        } => format!("/v1/machines/{machine_name}/secrets/{secret_name}"),
        HostedServiceRoute::Apply { machine_name } => {
            format!("/v1/machines/{machine_name}/services")
        }
        HostedServiceRoute::List { machine_name } => {
            format!("/v1/machines/{machine_name}/services")
        }
        HostedServiceRoute::Status {
            machine_name,
            service_name,
        } => format!("/v1/machines/{machine_name}/services/{service_name}"),
        HostedServiceRoute::Stop {
            machine_name,
            service_name,
        } => format!("/v1/machines/{machine_name}/services/{service_name}:stop"),
    }
}

fn registration_route_path(route: &HostedRegistrationRoute) -> String {
    match route {
        HostedRegistrationRoute::Refresh { node_name } => {
            format!("/v1/nodes/{node_name}/registration")
        }
    }
}

fn preparation_route_path(route: &HostedPreparationRoute) -> String {
    match route {
        HostedPreparationRoute::PreparePvm { node_name } => {
            format!("/v1/nodes/{node_name}/prepare-pvm")
        }
    }
}

fn machine_node_route_suffix(route: &HostedMachineRoute) -> String {
    match route {
        HostedMachineRoute::List => String::from("/machines"),
        HostedMachineRoute::Launch { machine_name } => {
            format!("/machines/{machine_name}:launch")
        }
        HostedMachineRoute::Status { machine_name } => {
            format!("/machines/{machine_name}")
        }
        HostedMachineRoute::Monitor { machine_name } => {
            format!("/machines/{machine_name}/monitor")
        }
        HostedMachineRoute::Top { machine_name } => {
            format!("/machines/{machine_name}/top")
        }
        HostedMachineRoute::Stop { machine_name } => {
            format!("/machines/{machine_name}:stop")
        }
    }
}

fn guest_node_route_suffix(route: &HostedGuestRoute) -> String {
    format!(
        "/machines/{}/guest:{}",
        route.machine_name,
        route.verb.as_str()
    )
}

fn guest_stream_node_route_suffix(route: &HostedGuestStreamRoute) -> String {
    format!(
        "/machines/{}/guest:{}:stream",
        route.machine_name,
        route.verb.as_str()
    )
}

fn detached_forward_node_route_suffix(route: &HostedDetachedForwardRoute) -> String {
    match route {
        HostedDetachedForwardRoute::Start { machine_name }
        | HostedDetachedForwardRoute::List { machine_name } => {
            format!("/machines/{machine_name}/guest:forward:detached")
        }
        HostedDetachedForwardRoute::Stop {
            machine_name,
            forward_name,
        } => {
            format!("/machines/{machine_name}/guest:forward:detached/{forward_name}/stop")
        }
    }
}

fn service_node_route_suffix(route: &HostedServiceRoute) -> String {
    match route {
        HostedServiceRoute::SecretPut {
            machine_name,
            secret_name,
        } => format!("/machines/{machine_name}/secrets/{secret_name}"),
        HostedServiceRoute::SecretList { machine_name } => {
            format!("/machines/{machine_name}/secrets")
        }
        HostedServiceRoute::SecretRemove {
            machine_name,
            secret_name,
        } => format!("/machines/{machine_name}/secrets/{secret_name}"),
        HostedServiceRoute::Apply { machine_name } => {
            format!("/machines/{machine_name}/services")
        }
        HostedServiceRoute::List { machine_name } => {
            format!("/machines/{machine_name}/services")
        }
        HostedServiceRoute::Status {
            machine_name,
            service_name,
        } => format!("/machines/{machine_name}/services/{service_name}"),
        HostedServiceRoute::Stop {
            machine_name,
            service_name,
        } => format!("/machines/{machine_name}/services/{service_name}:stop"),
    }
}

impl HostedGuestVerb {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::Copy => "copy",
            Self::Pty => "pty",
            Self::Logs => "logs",
            Self::Forward => "forward",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::to_value;

    use port_model::{
        ArtifactReference, ArtifactSelector, ExecutionSubstrate, HostProvider,
        HostedGuestProtocolContract, HostedImportedNodeRecord, HostedNodeRegistration,
        HostedPvmHostKitPackageAttachment, HostedSchedulerPolicy, MachineArchitecture,
        MachineCommandRoute, MachineGuestBroker, MachineInventoryOwner, MachineLifecycleOwner,
        PortConfig, ProtectionMode, PvmHostKitPackage,
    };

    use super::{
        HostedArtifactRoute, HostedArtifactTransferRequest, HostedArtifactTransferResult,
        HostedClientHeaders, HostedControlPlaneRoute, HostedDetachedForwardRoute, HostedGuestRoute,
        HostedGuestSessionContract, HostedGuestSessionScope, HostedGuestStreamProtocol,
        HostedGuestStreamRoute, HostedGuestVerb, HostedMachineRoute, HostedNodeAgentHeaders,
        HostedNodeRegistrationRequest, HostedNodeRoute, HostedPreparationRoute,
        HostedPreparePvmNodeRequest, HostedRegistrationRoute, HostedRouteContext,
        HostedServiceRoute, HostedShellDriverContract, HostedSuccess,
        PORT_ARTIFACT_TRANSFER_HEADER, PORT_AUDIENCE_HEADER, PORT_NODE_AGENT_TOKEN_HEADER,
    };

    #[test]
    fn hosted_client_headers_follow_identity_contract() {
        let config = PortConfig::sample();
        let identity = config
            .hosted_api_identity_contract("cloud-aws")
            .expect("config should validate")
            .expect("cloud-aws should resolve hosted identity");

        let headers = HostedClientHeaders::from_identity(&identity, "demo-token");
        let map = headers.to_header_map();
        assert_eq!(map["authorization"], "Bearer demo-token");
        assert_eq!(map[PORT_AUDIENCE_HEADER], "port-hosted-demo");
    }

    #[test]
    fn node_agent_headers_use_explicit_internal_header() {
        let headers = HostedNodeAgentHeaders::new("node-demo").to_header_map();
        assert_eq!(headers[PORT_NODE_AGENT_TOKEN_HEADER], "node-demo");
    }

    #[test]
    fn control_plane_routes_render_canonical_paths() {
        assert_eq!(
            HostedControlPlaneRoute::Artifact(HostedArtifactRoute::Push).path(),
            "/v1/artifacts:push"
        );
        assert_eq!(
            HostedControlPlaneRoute::Artifact(HostedArtifactRoute::Pull).path(),
            "/v1/artifacts:pull"
        );
        assert_eq!(
            HostedControlPlaneRoute::Machine(HostedMachineRoute::List).path(),
            "/v1/machines"
        );
        assert_eq!(
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Launch {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/machines/cloud-aws:launch"
        );
        assert_eq!(
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Stop {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/machines/cloud-aws:stop"
        );
        assert_eq!(
            HostedControlPlaneRoute::Guest(HostedGuestRoute {
                machine_name: String::from("cloud-aws"),
                verb: HostedGuestVerb::Forward,
            })
            .path(),
            "/v1/machines/cloud-aws/guest:forward"
        );
        assert_eq!(
            HostedControlPlaneRoute::GuestStream(HostedGuestStreamRoute {
                machine_name: String::from("cloud-aws"),
                verb: HostedGuestVerb::Pty,
            })
            .path(),
            "/v1/machines/cloud-aws/guest:pty:stream"
        );
        assert_eq!(
            HostedControlPlaneRoute::DetachedForward(HostedDetachedForwardRoute::Start {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/machines/cloud-aws/guest:forward:detached"
        );
        assert_eq!(
            HostedControlPlaneRoute::DetachedForward(HostedDetachedForwardRoute::Stop {
                machine_name: String::from("cloud-aws"),
                forward_name: String::from("demo-web"),
            })
            .path(),
            "/v1/machines/cloud-aws/guest:forward:detached/demo-web/stop"
        );
        assert_eq!(
            HostedControlPlaneRoute::Service(HostedServiceRoute::Status {
                machine_name: String::from("cloud-aws"),
                service_name: String::from("buildbox"),
            })
            .path(),
            "/v1/machines/cloud-aws/services/buildbox"
        );
        assert_eq!(
            HostedControlPlaneRoute::Registration(HostedRegistrationRoute::Refresh {
                node_name: String::from("aws-linux-node"),
            })
            .path(),
            "/v1/nodes/aws-linux-node/registration"
        );
        assert_eq!(
            HostedControlPlaneRoute::Preparation(HostedPreparationRoute::PreparePvm {
                node_name: String::from("generic-linux-node"),
            })
            .path(),
            "/v1/nodes/generic-linux-node/prepare-pvm"
        );
    }

    #[test]
    fn hosted_artifact_transfer_contract_round_trips_selector_and_store_path() {
        let request = HostedArtifactTransferRequest {
            artifact_name: String::from("demo-kernel"),
            reference: ArtifactReference {
                registry: String::from("demo-fs"),
                repository: String::from("port/demo-kernel"),
                version: String::from("v1"),
            },
            selector: ArtifactSelector {
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
            filename: String::from("vmlinux"),
            store_path: PathBuf::from(
                ".port/hosted/demo/artifacts/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux",
            ),
        };
        let result = HostedArtifactTransferResult {
            artifact_name: request.artifact_name.clone(),
            reference: request.reference.clone(),
            selector: request.selector,
            store_path: request.store_path.clone(),
            bytes_copied: 17,
        };

        let request_value = to_value(&request).expect("request should serialize");
        let result_value = to_value(&result).expect("result should serialize");

        assert_eq!(request_value["artifact_name"], "demo-kernel");
        assert_eq!(request_value["selector"]["architecture"], "x86_64");
        assert_eq!(
            request_value["store_path"],
            ".port/hosted/demo/artifacts/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux"
        );
        assert_eq!(result_value["bytes_copied"], 17);
    }

    #[test]
    fn hosted_artifact_transfer_header_name_is_stable() {
        assert_eq!(PORT_ARTIFACT_TRANSFER_HEADER, "x-port-artifact-transfer");
    }

    #[test]
    fn node_routes_render_internal_paths() {
        assert_eq!(
            HostedNodeRoute::Machine(HostedMachineRoute::Launch {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/node/machines/cloud-aws:launch"
        );
        assert_eq!(
            HostedNodeRoute::Machine(HostedMachineRoute::Monitor {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/node/machines/cloud-aws/monitor"
        );
        assert_eq!(
            HostedNodeRoute::Guest(HostedGuestRoute {
                machine_name: String::from("cloud-aws"),
                verb: HostedGuestVerb::Exec,
            })
            .path(),
            "/v1/node/machines/cloud-aws/guest:exec"
        );
        assert_eq!(
            HostedNodeRoute::GuestStream(HostedGuestStreamRoute {
                machine_name: String::from("cloud-aws"),
                verb: HostedGuestVerb::Logs,
            })
            .path(),
            "/v1/node/machines/cloud-aws/guest:logs:stream"
        );
        assert_eq!(
            HostedNodeRoute::DetachedForward(HostedDetachedForwardRoute::List {
                machine_name: String::from("cloud-aws"),
            })
            .path(),
            "/v1/node/machines/cloud-aws/guest:forward:detached"
        );
    }

    #[test]
    fn hosted_guest_stream_protocol_serializes_stably() {
        let encoded = serde_json::to_string(&HostedGuestStreamProtocol::PortAgentStreamV1)
            .expect("stream protocol should encode");
        assert_eq!(encoded, "\"port-agent-stream-v1\"");
    }

    #[test]
    fn route_context_preserves_inventory_and_guest_broker_context() {
        let config = PortConfig::sample();
        let summary = config
            .hosted_machine_summary_contract("cloud-aws")
            .expect("summary should resolve")
            .expect("cloud-aws should be hosted");
        let guest_attach = config
            .hosted_guest_attach_contract("cloud-aws")
            .expect("guest attach should resolve")
            .expect("guest attach contract should exist");

        let summary_context = HostedRouteContext::from_machine_summary(&summary);
        assert_eq!(summary_context.control_plane.as_deref(), Some("demo"));
        assert_eq!(summary_context.machine_name.as_deref(), Some("cloud-aws"));
        assert_eq!(
            summary_context.inventory_owner,
            Some(MachineInventoryOwner::HostedControlPlane)
        );
        assert_eq!(
            summary_context.lifecycle_owner,
            Some(MachineLifecycleOwner::HostedNodeAgent)
        );
        assert_eq!(
            summary_context.guest_broker,
            Some(MachineGuestBroker::ControlPlaneNodeAgentTunnel)
        );
        assert!(summary_context.guest_session.is_none());
        assert_eq!(
            summary_context.host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );

        let selected = HostedRouteContext::from_guest_attach(&guest_attach).with_selected_node(
            "aws-linux-node",
            PathBuf::from("runtime/hosted/aws-linux-node"),
        );
        assert_eq!(selected.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            selected.runtime_root,
            Some(PathBuf::from("runtime/hosted/aws-linux-node"))
        );
        let guest_session = selected
            .guest_session
            .as_ref()
            .expect("guest session contract should exist");
        assert_eq!(
            guest_session.id,
            "port-hosted://demo/machines/cloud-aws/guest-session"
        );
        assert_eq!(guest_session.scope, crate::HostedGuestSessionScope::Machine);
        assert_eq!(guest_session.driver.id, "port-guest-shell-driver-v1");
        assert_eq!(
            guest_session.driver.route,
            port_model::MachineCommandRoute::HostedControlPlane
        );
        assert_eq!(
            guest_session.driver.broker,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel
        );
        assert_eq!(
            guest_session.driver.protocol,
            port_model::HostedGuestProtocolContract::PortAgentProtocol
        );
        assert_eq!(
            guest_session
                .driver
                .command_surface
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["exec", "copy", "pty", "logs", "forward"]
        );
        let forwarded = selected.with_forward_name("demo-web");
        assert_eq!(forwarded.forward_name.as_deref(), Some("demo-web"));
    }

    #[test]
    fn canonical_shell_driver_contract_uses_port_guest_surface() {
        let driver = HostedShellDriverContract::canonical(
            MachineCommandRoute::HostedControlPlane,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel,
        );
        let session =
            HostedGuestSessionContract::machine_scoped("demo", "cloud-aws", driver.clone());

        assert_eq!(driver.id, "port-guest-shell-driver-v1");
        assert_eq!(driver.route, MachineCommandRoute::HostedControlPlane);
        assert_eq!(
            driver.broker,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel
        );
        assert_eq!(
            driver.protocol,
            HostedGuestProtocolContract::PortAgentProtocol
        );
        assert_eq!(
            driver
                .command_surface
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["exec", "copy", "pty", "logs", "forward"]
        );
        assert_eq!(
            session.id,
            "port-hosted://demo/machines/cloud-aws/guest-session"
        );
        assert_eq!(session.scope, HostedGuestSessionScope::Machine);
        assert_eq!(session.driver, driver);
    }

    #[test]
    fn hosted_success_serializes_inventory_with_pvm_capability_states() {
        let config = PortConfig::sample();
        let inventory = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        let body = to_value(HostedSuccess {
            route: HostedRouteContext::default(),
            result: inventory,
        })
        .expect("inventory success payload should serialize");

        assert_eq!(
            body["result"]["nodes"]["aws-linux-node"]["capabilities"]["pvm_lanes"][0]["state"],
            "planned"
        );
    }

    #[test]
    fn hosted_registration_request_and_contract_serialize_stably() {
        let request = HostedNodeRegistrationRequest {
            control_plane: String::from("demo"),
            node_name: String::from("aws-linux-node"),
            registration: HostedNodeRegistration {
                endpoint: String::from("http://127.0.0.1:9001"),
                token: String::from("node-demo-token"),
                registered_at: 10,
                refreshed_at: 25,
                ttl_seconds: 30,
            },
        };
        let body = to_value(&request).expect("registration request should serialize");
        assert_eq!(body["control_plane"], "demo");
        assert_eq!(body["node_name"], "aws-linux-node");
        assert_eq!(body["registration"]["endpoint"], "http://127.0.0.1:9001");
        assert_eq!(body["registration"]["token"], "node-demo-token");
        assert_eq!(body["registration"]["ttl_seconds"], 30);

        let config = PortConfig::sample();
        let registered = config
            .hosted_inventory_contract()
            .expect("hosted inventory should resolve")
            .hosted_registered_node_contract("demo", "aws-linux-node", &request.registration)
            .expect("registered node contract should resolve");
        let success = to_value(HostedSuccess {
            route: HostedRouteContext::default(),
            result: registered,
        })
        .expect("registered node success should serialize");
        assert_eq!(success["result"]["node_name"], "aws-linux-node");
        assert_eq!(success["result"]["endpoint"], "http://127.0.0.1:9001");
        assert_eq!(success["result"]["freshness"]["fresh_until"], 55);
        assert_eq!(success["result"]["host_groups"][0], "aws-builders");
    }

    #[test]
    fn hosted_prepare_pvm_request_and_import_record_serialize_stably() {
        let request = HostedPreparePvmNodeRequest {
            control_plane: String::from("demo"),
            node_name: String::from("generic-linux-node"),
            architecture: MachineArchitecture::X86_64,
            provenance: String::from("inventory-sync"),
            package: PvmHostKitPackage {
                name: String::from("firecracker-pvm-host-kit"),
                version: String::from("2026.04"),
                host_kernel_release: String::from("6.12.0-port-pvm"),
                firecracker_build: String::from("v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c"),
            },
        };
        let body = to_value(&request).expect("prepare request should serialize");
        assert_eq!(body["control_plane"], "demo");
        assert_eq!(body["node_name"], "generic-linux-node");
        assert_eq!(body["architecture"], "x86_64");
        assert_eq!(body["package"]["name"], "firecracker-pvm-host-kit");

        let record = HostedImportedNodeRecord {
            provider: HostProvider::GenericLinux,
            provenance: request.provenance.clone(),
            imported_at: 42,
            capability_summary: PortConfig::sample().nodes["generic-linux-node"]
                .capabilities
                .clone(),
            pvm_host_kit_packages: vec![HostedPvmHostKitPackageAttachment {
                architecture: request.architecture,
                package: request.package.clone(),
            }],
        };
        let success = to_value(HostedSuccess {
            route: HostedRouteContext::default(),
            result: record,
        })
        .expect("prepared record should serialize");
        assert_eq!(
            success["result"]["pvm_host_kit_packages"][0]["package"]["version"],
            "2026.04"
        );
    }

    #[test]
    fn route_context_serializes_hosted_pvm_rejection_detail() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-generic")
            .expect("cloud-generic should exist")
            .protection_mode = port_model::ProtectionMode::Pvm;
        let summary = config
            .hosted_machine_summary_contract("cloud-generic")
            .expect("summary should resolve")
            .expect("cloud-generic should be hosted");

        let body = to_value(HostedRouteContext::from_machine_summary(&summary))
            .expect("route context should serialize");

        let rejection = body["rejected_nodes"]["generic-linux-node"]
            .as_str()
            .expect("rejection detail should exist");
        assert!(rejection.contains("planned"), "{rejection}");
        assert!(
            rejection.contains("without a provider-backed host-kit contract"),
            "{rejection}"
        );
        assert!(
            body["placement_detail"]
                .as_str()
                .expect("placement detail should exist")
                .contains("planned")
        );
        assert_eq!(
            body["host_group_policies"]["remote-linux"],
            "deterministic-first-fit"
        );
    }

    #[test]
    fn route_context_serializes_detached_forward_identity() {
        let body = to_value(
            HostedRouteContext::default()
                .with_selected_node("aws-linux-node", "runtime/hosted/aws-linux-node")
                .with_forward_name("demo-web"),
        )
        .expect("route context should serialize");

        assert_eq!(body["node_name"], "aws-linux-node");
        assert_eq!(body["forward_name"], "demo-web");
    }

    #[test]
    fn route_context_serializes_service_identity() {
        let body = to_value(
            HostedRouteContext::default()
                .with_selected_node("aws-linux-node", "runtime/hosted/aws-linux-node")
                .with_service_name("buildbox"),
        )
        .expect("route context should serialize");

        assert_eq!(body["node_name"], "aws-linux-node");
        assert_eq!(body["service_name"], "buildbox");
    }
}
