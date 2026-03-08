use std::collections::BTreeMap;
use std::path::PathBuf;

use port_model::{
    HostedApiIdentityContract, HostedAuthScheme, HostedGuestAttachContract,
    HostedMachineSummaryContract, MachineGuestBroker, MachineInventoryOwner, MachineLifecycleOwner,
};
use serde::{Deserialize, Serialize};

pub const PORT_AUDIENCE_HEADER: &str = "x-port-audience";
pub const PORT_NODE_AGENT_TOKEN_HEADER: &str = "x-port-node-agent-token";

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
pub enum HostedControlPlaneRoute {
    Machine(HostedMachineRoute),
    Guest(HostedGuestRoute),
    Service(HostedServiceRoute),
}

impl HostedControlPlaneRoute {
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::Machine(route) => machine_route_path(route),
            Self::Guest(route) => guest_route_path(route),
            Self::Service(route) => service_route_path(route),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedNodeRoute {
    Machine(HostedMachineRoute),
    Guest(HostedGuestRoute),
    Service(HostedServiceRoute),
}

impl HostedNodeRoute {
    #[must_use]
    pub fn path(&self) -> String {
        let suffix = match self {
            Self::Machine(route) => machine_node_route_suffix(route),
            Self::Guest(route) => guest_node_route_suffix(route),
            Self::Service(route) => service_node_route_suffix(route),
        };
        format!("/v1/node{suffix}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HostedRouteContext {
    pub control_plane: Option<String>,
    pub machine_name: Option<String>,
    pub node_name: Option<String>,
    pub candidate_nodes: Vec<String>,
    pub host_groups: Vec<String>,
    pub runtime_root: Option<PathBuf>,
    pub inventory_owner: Option<MachineInventoryOwner>,
    pub lifecycle_owner: Option<MachineLifecycleOwner>,
    pub guest_broker: Option<MachineGuestBroker>,
}

impl HostedRouteContext {
    #[must_use]
    pub fn from_machine_summary(summary: &HostedMachineSummaryContract) -> Self {
        Self {
            control_plane: Some(summary.control_plane.clone()),
            machine_name: Some(summary.machine_name.clone()),
            node_name: None,
            candidate_nodes: summary.candidate_nodes.clone(),
            host_groups: summary.host_groups.clone(),
            runtime_root: None,
            inventory_owner: Some(summary.control.inventory_owner),
            lifecycle_owner: Some(summary.control.lifecycle_owner),
            guest_broker: Some(summary.control.guest_broker),
        }
    }

    #[must_use]
    pub fn from_guest_attach(contract: &HostedGuestAttachContract) -> Self {
        let mut context = Self::from_machine_summary(&contract.machine);
        context.guest_broker = Some(contract.guest_broker);
        context
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

fn machine_route_path(route: &HostedMachineRoute) -> String {
    match route {
        HostedMachineRoute::List => String::from("/v1/machines"),
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

fn machine_node_route_suffix(route: &HostedMachineRoute) -> String {
    match route {
        HostedMachineRoute::List => String::from("/machines"),
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
        MachineGuestBroker, MachineInventoryOwner, MachineLifecycleOwner, PortConfig,
    };

    use super::{
        HostedClientHeaders, HostedControlPlaneRoute, HostedGuestRoute, HostedGuestVerb,
        HostedMachineRoute, HostedNodeAgentHeaders, HostedNodeRoute, HostedRouteContext,
        HostedServiceRoute, HostedSuccess, PORT_AUDIENCE_HEADER, PORT_NODE_AGENT_TOKEN_HEADER,
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
            HostedControlPlaneRoute::Machine(HostedMachineRoute::List).path(),
            "/v1/machines"
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
            HostedControlPlaneRoute::Service(HostedServiceRoute::Status {
                machine_name: String::from("cloud-aws"),
                service_name: String::from("buildbox"),
            })
            .path(),
            "/v1/machines/cloud-aws/services/buildbox"
        );
    }

    #[test]
    fn node_routes_render_internal_paths() {
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

        let selected = HostedRouteContext::from_guest_attach(&guest_attach).with_selected_node(
            "aws-linux-node",
            PathBuf::from("runtime/hosted/aws-linux-node"),
        );
        assert_eq!(selected.node_name.as_deref(), Some("aws-linux-node"));
        assert_eq!(
            selected.runtime_root,
            Some(PathBuf::from("runtime/hosted/aws-linux-node"))
        );
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
            "ready"
        );
    }
}
