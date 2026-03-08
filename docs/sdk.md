# Hosted SDK And API Clients

Port now publishes a supported typed client surface in the `port-sdk` crate for
hosted machine, guest, and service operations.

This is intentionally a request-building SDK, not a claim that Port already
ships a live remote control plane transport. The crate gives operators and
future SDK consumers one canonical way to construct hosted API requests that
mirror the existing CLI and model.

## Scope

Shipped today:

- `HostedClient::from_machine` derives hosted endpoint, audience, and auth
  header shape from the shared Port model
- `machines()` mirrors `port machine list|status|monitor|top|stop`
- `guest()` mirrors `port guest exec|copy|pty|logs|forward` using the existing
  `port-agent-protocol` request payloads
- `services()` mirrors `port service secret put|list|remove` plus
  `port service apply|list|status|stop`

Still planned:

- a real HTTP transport implementation
- remote response decoding and retries
- generated or versioned external API packages beyond the in-repo Rust crate
- advanced auth, RBAC, and multi-tenant concerns

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
stable typed surface instead of inventing a second client model.
