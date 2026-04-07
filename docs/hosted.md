# Hosted Control Contract

Port's hosted product is intentionally defined as an extension of the existing
local command model, not as a separate product with different lifecycle or
guest-operation semantics.

If you need the strongest production-oriented AWS path, start with
[`aws.md`](aws.md). This file explains the shared hosted control-plane and
node-agent split that both the standard hosted lane and the AWS x86_64 PVM
lane reuse.

What is shipped today:

- local Linux launch through `port machine launch`
- local runtime-root inspection through `port machine list`, `status`, and
  `stop`
- guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical
  guest protocol
- hosted standard Firecracker launch through `port machine launch` for the
  sample `cloud-generic`, `cloud-aws`, and `cloud-gcp` machines when a
  registered hosted node owns the target runtime root and standard artifacts
  are present
- provider-backed AWS `x86_64` Firecracker/PVM launch through
  `port machine launch` for `cloud-aws` when `aws-linux-node` is prepared,
  imported as ready, and backed by real PVM host-kit and artifact-kit inputs
- hosted K3s lifecycle through `port cluster show|up|status|kubeconfig|down`
  for `[k3s_clusters.*]`, where the K3s nodes are hosted Firecracker guest
  microVMs rather than the execution host itself
- `port control-plane serve` as the first live hosted HTTP server for canonical
  machine and guest routes, accepting registered node agents for the demo lane
- `port node-agent serve` as the first live hosted node-runtime server for one
  configured hosted node and runtime root

What is planned:

- a long-lived node agent that owns hypervisor processes on each execution host
- broader hosted rollout beyond the single-node demo lane, including durable
  inventory, placement, and policy
- external API endpoint management and the surrounding load-balancer or VIP
  contract needed for real HA K3s
- a client/API path so the same `port` verbs can target local or hosted
  environments without changing their core meaning

## Role Split

### CLI / Client

The `port` CLI remains the canonical operator surface.

- In local mode, it can still launch or inspect directly against the local
  runtime root.
- In hosted mode, it becomes a client of the control plane instead of owning
  VM processes itself.
- The command verbs stay stable: `machine launch`, `list`, `status`,
  `monitor`, `top`, `stop`, and guest `exec`, `copy`, `pty`, `logs`, and
  `forward`.

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
- asks node agents to launch, inspect, monitor, stop, or connect to machines
- surfaces hosted inventory and status back through the CLI and SDK/API surface

The control plane does not execute guest commands inside the VM directly. It
coordinates and authorizes them.

For the first live demo lane, registered node agents publish their endpoint and
runtime ownership to the configured control plane:

```bash
port --config examples/port.toml control-plane serve \
  --control-plane demo \
  --bind 127.0.0.1:7040

PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve \
  --node aws-linux-node \
  --bind 127.0.0.1:9234 \
  --token node-secret
```

`port control-plane serve --node-binding <node>=<endpoint>,<token>` remains
available only as a bootstrap or debug override when a node cannot
self-register yet. The default operator path is control plane first, then node
registration through `port node-agent serve`.

The repository-local end-to-end demo workflow is:

```bash
export PORT_DEMO_TOKEN=demo-token
bash scripts/hosted-demo.sh
```

That script prepares temporary hosted server and client configs, starts
`port-guest-agent`, `port node-agent serve`, and `port control-plane serve`,
waits for node registration, then runs canonical hosted `port machine list`,
`port machine status`, `port guest exec`, `port guest copy`, and `port guest
logs` commands through the live hosted HTTP path.

## AWS Hosted PVM On The Same Hosted Split

The AWS PVM lane does not introduce a second hosted architecture. It reuses the
same control plane and node agent with tighter requirements:

- machine: `cloud-aws`
- node: `aws-linux-node`
- lane: `x86_64` + `firecracker` + `pvm`
- readiness bridge: `port control-plane prepare-pvm-node`
- artifacts: dedicated `x86_64/firecracker/pvm` kernel and guest-image variants
- failure posture: no fallback to the standard hosted lane

Use [`aws.md`](aws.md) for the operator narrative and [`pvm.md`](pvm.md) for
the low-level host-kit and artifact-kit contract.

Repository-local hosted AWS PVM proof:

```bash
bash scripts/hosted-pvm-demo.sh
```

Human-reviewable artifact:

```bash
./scripts/render-hosted-pvm-proof.sh .keel/stories/VFgcoUoUd/EVIDENCE
```

## Hosted K3s MicroVM Contract

Hosted K3s uses the hosted split directly:

- the control plane owns the named K3s cluster contract from `[k3s_clusters.*]`
- node agents launch and own the guest microVM runtime on each execution host
- the K3s nodes are the guest microVMs named in `server_machines` and
  `worker_machines`

That means the AWS PVM host and the K3s node are different layers:

- AWS PVM host: the execution host that runs the node agent and hypervisor
- control-plane or worker microVM: the guest VM that actually runs K3s

Real HA is stricter than "multiple control-plane guests":

- at least three control-plane microVMs
- a stable HTTPS `api_endpoint` that fronts those control-plane microVMs
- control-plane microVMs placed across distinct execution hosts so one host
  loss does not remove quorum

Port now models the K3s topology and the stable API endpoint in the shared
config and lifecycle reports. Port does not yet ship the external load
balancer, VIP, DNS, or ingress layer that fronts that endpoint, and it does
not ship the infrastructure layer around that endpoint for you. The hosted K3s
contract now does include an explicit control-plane scheduler:
`control_plane_scheduler = "spread"` requires new control-plane microVM
placements to land on distinct eligible execution hosts instead of collapsing
back onto an already used host.

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

The shared hosted HTTP route and auth contract now lives in
`crates/port-hosted-protocol`. It carries:

- the public control-plane request paths
- the internal node-agent request paths
- the explicit auth-header rules for client and node-agent calls
- the route-context envelope that keeps control-plane, node, host-group, and
  runtime-owner context attached to hosted responses and failures

This is still a narrow hosted product slice, not a claim that Port already
ships a broad hosted platform. The current implementation does run real hosted
machine, guest, and service routes in the repo-backed proofs, but broader
policy, tenancy, autoscaling, and fleet management remain follow-on scope.

### Guest Agent

The in-guest `port-guest-agent` remains the executor of guest operations.

- `exec` runs commands inside the guest
- `copy` transfers file content across the host/guest boundary
- `pty` owns terminal-backed sessions inside the guest
- `logs` reads guest-visible log paths
- `forward` opens guest-side TCP or Unix-socket connections for host or hosted
  proxying

Hosted Port keeps this guest agent and its protocol semantics intact.

## Lifecycle Ownership

| Environment | Inventory owner | Hypervisor/process owner | Guest-operation broker | Source of truth |
|-------------|-----------------|--------------------------|------------------------|-----------------|
| Local Port today | local runtime root plus CLI output | local `port-runtime` invocation | local CLI plus runtime transport | runtime manifests and host process state |
| Hosted Port current slice | central control plane | node agent on the selected host | control plane plus node agent tunnel | control-plane inventory plus node-agent reported runtime state |

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
scheduler = "deterministic-first-fit"
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
- carries the first scheduler-policy contract explicitly with
  `scheduler = "deterministic-first-fit"`
- stays within one hosted control plane
- becomes the first placement boundary for later lifecycle, scheduler,
  monitoring, and services work

What this does not claim:

- no scheduler behavior exists yet beyond the deterministic-first-fit contract
- no autoscaling, retry policy, or broader placement product exists beyond the
  shipped explicit-node hosted launch path
- external secret-manager integrations, multi-tenant isolation, and broader
  hosted orchestration remain follow-on work for services and sandboxes
- hosted `machine monitor` and `top` are runtime-inspection surfaces, not a
  full metrics or fleet-observability product yet

Those later features are expected to reuse the same node and host-group
vocabulary instead of inventing a second inventory model.

## Hosted Machine Lifecycle Surface

Port now ships the first hosted lifecycle runtime slice without pretending it
already solves the entire hosted-platform problem.

The canonical operator verbs stay the same:

- `port machine launch --machine <name>`
- `port machine list`
- `port machine status --machine <name>`
- `port machine monitor --machine <name>`
- `port machine top --machine <name>`
- `port machine stop --machine <name>`

For a hosted machine, the shared model now derives five explicit contracts:

- summary: which control plane owns the machine, which hosted nodes can run it,
  and which explicit host groups include those nodes
- launch: the placement route and runtime owner for hosted
  `port machine launch`
- status: the status source and route for hosted `port machine status`
- monitor: the runtime owner plus route for hosted `port machine monitor` and
  `port machine top`
- stop: the lifecycle owner and route for hosted `port machine stop`

Current hosted lifecycle contract for a sample machine such as `cloud-aws`:

- `control_plane = "demo"`
- `candidate_nodes = ["aws-linux-node"]`
- `host_groups = ["remote-linux", "aws-builders"]`
- `runtime_root = "runtime/hosted/aws-linux-node"`
- `status_source = "control-plane-inventory-and-node-agent-runtime"`
- `status_route = "hosted-control-plane"`
- `monitor_route = "hosted-control-plane"`
- `top_route = "hosted-control-plane"`
- `stop_route = "hosted-control-plane"`
- `lifecycle_owner = "hosted-node-agent"`

What this means operationally:

- the control plane remains the routing entry point for list, status, monitor,
  top, and stop
- the node agent remains the eventual owner of host-local lifecycle actions
- the node agent already owns the host-local runtime state that `monitor` and
  `top` inspect, including detached forward manifests
- the CLI verbs and guest protocol do not need a second hosted-only naming
  scheme

What is runnable today:

- hosted `machine launch` now resolves through the live hosted HTTP path:
  CLI/SDK -> `port control-plane serve` -> `port node-agent serve`
- the shipped standard demo lane covers `cloud-generic`, `cloud-aws`, and
  `cloud-gcp` when their matching hosted nodes are registered and standard
  Firecracker artifacts exist on the selected node host
- local `port machine list`, `status`, `monitor`, `top`, and `stop` inspect
  and manage Port-managed runtime directories on Linux
- hosted `machine list`, `status`, `monitor`, `top`, and `stop` now resolve
  through the live hosted HTTP path:
  CLI/SDK -> `port control-plane serve` -> `port node-agent serve`
- hosted machines with unresolved inventory, such as a host without a matching
  node runtime binding, fail with explicit hosted route context so the
  control-plane mismatch is visible to the operator
- hosted `machine monitor` currently reports runtime-owner context, log and
  manifest paths, and detached forward state from the selected node runtime
  root
- hosted `machine top` currently reports the hypervisor process plus any
  detached forward processes Port recorded under that node runtime root
- hosted `guest exec`, `copy`, `pty`, and `logs` now execute through that same
  live hosted HTTP path while preserving the existing guest protocol payloads
- hosted `guest pty` and `guest logs --follow` now keep their streamed session
  semantics through that hosted route instead of collapsing back to transcript-
  only operator behavior
- hosted guest attach failures surface the control plane and node-routing
  context directly, so missing guest sockets or unresolved hosted node
  ownership stay visible to the operator
- hosted `guest copy` now relays bytes through the control plane and node
  agent using the shared guest copy protocol, so client-side host paths no
  longer need to be visible on the selected node
- hosted `guest forward` now starts a node-owned listener through the live
  hosted control-plane and node-agent path while keeping the same canonical
  command family
- hosted `service secret put|list|remove` now stores machine-scoped secret
  metadata plus runtime-file backend state under the resolved runtime owner
  instead of inventing a separate hosted secret store
- hosted `service apply --kind service|sandbox` now stores service and sandbox
  definitions under that same runtime owner, including desired state, guest
  command, secret bindings, secret-source provenance, hosted routing context,
  and the node-owned runtime record path
- hosted `service list|status|stop` now inspects, updates, and stops the live
  managed process through that same hosted route while surfacing runtime state
  back through the canonical `port service` surface

## Hosted Standard Cloud Workflow

This is the simpler hosted proof lane. It keeps the same command family and
only changes runtime ownership:

```bash
export PORT_DEMO_TOKEN=demo-token
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine stop --machine cloud-aws
```

Provider mapping for the shipped sample machines:

- `cloud-generic` launches through `generic-linux-node`
- `cloud-aws` launches through `aws-linux-node`
- `cloud-gcp` launches through `gcp-linux-node`

Repository-local proof for the shipped standard hosted lane:

```bash
cargo test -q -p port --test machine_commands cli_hosted_standard_cloud_launch_round_trip
cargo test -q -p port --test machine_commands cli_hosted_standard_status_and_stop_round_trip
```

Prepared-node PVM remains a second hosted launch contract layered on the same
control-plane and node-agent route when the machine is switched to
`protection_mode = "pvm"` and the prepared AWS host kit exists. Use
[`aws.md`](aws.md) when you need the stronger production-oriented narrative
instead of this standard-lane proof.

## Multi-Node Hosted Service Workflow

The first multi-node hosted service slice is intentionally narrow: it proves
host-group-targeted placement and canonical service visibility without claiming
full hosted fleet management.

Repository-local workflow:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node-b --bind 127.0.0.1:9235 --token node-secret-b
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service secret put --machine cloud-aws --name demo-token --value s3cr3t
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service apply --machine cloud-aws --host-group aws-secondary --name api --kind service --restart on-failure --health command --health-command /bin/test --health-command=-f --health-command workspace/healthy --secret API_TOKEN=demo-token -- /bin/sh -lc 'count_file=workspace/restarts; count=$(cat "$count_file" 2>/dev/null || echo 0); count=$((count + 1)); printf "%s" "$count" > "$count_file"; if [ "$count" -eq 1 ]; then sleep 0.2; exit 23; fi; trap '\''exit 0'\'' TERM; while :; do sleep 1; done'
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service list --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service status --machine cloud-aws --name api
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service stop --machine cloud-aws --name api
```

Why this is the canonical workflow:

- `service apply --host-group aws-secondary` targets the existing hosted
  inventory model instead of inventing a second scheduler command family.
- The selected node remains visible through `service list`, `status`, and
  `stop`, together with the target host group, scheduler, restart/health
  state, secret-source provenance, and runtime state.
- Both node agents register with the control plane before placement starts, so
  the service workflow now matches the same registered-node story as `machine
  list` and `machine status`.
- If the selected node binding goes stale, the stored placement remains
  operator-visible through the same service commands so routing drift is not
  hidden behind a generic hosted control-plane failure.

Repository-local proof:

```bash
bash scripts/service-reliability-demo.sh
```

Current hosted service limits:

- No autoscaling or rescheduling yet.
- Deterministic-first-fit is the only shipped scheduler policy.
- No broader fleet policy yet.
- Imported inventory exists, but it is still a control-plane-owned state file
  contract rather than a first-class `port inventory import` command.

## Durable Fleet Workflow

Port now ships a repository-local durable fleet workflow for the hosted demo
lane. The canonical operator surfaces stay the same:

- `port control-plane serve` owns and reloads durable hosted fleet state
- `port node-agent serve` refreshes live registration and heartbeat state
- `port machine list|status` surface the merged hosted fleet view

Repository-local workflow:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
```

Durable state files:

- live registration and heartbeat:
  `.port/hosted/<control-plane>/registered-nodes.json`
- imported inventory:
  `.port/hosted/<control-plane>/imported-inventory.json`

Current imported-inventory contract:

- the file is owned by the control plane, not by a separate hosted-only CLI
  family
- records must resolve onto configured node names instead of inventing a
  second fleet namespace
- `port machine status` merges configured inventory, imported inventory, and
  live registration into one canonical fleet view

Illustrative imported-inventory skeleton:

```json
{
  "control_plane": "demo",
  "nodes": {
    "aws-linux-node": {
      "provider": "aws",
      "provenance": "inventory-sync",
      "imported_at": 123,
      "capability_summary": {
        "providers": ["aws"],
        "platforms": ["linux"],
        "substrates": ["firecracker"],
        "architectures": ["x86_64"],
        "protection_modes": ["standard", "pvm"]
      }
    }
  }
}
```

The real `capability_summary` must still match the configured node capability
contract exactly, including any populated `pvm_lanes`, or the control plane
will reject the imported record during validation.

What `port machine status --machine cloud-aws` now surfaces for each hosted
node:

- whether the node is configured
- whether imported inventory exists for that node
- whether a live registration exists
- freshness state (`live`, `stale`, or `missing-registration`)
- routing eligibility and selected-node detail

Restart behavior is now part of the contract. Restarting
`port control-plane serve` reloads both durable state files, so the canonical
`port machine status` view remains available after control-plane restart
instead of depending on process-local memory.

## Canonical Machine Control Contract

Port now names the lifecycle and inventory contract explicitly so later hosted
drivers can reuse the same vocabulary that local `machine list`, `status`,
`monitor`, `top`, and `stop` already publish.

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
- `monitor_route = "direct-local-runtime"`
- `top_route = "direct-local-runtime"`
- `service_route = "direct-local-runtime"`
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
- `monitor_route = "hosted-control-plane"`
- `top_route = "hosted-control-plane"`
- `service_route = "hosted-control-plane"`
- `guest_route = "hosted-control-plane"`

Those tokens are the implementation-ready contract for the current hosted slice
and the next hosted node-agent and control-plane expansions. The local lane
already reports the local form through runtime status surfaces, and hosted
drivers are expected to fill the hosted form rather than inventing a second
vocabulary.

## Guest Operation Brokerage

Hosted Port preserves the existing guest protocol by tunneling it, not by
redefining it.

Hosted guest attach contract for a hosted machine such as `cloud-aws`:

- `guest_broker = "control-plane-node-agent-tunnel"`
- `guest_route = "hosted-control-plane"`
- `command_surface = ["exec", "copy", "pty", "logs", "forward"]`
- `protocol = "port-agent-protocol"`
- `session.id = "port-hosted://demo/machines/cloud-aws/guest-session"`
- `session.scope = "machine"`
- `session.driver.id = "port-guest-shell-driver-v1"`
- `session.driver.route = "hosted-control-plane"`
- `session.driver.broker = "control-plane-node-agent-tunnel"`
- `session.driver.protocol = "port-agent-protocol"`
- attach path:
  CLI -> hosted control plane -> hosted node agent -> in-guest `port-guest-agent`

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

The hosted route context now carries that session contract on both success and
failure responses for guest-backed shell flows. The session identifier stays
machine-scoped and stable across hosted `exec`, `pty`, `forward`, and detached
forward lifecycle calls for the same machine, while the driver metadata keeps
Port's canonical shell-driver vocabulary explicit for downstream systems.
If Port cannot derive the hosted guest-session contract for a hosted guest
route, the request fails instead of falling back to anonymous or partial
session metadata.

For upstream control planes, that is one shell-driver contract rather than
three protocol variants:

| Flow | Canonical Port surface | Lifecycle contract | Stable identity surface |
|------|-------------------------|--------------------|-------------------------|
| `exec` | `port guest exec` | One request and one completed result | Hosted success or error route carries `session.id` and `session.driver.*` |
| `pty` | `port guest pty` | One streamed terminal session until guest exit | `port-sdk` derives the same machine-scoped shell-driver contract before the stream opens |
| `forward` | `port guest forward` | One streamed listener or detached start/list/stop lifecycle | The same machine-scoped shell-driver contract applies across forward start/list/stop |

Provider-aware failures stay explicit on that same contract surface:

- wrong-lane or unplaceable hosted targets return placement guidance instead of
  silently falling back to the standard local lane
- missing AWS host-kit preparation, missing patched PVM VMM support, or missing
  PVM artifacts fail with `cloud-aws`-specific guidance from the hosted control
  plane and node-agent path
- missing hosted guest-session metadata fails immediately instead of degrading
  to anonymous shell state

Hosted `guest forward` keeps the same command family and now starts a
node-owned listener through the hosted control-plane and node-agent path.
Hosted detached lifecycle now ships through the same surface: `--lifecycle
detached --name <forward>`, `--list`, and `--stop --name <forward>` all route
through the live control plane and node agent and operate on node-owned
detached forward state under the selected runtime root.

Example hosted guest workflow:

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest pty \
  --machine cloud-aws -- /bin/sh -lc 'printf hosted-pty-ok'

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest logs \
  --machine cloud-aws --path /var/log/app.log --follow

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest copy \
  --machine cloud-aws --direction host-to-guest \
  --source ./host.txt --destination /workspace/host.txt

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest forward \
  --machine cloud-aws --listen 127.0.0.1:8081 --target 127.0.0.1:80

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest forward \
  --machine cloud-aws --listen unix:/tmp/cloud-aws.sock --target unix:/var/run/app.sock \
  --lifecycle detached --name demo-sock

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest forward \
  --machine cloud-aws --list

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted.toml guest forward \
  --machine cloud-aws --stop --name demo-sock
```

- `guest pty` and `guest logs --follow` stay streamed across the hosted route.
- `guest copy` now streams bytes through the node agent instead of assuming
  node-visible host paths.
- `guest forward` returns the node-owned listener address and does not require
  a repo-local guest transport fallback any more.
- `guest forward --lifecycle detached --name <forward>` records detached state
  under the selected node runtime root, `--list` reads that state, and
  `--stop --name <forward>` tears the listener down through the same hosted
  route family.

What still remains after this runtime slice:

- retries and richer client policies on top of the shipped transport
- advanced auth/tenancy work on top of the same API paths

## Hosted API Shape

The exact wire API is not fully implemented yet, but Port now documents and
publishes the request surface through `port-sdk`. The contract mirrors the CLI:

- `machines.create`
- `machines.list`
- `machines.get`
- `machines.monitor`
- `machines.top`
- `machines.stop`
- `machines.connect_guest`
- `services.apply`
- `services.list`
- `services.get`
- `services.stop`
- `secrets.put`
- `secrets.list`
- `secrets.remove`

Those verbs are the hosted counterpart of today's local runtime calls and
should remain substrate-aware without becoming Firecracker-specific API names.
`port-sdk` now builds and executes typed requests for these paths while reusing
the shared route and auth contract from `port-hosted-protocol`.

## Current Boundary

- Port now ships a repository-local hosted daemon pair for the single-node demo
  lane: `port control-plane serve` and `port node-agent serve`.
- hosted `service secret` and `service apply|list|status|stop` are also
  config-backed and in-process; they persist spec state under the selected node
  `runtime_root`, execute managed guest processes through the live hosted
  route, and expose restart/health state, secret-source provenance, and the
  canonical runtime-state record path.
- managed guest-process `start|list|status|stop` remains an internal guest and
  node runtime contract beneath the same canonical `port service` surface; it
  is not a hosted-only CLI family.
- `port-sdk` now ships the supported typed client entry points plus live JSON
  execution for machine, guest, and service operations.
- Those commands already report the control-contract fields above so the
  operator-visible lifecycle vocabulary does not need to change when the demo
  lane grows into a broader hosted product.
- Remote Linux providers are modeled and diagnosed, but remote launch remains a
  designed boundary rather than a shipped orchestration path.
- The hosted contract is executable through the canonical CLI and mirrored by
  the SDK, but the shipped lane is still explicitly a repository-local,
  explicit-binding demo rather than a hardened hosted fleet product.
