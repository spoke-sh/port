use std::collections::BTreeMap;

use anyhow::{Context, Result};
use port_agent_protocol::{
    CopyRequest, ExecRequest, ForwardRequest, GuestOperation, LogsRequest, PtyRequest,
};
use port_hosted_protocol::{
    HostedClientHeaders, HostedControlPlaneRoute, HostedGuestRoute, HostedGuestVerb,
    HostedMachineRoute, HostedServiceRoute,
};
use port_model::{HostedApiIdentityContract, PortConfig};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => f.write_str("GET"),
            Self::Post => f.write_str("POST"),
            Self::Put => f.write_str("PUT"),
            Self::Delete => f.write_str("DELETE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostedApiRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedClient {
    base_url: String,
    auth_headers: HostedClientHeaders,
}

impl HostedClient {
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        audience: impl Into<String>,
        auth_header: impl Into<String>,
        auth_value: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth_headers: HostedClientHeaders::new(auth_header, auth_value, audience),
        }
    }

    pub fn from_machine(
        config: &PortConfig,
        machine_name: &str,
        token: impl Into<String>,
    ) -> Result<Self> {
        let contract = config
            .hosted_api_identity_contract(machine_name)?
            .with_context(|| {
                format!(
                    "machine '{}' does not target a hosted control plane",
                    machine_name
                )
            })?;
        Ok(Self::from_identity(contract, token.into()))
    }

    #[must_use]
    pub fn machines(&self) -> MachineClient<'_> {
        MachineClient { client: self }
    }

    #[must_use]
    pub fn guest(&self) -> GuestClient<'_> {
        GuestClient { client: self }
    }

    #[must_use]
    pub fn services(&self) -> ServiceClient<'_> {
        ServiceClient { client: self }
    }

    fn from_identity(contract: HostedApiIdentityContract, token: String) -> Self {
        let headers = HostedClientHeaders::from_identity(&contract, token);
        Self::new(
            contract.endpoint,
            contract.audience,
            contract.auth.header,
            headers.auth_value,
        )
    }

    fn request(
        &self,
        method: HttpMethod,
        route: HostedControlPlaneRoute,
        body: Option<Value>,
    ) -> HostedApiRequest {
        let mut headers = self.auth_headers.to_header_map();
        if body.is_some() {
            headers.insert(
                String::from("content-type"),
                String::from("application/json"),
            );
        }
        HostedApiRequest {
            method,
            url: format!("{}/{}", self.base_url, route.path().trim_start_matches('/')),
            headers,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Service,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceSecretBinding {
    pub env: String,
    pub secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceApplyRequest {
    pub name: String,
    pub kind: ServiceKind,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecretPutRequest {
    pub name: String,
    pub value: String,
}

pub struct MachineClient<'a> {
    client: &'a HostedClient,
}

impl<'a> MachineClient<'a> {
    #[must_use]
    pub fn list(&self) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::List),
            None,
        )
    }

    #[must_use]
    pub fn status(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Status {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn monitor(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Monitor {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn top(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Top {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn stop(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Stop {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }
}

pub struct GuestClient<'a> {
    client: &'a HostedClient,
}

impl<'a> GuestClient<'a> {
    pub fn exec(&self, machine_name: &str, request: ExecRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "exec", GuestOperation::Exec(request))
    }

    pub fn copy(&self, machine_name: &str, request: CopyRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "copy", GuestOperation::Copy(request))
    }

    pub fn pty(&self, machine_name: &str, request: PtyRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "pty", GuestOperation::Pty(request))
    }

    pub fn logs(&self, machine_name: &str, request: LogsRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "logs", GuestOperation::Logs(request))
    }

    pub fn forward(&self, machine_name: &str, request: ForwardRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "forward", GuestOperation::Forward(request))
    }

    fn operation(
        &self,
        machine_name: &str,
        verb: &str,
        operation: GuestOperation,
    ) -> Result<HostedApiRequest> {
        let body = serde_json::to_value(operation).context("failed to encode guest operation")?;
        Ok(self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Guest(HostedGuestRoute {
                machine_name: machine_name.to_string(),
                verb: match verb {
                    "exec" => HostedGuestVerb::Exec,
                    "copy" => HostedGuestVerb::Copy,
                    "pty" => HostedGuestVerb::Pty,
                    "logs" => HostedGuestVerb::Logs,
                    "forward" => HostedGuestVerb::Forward,
                    _ => unreachable!("unsupported guest route verb"),
                },
            }),
            Some(body),
        ))
    }
}

pub struct ServiceClient<'a> {
    client: &'a HostedClient,
}

impl<'a> ServiceClient<'a> {
    pub fn secret_put(
        &self,
        machine_name: &str,
        request: SecretPutRequest,
    ) -> Result<HostedApiRequest> {
        let body = serde_json::to_value(&request).context("failed to encode secret request")?;
        Ok(self.client.request(
            HttpMethod::Put,
            HostedControlPlaneRoute::Service(HostedServiceRoute::SecretPut {
                machine_name: machine_name.to_string(),
                secret_name: request.name.clone(),
            }),
            Some(body),
        ))
    }

    #[must_use]
    pub fn secret_list(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Service(HostedServiceRoute::SecretList {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn secret_remove(&self, machine_name: &str, secret_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Delete,
            HostedControlPlaneRoute::Service(HostedServiceRoute::SecretRemove {
                machine_name: machine_name.to_string(),
                secret_name: secret_name.to_string(),
            }),
            None,
        )
    }

    pub fn apply(
        &self,
        machine_name: &str,
        request: ServiceApplyRequest,
    ) -> Result<HostedApiRequest> {
        let body = serde_json::to_value(request).context("failed to encode service request")?;
        Ok(self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Service(HostedServiceRoute::Apply {
                machine_name: machine_name.to_string(),
            }),
            Some(body),
        ))
    }

    #[must_use]
    pub fn list(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Service(HostedServiceRoute::List {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn status(&self, machine_name: &str, service_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::Service(HostedServiceRoute::Status {
                machine_name: machine_name.to_string(),
                service_name: service_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn stop(&self, machine_name: &str, service_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Service(HostedServiceRoute::Stop {
                machine_name: machine_name.to_string(),
                service_name: service_name.to_string(),
            }),
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use port_agent_protocol::{CopyDirection, CopyRequest, ExecRequest};
    use port_hosted_protocol::PORT_AUDIENCE_HEADER;

    use super::{
        HostedClient, HttpMethod, SecretPutRequest, ServiceApplyRequest, ServiceClient,
        ServiceKind, ServiceSecretBinding,
    };

    #[test]
    fn client_from_machine_uses_hosted_identity_contract() {
        let client = HostedClient::from_machine(
            &port_model::PortConfig::sample(),
            "cloud-aws",
            "demo-token",
        )
        .expect("hosted client should resolve");

        let request = client.machines().list();
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://port.example.internal/v1/machines");
        assert_eq!(request.headers["authorization"], "Bearer demo-token");
        assert_eq!(request.headers[PORT_AUDIENCE_HEADER], "port-hosted-demo");
    }

    #[test]
    fn guest_requests_preserve_existing_guest_payloads() {
        let client = HostedClient::new(
            "https://port.example.internal",
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let request = client
            .guest()
            .copy(
                "cloud-aws",
                CopyRequest {
                    source: String::from("/tmp/src"),
                    destination: String::from("/tmp/dst"),
                    direction: CopyDirection::GuestToHost,
                    size_bytes: None,
                },
            )
            .expect("copy request should encode");

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(
            request.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:copy"
        );
        assert_eq!(request.body.expect("body should exist")["type"], "copy");
    }

    #[test]
    fn service_requests_cover_secret_and_sandbox_surfaces() {
        let client = HostedClient::new(
            "https://port.example.internal",
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let services = ServiceClient { client: &client };

        let secret_put = services
            .secret_put(
                "cloud-aws",
                SecretPutRequest {
                    name: String::from("demo-token"),
                    value: String::from("s3cr3t"),
                },
            )
            .expect("secret request should encode");
        assert_eq!(secret_put.method, HttpMethod::Put);
        assert_eq!(
            secret_put.url,
            "https://port.example.internal/v1/machines/cloud-aws/secrets/demo-token"
        );

        let apply = services
            .apply(
                "cloud-aws",
                ServiceApplyRequest {
                    name: String::from("buildbox"),
                    kind: ServiceKind::Sandbox,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("make test"),
                    ],
                    secret_bindings: vec![ServiceSecretBinding {
                        env: String::from("API_TOKEN"),
                        secret: String::from("demo-token"),
                    }],
                },
            )
            .expect("service request should encode");
        assert_eq!(apply.method, HttpMethod::Post);
        assert_eq!(
            apply.url,
            "https://port.example.internal/v1/machines/cloud-aws/services"
        );
        let body = apply.body.expect("body should exist");
        assert_eq!(body["kind"], "sandbox");
        assert_eq!(body["secret_bindings"][0]["secret"], "demo-token");

        let status = services.status("cloud-aws", "buildbox");
        assert_eq!(status.method, HttpMethod::Get);
    }

    #[test]
    fn machine_and_guest_paths_follow_canonical_operator_model() {
        let client = HostedClient::new(
            "https://port.example.internal",
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let exec = client
            .guest()
            .exec(
                "cloud-aws",
                ExecRequest {
                    command: vec![String::from("/bin/echo"), String::from("hi")],
                    cwd: None,
                    env: BTreeMap::new(),
                },
            )
            .expect("exec request should encode");
        assert_eq!(
            exec.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:exec"
        );
        assert_eq!(
            client.machines().monitor("cloud-aws").method,
            HttpMethod::Get
        );
        assert_eq!(
            client.machines().top("cloud-aws").url,
            "https://port.example.internal/v1/machines/cloud-aws/top"
        );
        assert_eq!(
            client.services().stop("cloud-aws", "api").url,
            "https://port.example.internal/v1/machines/cloud-aws/services/api:stop"
        );
    }
}
