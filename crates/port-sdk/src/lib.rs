use std::collections::BTreeMap;
use std::env;

use anyhow::{Context, Result};
use port_agent_protocol::{
    CopyRequest, ExecRequest, ForwardRequest, GuestOperation, LogsRequest, PtyRequest,
};
use port_hosted_protocol::{
    HostedArtifactRoute, HostedArtifactTransferRequest, HostedClientHeaders,
    HostedControlPlaneRoute, HostedDetachedForwardRoute, HostedDetachedForwardStartRequest,
    HostedError, HostedGuestRoute, HostedGuestStreamProtocol, HostedGuestStreamRoute,
    HostedGuestVerb, HostedMachineRoute, HostedPreparationRoute, HostedPreparePvmNodeRequest,
    HostedRouteContext, HostedServiceRoute, PORT_ARTIFACT_TRANSFER_HEADER,
};
use port_model::{
    HostedApiIdentityContract, HostedAuthTokenSource, MachineCommandRoute, PortConfig,
};
use reqwest::blocking::Client as BlockingClient;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostedApiStreamRequest {
    pub request: HostedApiRequest,
    pub protocol: HostedGuestStreamProtocol,
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

    pub fn from_machine_env(config: &PortConfig, machine_name: &str) -> Result<Self> {
        let contract = config
            .hosted_api_identity_contract(machine_name)?
            .with_context(|| {
                format!(
                    "machine '{}' does not target a hosted control plane",
                    machine_name
                )
            })?;
        let token = token_from_source(&contract.auth.source)?;
        Ok(Self::from_identity(contract, token))
    }

    pub fn from_control_plane_env(config: &PortConfig, control_plane_name: &str) -> Result<Self> {
        let control_plane = config
            .control_planes
            .get(control_plane_name)
            .with_context(|| format!("unknown control plane '{}'", control_plane_name))?;
        let token = token_from_source(&control_plane.auth.source)?;
        Ok(Self::from_identity(
            HostedApiIdentityContract {
                control_plane: control_plane_name.to_string(),
                endpoint: control_plane.endpoint.clone(),
                audience: control_plane.audience.clone(),
                auth: control_plane.auth.clone(),
                route: MachineCommandRoute::HostedControlPlane,
            },
            token,
        ))
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
    pub fn artifacts(&self) -> ArtifactClient<'_> {
        ArtifactClient { client: self }
    }

    #[must_use]
    pub fn services(&self) -> ServiceClient<'_> {
        ServiceClient { client: self }
    }

    #[must_use]
    pub fn inventory(&self) -> InventoryClient<'_> {
        InventoryClient { client: self }
    }

    pub fn execute_json<T>(&self, request: HostedApiRequest) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let client = BlockingClient::builder()
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

        let response = builder.send().with_context(|| {
            format!(
                "failed to send hosted request {} {}",
                request.method, request.url
            )
        })?;
        let status = response.status();
        if status.is_success() {
            return response.json::<T>().with_context(|| {
                format!(
                    "failed to decode hosted response body for {} {}",
                    request.method, request.url
                )
            });
        }

        let fallback = status
            .canonical_reason()
            .unwrap_or("unknown error")
            .to_string();
        match response.json::<HostedError>() {
            Ok(error) => {
                anyhow::bail!(
                    "hosted request {} {} failed with {}: {}{}",
                    request.method,
                    request.url,
                    status,
                    error.message,
                    render_route_context(error.route.as_ref()),
                );
            }
            Err(_) => {
                anyhow::bail!(
                    "hosted request {} {} failed with {}: {}",
                    request.method,
                    request.url,
                    status,
                    fallback,
                );
            }
        }
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

fn token_from_source(source: &HostedAuthTokenSource) -> Result<String> {
    match source {
        HostedAuthTokenSource::Env { variable } => env::var(variable).with_context(|| {
            format!(
                "hosted auth token is missing from environment variable '{}'",
                variable
            )
        }),
    }
}

fn render_route_context(route: Option<&HostedRouteContext>) -> String {
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
    if let Some(runtime_root) = &route.runtime_root {
        parts.push(format!("runtime-root={}", runtime_root.display()));
    }
    if let Some(placement_detail) = &route.placement_detail {
        parts.push(format!("placement={placement_detail}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Service,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSecretBinding {
    pub env: String,
    pub secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceApplyRequest {
    pub name: String,
    pub kind: ServiceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_group: Option<String>,
    pub command: Vec<String>,
    pub secret_bindings: Vec<ServiceSecretBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretPutRequest {
    pub name: String,
    pub value: String,
}

pub struct MachineClient<'a> {
    client: &'a HostedClient,
}

pub struct InventoryClient<'a> {
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
    pub fn launch(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Machine(HostedMachineRoute::Launch {
                machine_name: machine_name.to_string(),
            }),
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

impl<'a> InventoryClient<'a> {
    #[must_use]
    pub fn prepare_pvm_node(&self, request: HostedPreparePvmNodeRequest) -> HostedApiRequest {
        let node_name = request.node_name.clone();
        self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Preparation(HostedPreparationRoute::PreparePvm { node_name }),
            Some(
                serde_json::to_value(request)
                    .expect("hosted pvm node prepare request should serialize"),
            ),
        )
    }
}

pub struct GuestClient<'a> {
    client: &'a HostedClient,
}

pub struct ArtifactClient<'a> {
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

    pub fn pty_stream(
        &self,
        machine_name: &str,
        request: PtyRequest,
    ) -> Result<HostedApiStreamRequest> {
        self.stream(
            machine_name,
            HostedGuestVerb::Pty,
            GuestOperation::Pty(request),
        )
    }

    pub fn logs(&self, machine_name: &str, request: LogsRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "logs", GuestOperation::Logs(request))
    }

    pub fn logs_stream(
        &self,
        machine_name: &str,
        request: LogsRequest,
    ) -> Result<HostedApiStreamRequest> {
        self.stream(
            machine_name,
            HostedGuestVerb::Logs,
            GuestOperation::Logs(request),
        )
    }

    pub fn forward(&self, machine_name: &str, request: ForwardRequest) -> Result<HostedApiRequest> {
        self.operation(machine_name, "forward", GuestOperation::Forward(request))
    }

    pub fn copy_stream(
        &self,
        machine_name: &str,
        request: CopyRequest,
    ) -> Result<HostedApiStreamRequest> {
        self.stream(
            machine_name,
            HostedGuestVerb::Copy,
            GuestOperation::Copy(request),
        )
    }

    pub fn forward_stream(
        &self,
        machine_name: &str,
        request: ForwardRequest,
    ) -> Result<HostedApiStreamRequest> {
        self.stream(
            machine_name,
            HostedGuestVerb::Forward,
            GuestOperation::Forward(request),
        )
    }

    pub fn forward_detached_start(
        &self,
        machine_name: &str,
        request: HostedDetachedForwardStartRequest,
    ) -> Result<HostedApiRequest> {
        let body =
            serde_json::to_value(&request).context("failed to encode detached forward request")?;
        Ok(self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::DetachedForward(HostedDetachedForwardRoute::Start {
                machine_name: machine_name.to_string(),
            }),
            Some(body),
        ))
    }

    #[must_use]
    pub fn forward_detached_list(&self, machine_name: &str) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Get,
            HostedControlPlaneRoute::DetachedForward(HostedDetachedForwardRoute::List {
                machine_name: machine_name.to_string(),
            }),
            None,
        )
    }

    #[must_use]
    pub fn forward_detached_stop(
        &self,
        machine_name: &str,
        forward_name: &str,
    ) -> HostedApiRequest {
        self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::DetachedForward(HostedDetachedForwardRoute::Stop {
                machine_name: machine_name.to_string(),
                forward_name: forward_name.to_string(),
            }),
            None,
        )
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

    fn stream(
        &self,
        machine_name: &str,
        verb: HostedGuestVerb,
        operation: GuestOperation,
    ) -> Result<HostedApiStreamRequest> {
        let body = serde_json::to_value(operation).context("failed to encode guest operation")?;
        Ok(HostedApiStreamRequest {
            request: self.client.request(
                HttpMethod::Post,
                HostedControlPlaneRoute::GuestStream(HostedGuestStreamRoute {
                    machine_name: machine_name.to_string(),
                    verb,
                }),
                Some(body),
            ),
            protocol: HostedGuestStreamProtocol::PortAgentStreamV1,
        })
    }
}

impl<'a> ArtifactClient<'a> {
    pub fn push(&self, request: HostedArtifactTransferRequest) -> Result<HostedApiStreamRequest> {
        let mut headers = self.client.auth_headers.to_header_map();
        headers.insert(
            String::from(PORT_ARTIFACT_TRANSFER_HEADER),
            serde_json::to_string(&request)
                .context("failed to encode hosted artifact transfer request header")?,
        );
        Ok(HostedApiStreamRequest {
            request: HostedApiRequest {
                method: HttpMethod::Post,
                url: format!(
                    "{}/{}",
                    self.client.base_url,
                    HostedControlPlaneRoute::Artifact(HostedArtifactRoute::Push)
                        .path()
                        .trim_start_matches('/')
                ),
                headers,
                body: None,
            },
            protocol: HostedGuestStreamProtocol::PortAgentStreamV1,
        })
    }

    pub fn pull(&self, request: HostedArtifactTransferRequest) -> Result<HostedApiRequest> {
        Ok(self.client.request(
            HttpMethod::Post,
            HostedControlPlaneRoute::Artifact(HostedArtifactRoute::Pull),
            Some(
                serde_json::to_value(request)
                    .context("failed to encode hosted artifact transfer request body")?,
            ),
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
    use std::net::{TcpListener as StdTcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::{
        HostedApiStreamRequest, HostedClient, HttpMethod, SecretPutRequest, ServiceApplyRequest,
        ServiceClient, ServiceKind, ServiceSecretBinding,
    };
    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode};
    use axum::routing::get;
    use axum::{Json, Router};
    use port_agent_protocol::{CopyDirection, CopyRequest, ExecRequest, LogsRequest, PtyRequest};
    use port_hosted_protocol::{
        HostedDetachedForwardStartRequest, HostedError, HostedGuestStreamProtocol,
        HostedRouteContext, HostedSuccess, PORT_AUDIENCE_HEADER,
    };
    use serde_json::json;

    fn serve_router(router: Router) -> String {
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
                    tokio::net::TcpListener::from_std(listener).expect("listener should convert");
                let _ = axum::serve(listener, router).await;
            });
        });

        for _ in 0..100 {
            if TcpStream::connect(addr).is_ok() {
                return format!("http://{addr}");
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for test server at '{addr}'");
    }

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
                    host_group: Some(String::from("aws-builders")),
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
        assert_eq!(body["host_group"], "aws-builders");
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
            client.machines().launch("cloud-aws").url,
            "https://port.example.internal/v1/machines/cloud-aws:launch"
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

    #[test]
    fn guest_stream_requests_follow_stream_paths_and_protocol() {
        let client = HostedClient::new(
            "https://port.example.internal",
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let request: HostedApiStreamRequest = client
            .guest()
            .pty_stream(
                "cloud-aws",
                PtyRequest {
                    command: vec![String::from("/bin/sh")],
                    cols: 80,
                    rows: 24,
                },
            )
            .expect("stream request should encode");
        assert_eq!(
            request.request.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:pty:stream"
        );
        assert_eq!(
            request.protocol,
            HostedGuestStreamProtocol::PortAgentStreamV1
        );

        let logs = client
            .guest()
            .logs_stream(
                "cloud-aws",
                LogsRequest {
                    path: String::from("/var/log/app.log"),
                    follow: true,
                    tail_lines: Some(50),
                },
            )
            .expect("logs stream request should encode");
        assert_eq!(
            logs.request.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:logs:stream"
        );
    }

    #[test]
    fn detached_forward_requests_use_canonical_hosted_paths() {
        let client = HostedClient::new(
            "https://port.example.internal",
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let start = client
            .guest()
            .forward_detached_start(
                "cloud-aws",
                HostedDetachedForwardStartRequest {
                    listen: String::from("127.0.0.1:8081"),
                    target: String::from("127.0.0.1:80"),
                    name: Some(String::from("demo-web")),
                },
            )
            .expect("detached start request should encode");
        assert_eq!(start.method, HttpMethod::Post);
        assert_eq!(
            start.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:forward:detached"
        );
        let body = start.body.expect("body should exist");
        assert_eq!(body["name"], "demo-web");
        assert_eq!(body["listen"], "127.0.0.1:8081");

        let list = client.guest().forward_detached_list("cloud-aws");
        assert_eq!(list.method, HttpMethod::Get);
        assert_eq!(
            list.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:forward:detached"
        );

        let stop = client
            .guest()
            .forward_detached_stop("cloud-aws", "demo-web");
        assert_eq!(stop.method, HttpMethod::Post);
        assert_eq!(
            stop.url,
            "https://port.example.internal/v1/machines/cloud-aws/guest:forward:detached/demo-web/stop"
        );
        assert!(stop.body.is_none());
    }

    #[test]
    fn hosted_client_executes_live_machine_requests() {
        #[derive(Clone)]
        struct AppState {
            headers: Arc<Mutex<Vec<String>>>,
        }

        async fn status_handler(
            State(state): State<AppState>,
            headers: HeaderMap,
        ) -> Json<HostedSuccess<serde_json::Value>> {
            state.headers.lock().expect("headers lock").push(
                headers
                    .get("authorization")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
            );
            Json(HostedSuccess {
                route: HostedRouteContext {
                    control_plane: Some(String::from("demo")),
                    machine_name: Some(String::from("cloud-aws")),
                    ..HostedRouteContext::default()
                },
                result: json!({
                    "machine_name": "cloud-aws",
                    "state": "stopped",
                }),
            })
        }

        let state = AppState {
            headers: Arc::new(Mutex::new(Vec::new())),
        };
        let observed = state.headers.clone();
        let endpoint = serve_router(
            Router::new()
                .route("/v1/machines/cloud-aws", get(status_handler))
                .with_state(state),
        );

        let client = HostedClient::new(
            endpoint,
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let response: HostedSuccess<serde_json::Value> = client
            .execute_json(client.machines().status("cloud-aws"))
            .expect("request should succeed");

        assert_eq!(response.result["machine_name"], "cloud-aws");
        assert_eq!(
            observed.lock().expect("headers lock").as_slice(),
            &[String::from("Bearer demo-token")]
        );
    }

    #[test]
    fn hosted_client_surfaces_route_context_from_live_errors() {
        async fn error_handler() -> (StatusCode, Json<HostedError>) {
            (
                StatusCode::BAD_GATEWAY,
                Json(HostedError {
                    route: Some(HostedRouteContext {
                        control_plane: Some(String::from("demo")),
                        machine_name: Some(String::from("cloud-aws")),
                        node_name: Some(String::from("aws-linux-node")),
                        ..HostedRouteContext::default()
                    }),
                    message: String::from("node agent is unavailable"),
                }),
            )
        }

        let endpoint =
            serve_router(Router::new().route("/v1/machines/cloud-aws", get(error_handler)));

        let client = HostedClient::new(
            endpoint,
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let error = client
            .execute_json::<HostedSuccess<serde_json::Value>>(client.machines().status("cloud-aws"))
            .expect_err("request should fail");
        let message = error.to_string();
        assert!(message.contains("node agent is unavailable"));
        assert!(message.contains("control-plane=demo"));
        assert!(message.contains("machine=cloud-aws"));
        assert!(message.contains("node=aws-linux-node"));
    }

    #[test]
    fn hosted_client_surfaces_placement_detail_from_live_errors() {
        async fn error_handler() -> (StatusCode, Json<HostedError>) {
            (
                StatusCode::BAD_GATEWAY,
                Json(HostedError {
                    route: Some(HostedRouteContext {
                        control_plane: Some(String::from("demo")),
                        machine_name: Some(String::from("cloud-generic")),
                        rejected_nodes: BTreeMap::from([(
                            String::from("generic-linux-node"),
                            String::from("pvm-ready state is required but node advertises planned"),
                        )]),
                        placement_detail: Some(String::from(
                            "machine 'cloud-generic' requires PVM on x86_64 via firecracker; no hosted nodes satisfy that requirement; rejected nodes: generic-linux-node (pvm-ready state is required but node advertises planned)",
                        )),
                        ..HostedRouteContext::default()
                    }),
                    message: String::from("machine is not placeable"),
                }),
            )
        }

        let endpoint =
            serve_router(Router::new().route("/v1/machines/cloud-generic", get(error_handler)));

        let client = HostedClient::new(
            endpoint,
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let error = client
            .execute_json::<HostedSuccess<serde_json::Value>>(
                client.machines().status("cloud-generic"),
            )
            .expect_err("request should fail");
        let message = error.to_string();
        assert!(message.contains("machine is not placeable"));
        assert!(message.contains("machine=cloud-generic"));
        assert!(message.contains("placement=machine 'cloud-generic' requires PVM"));
        assert!(message.contains("planned"));
    }

    #[test]
    fn hosted_client_surfaces_service_identity_from_live_errors() {
        async fn error_handler() -> (StatusCode, Json<HostedError>) {
            (
                StatusCode::BAD_GATEWAY,
                Json(HostedError {
                    route: Some(HostedRouteContext {
                        control_plane: Some(String::from("demo")),
                        machine_name: Some(String::from("cloud-aws")),
                        service_name: Some(String::from("buildbox")),
                        node_name: Some(String::from("aws-linux-node")),
                        ..HostedRouteContext::default()
                    }),
                    message: String::from("service runtime state is unavailable"),
                }),
            )
        }

        let endpoint = serve_router(Router::new().route(
            "/v1/machines/cloud-aws/services/buildbox",
            get(error_handler),
        ));

        let client = HostedClient::new(
            endpoint,
            "port-hosted-demo",
            "authorization",
            "Bearer demo-token",
        );
        let error = client
            .execute_json::<HostedSuccess<serde_json::Value>>(
                client.services().status("cloud-aws", "buildbox"),
            )
            .expect_err("request should fail");
        let message = error.to_string();
        assert!(message.contains("service runtime state is unavailable"));
        assert!(message.contains("machine=cloud-aws"));
        assert!(message.contains("service=buildbox"));
        assert!(message.contains("node=aws-linux-node"));
    }
}
