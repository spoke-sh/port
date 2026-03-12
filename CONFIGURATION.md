# Configuration Guide

Port uses one shared model for artifacts, hosts, nodes, control planes,
machines, and services.

## Config Selection

Port has two config modes:

1. `port` with no `--config` uses the built-in sample model from the product.
2. `port --config <path>` loads and validates the TOML file at that path.

For repo-local workflows, start from
[`examples/port.toml`](examples/port.toml).

## Top-Level Sections

| Section | Purpose |
|---------|---------|
| `[artifacts.*]` | Logical artifact references, distribution backends, and variant selectors |
| `[control_planes.*]` | Hosted endpoint, audience, and auth contract |
| `[hosts.*]` | Host platform/provider capabilities and local substrate contracts |
| `[nodes.*]` | Hosted node-agent inventory, capabilities, and runtime roots |
| `[machines.*]` | Operator-visible machine definitions and lane selection |

## Key Concepts

### Artifacts

Artifacts are selected by:

- `architecture`
- `substrate`
- `protection_mode`

Each artifact binds:

- one logical reference
- one or more concrete variants
- a `push` backend
- a `pull` backend
- a local cache root

### Control Planes

Hosted lanes use an explicit control-plane entry:

- `endpoint`
- `audience`
- auth source, typically `PORT_DEMO_TOKEN`

### Hosts And Nodes

- `hosts.*` describe platform/provider capabilities and local boundaries.
- `nodes.*` describe hosted node-agent identity, runtime roots, and supported
  substrates or protection modes.

### Machines

Each machine picks:

- a host or hosted control plane
- an artifact selector
- a substrate/protection combination
- any lane-specific routing detail

### Attached Volumes

The first storage slice adds one optional attached volume per machine:

- one persistent `host-file` backend
- one explicit host path
- one ownership contract surfaced through `port doctor`, `machine launch`,
  `machine status`, and `machine stop`
- support only on the local Firecracker `standard` lane in this slice

Example:

```toml
[[machines.demo.volumes]]
name = "data"
backend = "host-file"
persistence = "persistent"
path = "volumes/demo-data.ext4"
```

Canonical direct-runtime workflow:

```bash
port --config /tmp/port-attached-volume.toml doctor
port --config /tmp/port-attached-volume.toml machine launch --machine demo
port --config /tmp/port-attached-volume.toml machine status --machine demo
port --config /tmp/port-attached-volume.toml machine stop --machine demo
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `PORT_DEMO_TOKEN` | Sample hosted bearer token source |
| `PORT_AVF_LAUNCHER` | AVF launcher helper path on macOS |
| `PORT_PVM_FIRECRACKER_BINARY` | Prepared-node Firecracker/PVM binary |
| `PORT_OCI_USER` / `PORT_OCI_PASSWORD` | OCI basic auth variables when the selected backend requires them |
| `PORT_OCI_DEMO_CONTAINER_RUNTIME` | Repo-local demo helper override for the OCI registry proof |

## Detailed Examples

### 1. Local Linux Firecracker

Use the checked-in sample config directly:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
port --config examples/port.toml artifacts build --artifact demo-guest --architecture native
port --config examples/port.toml machine launch --machine demo
port --config examples/port.toml machine list
port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'cat /proc/version'
port --config examples/port.toml machine status --machine demo
port --config examples/port.toml machine stop --machine demo
```

### 2. Hosted Standard Control Plane

The sample file already includes `[control_planes.demo]` and hosted node or
machine definitions. The minimal hosted demo path is:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine stop --machine cloud-aws
```

Hosted and SSH-owned machines keep the attached-volume boundary explicit:
declaring an attached volume on those lanes fails validation before launch
instead of rerouting the request or collapsing it back into the rootfs story.

### 3. Cloud Hypervisor Override

Start from a copy of `examples/port.toml` and make these changes:

- point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`
- set `machines.cloud-aws.substrate = "cloud-hypervisor"`
- set `machines.cloud-aws.architecture = "x86_64"`
- set `nodes.aws-linux-node.capabilities.substrates = ["cloud-hypervisor"]`

Then run:

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml guest exec --machine cloud-aws -- /bin/sh -lc 'uname -a'
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml machine stop --machine cloud-aws
```

### 4. Prepared-Node Firecracker/PVM

Start from a copy of `examples/port.toml` and make these changes:

- point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`
- set `machines.cloud-generic.protection_mode = "pvm"`
- point the `x86_64/firecracker/pvm` kernel and guest-image variants at the
  prepared artifact paths for the target node
- export `PORT_PVM_FIRECRACKER_BINARY` to the prepared PVM binary

Then run:

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_PVM_FIRECRACKER_BINARY=/path/to/firecracker-pvm PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml node-agent serve --node generic-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane prepare-pvm-node --control-plane demo --node generic-linux-node --architecture x86-64 --provenance repo-proof --package-name firecracker-pvm-host-kit --package-version 2026.03 --host-kernel-release 6.12.0-port-pvm --firecracker-build v1.12.0-port-pvm
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine launch --machine cloud-generic
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine stop --machine cloud-generic
```

### 5. Service Secrets, Restart, And Health

The checked-in sample already models hosted service nodes and groups. A typical
service workflow is:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service secret put --machine cloud-aws --name demo-token --value s3cr3t
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service apply --machine cloud-aws --host-group aws-secondary --name api --kind service --restart on-failure --health command --health-command /bin/test --health-command=-f --health-command workspace/healthy --secret API_TOKEN=demo-token -- /bin/sh -lc 'trap '\''exit 0'\'' TERM; while :; do sleep 1; done'
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service status --machine cloud-aws --name api
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service stop --machine cloud-aws --name api
```

### 6. Artifact Backend Switching

The sample config defaults to file-backed distribution:

```toml
[artifacts.kernels.demo-kernel.distribution.push]
backend = "file-system"
root = "artifact-store/demo-fs"
```

Hosted artifact routing uses:

```toml
[artifacts.kernels.demo-kernel.distribution.push]
backend = "hosted-api"
endpoint = "http://127.0.0.1:7040"
```

Repo-local OCI proof uses:

```toml
[artifacts.kernels.demo-kernel.distribution.push]
backend = "oci-registry"
transport = "plain-http"

[artifacts.kernels.demo-kernel.distribution.push.auth]
kind = "anonymous"
```

The same selector flags remain canonical across all backends:

- `--architecture`
- `--substrate`
- `--protection-mode`

## Related Guides

- [`docs/artifacts.md`](docs/artifacts.md)
- [`docs/hosted.md`](docs/hosted.md)
- [`docs/cloud.md`](docs/cloud.md)
- [`docs/pvm.md`](docs/pvm.md)
- [`docs/avf.md`](docs/avf.md)
