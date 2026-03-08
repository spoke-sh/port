# Hosted Control Contract

Port's hosted product is intentionally defined as an extension of the existing
local command model, not as a separate product with different lifecycle or
guest-operation semantics.

What is shipped today:

- local Linux launch through `port machine launch`
- local runtime-root inspection through `port machine list`, `status`, and
  `stop`
- guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical
  guest protocol

What is planned:

- a long-lived node agent that owns hypervisor processes on each execution host
- a central control plane that owns inventory, desired state, placement, and
  policy
- a client/API path so the same `port` verbs can target local or hosted
  environments without changing their core meaning

## Role Split

### CLI / Client

The `port` CLI remains the canonical operator surface.

- In local mode, it can still launch or inspect directly against the local
  runtime root.
- In hosted mode, it becomes a client of the control plane instead of owning
  VM processes itself.
- The command verbs stay stable: `machine launch`, `list`, `status`, `stop`,
  and guest `exec`, `copy`, `pty`, `logs`, and `forward`.

### Node Agent

The node agent is the host-local runtime owner on each execution host.

- launches and stops VMs on the local substrate
- owns host-local runtime state, manifests, pid/process inspection, and socket
  paths
- realizes artifacts onto the host before launch
- brokers guest-operation tunnels from the control plane down to the guest
  agent
- reports node and machine health back to the control plane

The node agent is the hosted analog of today's local `port-runtime` ownership.
If a hypervisor process exists on a host, the node agent owns it.

### Control Plane

The central control plane is the system of record for hosted Port.

- authenticates clients and enforces policy
- stores machine inventory and desired lifecycle state
- selects nodes or host groups for placement
- asks node agents to launch, inspect, stop, or connect to machines
- surfaces hosted inventory and status back through the CLI and future SDK/API

The control plane does not execute guest commands inside the VM directly. It
coordinates and authorizes them.

## Hosted API Identity Contract

The first hosted auth slice is now explicit in the shared Port model.

Sample config shape:

```toml
[control_planes.demo]
endpoint = "https://port.example.internal"
audience = "port-hosted-demo"

[control_planes.demo.auth]
scheme = "bearer"
header = "authorization"

[control_planes.demo.auth.source]
kind = "env"
variable = "PORT_DEMO_TOKEN"
```

Hosted hosts point at that contract directly:

```toml
[hosts.aws-linux.connection]
mode = "hosted-control-plane"
control_plane = "demo"
```

That means Port now has a canonical way to say:

- which hosted API endpoint owns a machine,
- which audience the CLI is targeting,
- which header carries the token,
- and where the operator provides that token.

This is still a contract, not a claim that the hosted API already runs. The
current implementation uses it for validation, docs, help text, and provider-
aware guidance rather than for real remote execution.

### Guest Agent

The in-guest `port-guest-agent` remains the executor of guest operations.

- `exec` runs commands inside the guest
- `copy` transfers file content across the host/guest boundary
- `pty` owns terminal-backed sessions inside the guest
- `logs` reads guest-visible log paths
- `forward` opens guest-side connections for host or hosted proxying

Hosted Port keeps this guest agent and its protocol semantics intact.

## Lifecycle Ownership

| Environment | Inventory owner | Hypervisor/process owner | Guest-operation broker | Source of truth |
|-------------|-----------------|--------------------------|------------------------|-----------------|
| Local Port today | local runtime root plus CLI output | local `port-runtime` invocation | local CLI plus runtime transport | runtime manifests and host process state |
| Hosted Port planned | central control plane | node agent on the selected host | control plane plus node agent tunnel | control-plane inventory plus node-agent reported runtime state |

The intended rule is simple:

- local mode: the runtime root is the durable lifecycle record
- hosted mode: the control plane owns desired state, while the node agent owns
  the actual host-local process state

## Node And Host-Group Inventory Contract

Hosted Port now has an explicit inventory vocabulary for placement and
ownership instead of leaving "node" and "host group" as prose-only terms.

Sample config shape:

```toml
[nodes.aws-linux-node]
host = "aws-linux"
notes = ["AWS stays explicit because later host-group and PVM planning will care about provider identity."]

[nodes.aws-linux-node.capabilities]
providers = ["aws"]
platforms = ["linux"]
substrates = ["firecracker"]
architectures = ["x86_64"]
protection_modes = ["standard"]

[host_groups.aws-builders]
placement = "explicit-membership"
nodes = ["aws-linux-node"]
notes = ["Provider-specific groups stay explicit so later scheduling and service placement can target them without creating a second host taxonomy."]
```

Node contract:

- resolves through one hosted control plane via the referenced `host`
- keeps `inventory_owner = "hosted-control-plane"`
- keeps `lifecycle_owner = "hosted-node-agent"`
- publishes capability fields for provider, platform, substrate, architecture,
  and protection mode

Host-group contract:

- is an explicit membership list, not a hidden scheduler rule
- stays within one hosted control plane
- becomes the first placement boundary for later lifecycle, scheduler,
  monitoring, and services work

What this does not claim:

- no scheduler policy exists yet beyond explicit membership
- no hosted `machine list` or remote lifecycle implementation ships yet
- no services or monitoring product exists yet

Those later features are expected to reuse the same node and host-group
vocabulary instead of inventing a second inventory model.

## Canonical Machine Control Contract

Port now names the lifecycle and inventory contract explicitly so later hosted
drivers can reuse the same vocabulary that local `machine list`, `status`, and
`stop` already publish.

Local runtime-root contract:

- `inventory_scope = "local-runtime-root"`
- `inventory_owner = "local-runtime-root"`
- `lifecycle_owner = "local-port-runtime"`
- `guest_broker = "local-runtime-transport"`
- `status_source = "runtime-manifest-and-host-process"`
- `launch_route = "direct-local-runtime"`
- `inventory_route = "direct-local-runtime"`
- `status_route = "direct-local-runtime"`
- `stop_route = "direct-local-runtime"`
- `guest_route = "direct-local-runtime"`

Hosted control-plane contract:

- `inventory_scope = "hosted-fleet"`
- `inventory_owner = "hosted-control-plane"`
- `lifecycle_owner = "hosted-node-agent"`
- `guest_broker = "control-plane-node-agent-tunnel"`
- `status_source = "control-plane-inventory-and-node-agent-runtime"`
- `launch_route = "hosted-control-plane"`
- `inventory_route = "hosted-control-plane"`
- `status_route = "hosted-control-plane"`
- `stop_route = "hosted-control-plane"`
- `guest_route = "hosted-control-plane"`

Those tokens are the implementation-ready contract for the next hosted node
agent and control-plane slices. The local lane already reports the local form
through runtime status surfaces, and future hosted drivers are expected to fill
the hosted form rather than inventing a second vocabulary.

## Guest Operation Brokerage

Hosted Port preserves the existing guest protocol by tunneling it, not by
redefining it.

1. The operator runs a canonical CLI command such as `port guest exec`.
2. The client resolves whether the target is local or hosted.
3. For hosted targets, the control plane authorizes the request and resolves
   the owning node.
4. The control plane asks the node agent to open or attach to the machine's
   guest transport.
5. The node agent bridges the existing guest protocol stream to the in-guest
   `port-guest-agent`.
6. Responses stream back through the same path to the CLI.

That means hosted Port still uses the same guest-operation model for `exec`,
`copy`, `pty`, `logs`, and `forward`; the difference is who brokers the byte
stream.

## Hosted API Shape

The exact wire API is not fully implemented yet, but the contract is expected
to expose lifecycle and guest-transport verbs that mirror the CLI:

- `machines.create`
- `machines.list`
- `machines.get`
- `machines.stop`
- `machines.connect_guest`

Those verbs are the hosted counterpart of today's local runtime calls and
should remain substrate-aware without becoming Firecracker-specific API names.

## Current Boundary

- Port does not ship a hosted daemon or control plane yet.
- `port machine list`, `status`, and `stop` currently inspect local runtime
  roots only.
- Those commands already report the local control-contract fields above so the
  operator-visible lifecycle vocabulary does not need to change when hosted
  routing lands.
- Remote Linux providers are modeled and diagnosed, but remote launch remains a
  designed boundary rather than a shipped orchestration path.
- The hosted contract is canonical design work for the next implementation
  slices, not a claim that hosted Port is already available.
