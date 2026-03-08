# Hosted SDK And API Clients

Port now publishes a supported typed client surface in the `port-sdk` crate for
hosted machine, guest, and service operations.

The crate now covers both typed request construction and live JSON response
execution against the hosted control-plane transport that Port ships for the
single-node demo lane.

## Scope

Shipped today:

- `HostedClient::from_machine` derives hosted endpoint, audience, and auth
  header shape from the shared Port model plus `port-hosted-protocol`
- `HostedClient::from_machine_env` and `HostedClient::from_control_plane_env`
  derive the auth token source from the model and read the configured
  environment variable automatically
- `machines()` mirrors `port machine list|status|monitor|top|stop`
- `guest()` mirrors `port guest exec|copy|pty|logs|forward` using the existing
  `port-agent-protocol` request payloads
- `services()` mirrors `port service secret put|list|remove` plus
  `port service apply|list|status|stop`
- `HostedClient::execute_json` performs the live HTTP request and decodes
  structured success or hosted route errors
- `port-hosted-protocol` publishes the shared hosted HTTP route, auth-header,
  and route-context contract that the SDK now uses directly

Still planned:

- retries and richer client policies on top of the shipped transport
- generated or versioned external API packages beyond the in-repo Rust crate
- advanced auth, RBAC, and multi-tenant concerns
- streamed hosted file transfer and fully remote hosted forward lifecycle work

## Example

```rust
use port_agent_protocol::ExecRequest;
use port_model::PortConfig;
use port_sdk::{HostedClient, ServiceApplyRequest, ServiceKind, ServiceSecretBinding};

let config = PortConfig::sample();
let client = HostedClient::from_machine(&config, "cloud-aws", "demo-token")?;

let status = client.machines().status("cloud-aws");
let exec = client.guest().exec(
    "cloud-aws",
    ExecRequest {
        command: vec!["/bin/echo".into(), "hello".into()],
        cwd: None,
        env: Default::default(),
    },
)?;
let service = client.services().apply(
    "cloud-aws",
    ServiceApplyRequest {
        name: "buildbox".into(),
        kind: ServiceKind::Sandbox,
        command: vec!["/bin/sh".into(), "-lc".into(), "make test".into()],
        secret_bindings: vec![ServiceSecretBinding {
            env: "API_TOKEN".into(),
            secret: "demo-token".into(),
        }],
    },
)?;

assert_eq!(status.url, "https://port.example.internal/v1/machines/cloud-aws");
assert_eq!(exec.url, "https://port.example.internal/v1/machines/cloud-aws/guest:exec");
assert_eq!(service.url, "https://port.example.internal/v1/machines/cloud-aws/services");
# Ok::<(), anyhow::Error>(())
```

If a hosted control plane is running and the token source is configured in the
environment, the same client can execute the request directly:

```rust
# use port_hosted_protocol::HostedSuccess;
# use port_sdk::HostedClient;
# use port_model::PortConfig;
let config = PortConfig::sample();
let client = HostedClient::from_machine_env(&config, "cloud-aws")?;
let status: HostedSuccess<serde_json::Value> =
    client.execute_json(client.machines().status("cloud-aws"))?;
# Ok::<(), anyhow::Error>(())
```

Run the in-repo example with:

```bash
cargo run -p port-sdk --example hosted-sdk
```

## API Shape

Canonical hosted request paths now documented by Port:

- `GET /v1/machines`
- `GET /v1/machines/{machine}`
- `GET /v1/machines/{machine}/monitor`
- `GET /v1/machines/{machine}/top`
- `POST /v1/machines/{machine}:stop`
- `POST /v1/machines/{machine}/guest:exec|copy|pty|logs|forward`
- `PUT /v1/machines/{machine}/secrets/{secret}`
- `GET /v1/machines/{machine}/secrets`
- `DELETE /v1/machines/{machine}/secrets/{secret}`
- `POST /v1/machines/{machine}/services`
- `GET /v1/machines/{machine}/services`
- `GET /v1/machines/{machine}/services/{name}`
- `POST /v1/machines/{machine}/services/{name}:stop`

The SDK mirrors those paths exactly so later transport work can build on a
stable typed surface instead of inventing a second client model. The route and
header definitions themselves live in `crates/port-hosted-protocol`.
