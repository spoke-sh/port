# Port

Port is an open-source, Rust-based Firecracker management system for operating
isolated Linux workloads across local and cloud environments, with a CLI-first
control plane for services, sandboxes, and Kubernetes-oriented workloads.

Port aims to make microVM-backed workloads practical in the same way containers
made process-based workloads practical: fast provisioning, clear operational
surfaces, and repeatable deployment workflows, with a Linux guest boundary and
stronger isolation.

## What Port Is For

Port is intended to manage Firecracker microVMs that can be used to:

- Run coding agents inside isolated environments
- Launch disposable build, test, and automation environments
- Host long-lived applications and services
- Provide worker capacity for orchestration and Kubernetes-adjacent workflows
- Bridge local development and cloud deployment with the same operational model

## Planned Capabilities

- CLI-based microVM lifecycle management
- Local and remote environment targeting
- Fast provisioning of Linux workloads on Firecracker
- Command execution, shell access, file transfer, and service exposure
- Networking, secrets, configuration, and operational metadata management
- Support for both ephemeral workloads and long-lived deployed services
- Integration points for coding agent orchestration and higher-level schedulers

## Platform Support

Port currently supports one real Firecracker execution lane and two documented
operator lanes around it:

- Linux: run the full local MVP workflow, including artifact build, `port doctor`,
  and `port machine launch`, on a host where `port doctor` passes.
- macOS: `port doctor` now surfaces the AVF machine contract and entitlement
  boundary for locally modeled AVF machines, `machine launch|status|stop`
  route through a local AVF driver, and the canonical `guest exec|copy|pty|logs|forward`
  verbs now attach through the same runtime root when the configured launcher
  helper exposes the expected AVF transport socket and console log. `nix develop`
  evaluates on macOS for repo tooling, but it intentionally omits Linux-only
  runtime packages such as `firecracker`, `iproute2`, and `iptables`.
- Windows: use WSL for the repository and CLI if helpful, but treat `port doctor`
  as the gate for whether local Firecracker launch is actually available in that
  Linux environment. Otherwise run the same `port` commands on a remote Linux
  host.

Detailed operator workflows live in [`docs/operators.md`](docs/operators.md).
Cloud-provider boundaries and the current remote Linux support matrix live in
[`docs/cloud.md`](docs/cloud.md). The Firecracker/PVM host-kit contract lives in
[`docs/pvm.md`](docs/pvm.md). The Apple Virtualization Framework contract lives
in [`docs/avf.md`](docs/avf.md). The hosted SDK and API client surface lives in
[`docs/sdk.md`](docs/sdk.md).

## Project Status

Port is early-stage and under active development. The repository now contains
the canonical Rust workspace, a working local Linux Firecracker launch path,
the first guest-agent-backed CLI workflows, and in-repo kernel plus guest-image
artifact pipelines. Cloud host support is still partial: `port doctor` and the
shared model describe the remote Linux lane, while remote `port machine launch`
still fails fast with provider-aware guidance instead of attempting a hidden
runtime path. Substrate-aware expansion is now in flight: the current executable
lane remains Firecracker with standard KVM on Linux, while Firecracker/PVM,
Cloud Hypervisor, and Apple Virtualization Framework are modeled explicitly as
planned or research-backed lanes rather than hidden future scope. The hosted
Port control split is now documented explicitly: local CLI ownership is the
current shipped lane, while the hosted direction is a CLI client talking to a
control plane plus node-local runtime owner.

## Execution Lanes

Port now distinguishes provider identity from execution lane. The current
canonical substrate matrix is:

| Substrate | Protection mode | Architecture | Status | Current Port position |
|-----------|-----------------|--------------|--------|------------------------|
| Firecracker | `standard` | `x86_64`, `aarch64`, or `native` | Supported today | This is the real Linux execution lane behind today's `port machine launch` workflow |
| Firecracker | `pvm` | `x86_64` | Prepared-node / partial rollout | Launches through the hosted control-plane and node-agent path on prepared Linux nodes; broader packaging and rollout still require dedicated host-kit and artifact work |
| Firecracker | `pvm` | `aarch64` | Research lane | Upstream protected virtualization exists, but Port does not yet claim a supportable Firecracker runtime path here |
| Cloud Hypervisor | `standard` | `x86_64` or `aarch64` | Planned | Secondary Linux hypervisor lane, not yet implemented |
| Apple Virtualization Framework | `standard` | `arm64` or `x86_64` on macOS | Planned | First-class macOS lane in the model and docs, not yet implemented |

## PVM Contract

Port's PVM position is intentionally concrete:

- keep Firecracker/PVM on `x86_64` as the first implementation lane
- treat it as a dedicated host kit plus artifact kit, not just a model flag
- keep Firecracker/PVM on `aarch64` research-only until there is a supportable
  Firecracker runtime path

The full host-kit, artifact-kit, validation, and follow-on implementation
contract lives in [`docs/pvm.md`](docs/pvm.md).

Repository-local PVM foundation workflow:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml machine launch --machine demo
```

Interpretation:

- passing artifact commands mean the `x86_64/firecracker/pvm` artifact kit is
  materialized locally
- `port doctor` now emits `pvm:local:x86_64:*` checks for platform,
  architecture, boot-line, and patched-binary readiness
- a missing `pti=off` boot arg or `firecracker-pvm` binary is an explicit host
  kit failure, not a signal to fall back to the standard Firecracker lane
- `aarch64/firecracker/pvm` remains research-only in the model, docs, and
  scripts
- a local PVM launch still requires a prepared x86_64 Linux host with the
  patched `firecracker-pvm` binary and required host boot state even after the
  PVM artifact build/validate workflow passes
- `port machine launch --machine demo` remains the preserved standard
  Firecracker proof and should keep working independently of the PVM artifact
  workflow

Hosted prepared-node PVM workflow:

Start from a copy of `examples/port.toml` and make these temporary changes:

- switch `machines.cloud-aws.protection_mode` to `pvm`
- point the `x86_64/firecracker/pvm` kernel and guest-image variants at the
  prepared artifact paths for the node
- export `PORT_PVM_FIRECRACKER_BINARY` to the patched `firecracker-pvm` binary
  on the prepared node host

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_PVM_FIRECRACKER_BINARY=/path/to/firecracker-pvm PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine stop --machine cloud-aws
```

Interpretation:

- `cloud-generic` is the sample hosted denial case: if you switch it to
  `protection_mode = "pvm"`, Port marks it `malformed` with an explicit
  placement reason because `generic-linux-node` advertises `state = "planned"`
- `cloud-aws` is the sample hosted prepared-node case: once you switch it to
  `protection_mode = "pvm"` and provide the PVM artifact paths plus
  `firecracker-pvm`, `port machine launch` routes through the live control
  plane and prepared node agent to boot the VM
- missing `firecracker-pvm`, missing host boot prerequisites, or missing PVM
  artifact paths fail explicitly; Port does not fall back to the standard
  Firecracker lane
- other hosted launch paths still return provider-aware guidance until their
  runtime lanes ship

## AVF Contract

Port's macOS lane follows the same rule as the Linux and hosted lanes:

- keep the canonical `machine` and `guest` verbs
- map guest transport onto AVF virtio sockets
- map console/log capture onto AVF serial ports
- keep directory sharing and Rosetta as optional operator workflows, not as a
  replacement for the guest-agent protocol

Native macOS quick reference:

```bash
port --config examples/port.toml doctor
PORT_AVF_LAUNCHER=/path/to/port-avf-launcher port --config examples/port.toml machine launch --machine demo-avf
port --config examples/port.toml machine status --machine demo-avf
port --config examples/port.toml guest exec --machine demo-avf -- /bin/sh -lc 'uname -a'
port --config examples/port.toml machine monitor --machine demo-avf
port --config examples/port.toml machine stop --machine demo-avf
```

The checked-in sample config now includes `machines.demo-avf` on `hosts.mac-local`.
On non-macOS hosts that launch path fails fast with explicit macOS-only
guidance, while Firecracker launch remains a Linux-only workflow.

The full AVF runtime, operator, and follow-on implementation contract lives in
[`docs/avf.md`](docs/avf.md).

## CLI Surface

The canonical binary is `port`. The current command tree is:

```text
port doctor
port artifacts build --artifact <name> [--architecture <native|x86-64|aarch64>] [--substrate <firecracker|cloud-hypervisor|avf>] [--protection-mode <standard|pvm>]
port artifacts validate --artifact <name> [--architecture <native|x86-64|aarch64>] [--substrate <firecracker|cloud-hypervisor|avf>] [--protection-mode <standard|pvm>]
port artifacts push --artifact <name> [--architecture <native|x86-64|aarch64>] [--substrate <firecracker|cloud-hypervisor|avf>] [--protection-mode <standard|pvm>]
port artifacts pull --artifact <name> [--architecture <native|x86-64|aarch64>] [--substrate <firecracker|cloud-hypervisor|avf>] [--protection-mode <standard|pvm>]
port machine launch --machine <name>
port machine list [--runtime-root <path>]
port machine status --machine <name> [--runtime-root <path>]
port machine monitor --machine <name> [--runtime-root <path>]
port machine top --machine <name> [--runtime-root <path>]
port machine stop --machine <name> [--runtime-root <path>]
port guest exec --machine <name> -- <command...>
port guest copy --machine <name> --direction <host-to-guest|guest-to-host> --source <path> --destination <path>
port guest pty --machine <name> -- <command...>
port guest logs --machine <name> --path <path> [--tail-lines <n>] [--follow]
port guest forward --machine <name> --listen <tcp-addr|unix:path> --target <tcp-addr|unix:path> [--lifecycle <foreground|detached>] [--name <name>] [--list] [--stop]
port service secret put --machine <name> --name <secret> --value <value>
port service secret list --machine <name>
port service secret remove --machine <name> --name <secret>
port service apply --machine <name> --name <name> [--kind <service|sandbox>] [--secret <ENV=SECRET_NAME>]... -- <command...>
port service list --machine <name>
port service status --machine <name> --name <name>
port service stop --machine <name> --name <name>
port control-plane serve --control-plane <name> [--bind <addr>] [--node-binding <node>=<endpoint>,<token>]...
port node-agent serve --node <name> [--bind <addr>] --token <token>
```

Use `port --help` or any nested `--help` command to inspect the current command
model and examples. The sample `port --help` commands assume you are running
from the repository root. Local artifact and launch examples also assume the
needed runtime tools are available in the execution environment and on `PATH`.

Local lifecycle quick reference:

- `port machine launch --machine <name>` creates the Port-managed runtime state
  for one local machine under the selected runtime root and starts Firecracker.
- `port machine list` discovers local machines by enumerating Port-managed
  runtime state; it is the canonical local inventory view rather than a direct
  Firecracker API query.
- `port machine status --machine <name>` reads that runtime state back and
  prints the paths and process metadata Port recorded for one machine.
- `port machine monitor --machine <name>` expands that status view with the
  current runtime-owner context, detached forward state, and the paths Port is
  using for operator-visible logs and manifests.
- `port machine top --machine <name>` inspects the hypervisor process plus any
  detached forward processes Port recorded for that machine.
- `port machine stop --machine <name>` stops a local Port-managed machine and
  cleans up runtime ownership details so the next launch is deterministic.
- Those same lifecycle verbs are also the groundwork for the hosted product:
  the operator model stays the same while runtime ownership moves from local
  files plus processes to a hosted control plane and node-local agent.

Current behavior:

- `port artifacts build` and `port artifacts validate` now run real in-repo
  kernel and guest-image pipelines for the selected artifact variant.
- `port artifacts push` and `port artifacts pull` now use the artifact's
  configured mobility backend. The sample config defaults to a file-backed
  registry/cache contract, and `hosted-api` is also executable when the
  selected artifact points at a live control plane. OCI remains follow-on work.
- `port doctor` performs real host checks for Linux, `/dev/kvm`, `firecracker`,
  `ip`, and `iptables`. When you pass `--config`, it also validates the native
  Firecracker/standard artifact variant paths required on the current host.
- `port machine launch` now writes a Firecracker config plus runtime metadata
  and console/log files under the chosen runtime root before invoking
  Firecracker with `--config-file`.
- `port machine list`, `port machine status`, `port machine monitor`,
  `port machine top`, and `port machine stop` now use Port-managed manifests,
  detached forward manifests, and live PID inspection instead of relying on
  the Firecracker REST API. Once a machine is launched, these commands operate
  on the runtime root and do not require the model file again.
- The hosted direction keeps those same lifecycle and guest verbs; the
  published contract moves long-lived runtime ownership from the short-lived
  CLI process to a node agent plus control plane rather than inventing a second
  operator model.
- `port control-plane serve` now ships the first live hosted HTTP server for
  canonical machine and guest routes. It authenticates client requests from the
  configured control-plane contract and forwards them to explicitly bound
  node-agent endpoints for the demo lane.
- `port node-agent serve` now ships the matching hosted node-runtime server for
  the demo lane. It authenticates control-plane calls and reuses Port's
  existing runtime-root and guest transport logic behind the internal node
  routes.
- `port doctor` also reports provider-aware support boundaries for
  `generic-linux`, `aws`, `gcp`, and `azure` hosts when they are present in the
  config.
- `port machine launch` still supports the shipped local Linux standard lane
  directly, and now also supports hosted x86_64 PVM launch when a machine
  resolves to a ready prepared node with the required PVM host kit and PVM
  artifact paths.
- hosted PVM placement and host-kit failures stay explicit before launch:
  unplaceable nodes, missing `firecracker-pvm`, or missing PVM artifact paths
  fail with concrete detail instead of looking like generic transport failures.
- `port guest exec`, `copy`, `pty`, `logs`, and `forward` now speak the shared
  guest-agent protocol through the canonical CLI and return structured results
  rendered as human-readable CLI output.
- `port service secret` and `port service apply|list|status|stop` now persist
  machine-scoped secret references plus service or sandbox definitions under
  the resolved runtime owner, and the hosted demo lane executes them through
  the live control-plane and node-agent path. Status still surfaces desired
  state, guest command, secret bindings, hosted routing context, and a
  canonical service runtime record path.

## Cloud Linux Support

Port keeps one canonical host model for local Linux and remote Linux/cloud
targets, but provider is no longer the only planning axis. Today the executable
cloud-facing lane is still Firecracker with `standard` protection on Linux
hosts. The current provider matrix for that lane is:

| Provider | Example machine | MVP status | Current command behavior |
|----------|-----------------|------------|--------------------------|
| `local` | `demo` | Supported | `port doctor` performs local preflight; `port machine launch --machine demo` can launch Firecracker on Linux |
| `generic-linux` | `cloud-generic` | Partial | `port doctor` reports the future remote Linux lane; `port machine launch` tells you to run Port on that Linux host directly |
| `aws` | `cloud-aws` | Prepared-node / partial | `port doctor` reports AWS readiness details; `port machine launch --machine cloud-aws` launches through the hosted control plane when the machine is switched to `pvm` and the prepared node host kit plus PVM artifact paths exist |
| `gcp` | `cloud-gcp` | Partial | `port doctor` reports GCP as a justified future lane; `port machine launch` fails with GCP-specific guidance |
| `azure` | `cloud-azure` | Unsupported | `port doctor` reports Azure as unsupported for Firecracker MVP and `port machine launch` rejects it immediately |

The remote Linux workflow is intentionally limited today outside the prepared
hosted PVM lane:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml machine launch --machine cloud-aws
```

The first command surfaces the provider-aware support matrix through the CLI.
The second command is still expected to fail with an AWS-specific message until
you switch `cloud-aws` to `protection_mode = "pvm"` and provide the prepared
node host kit plus PVM artifact paths described in [`docs/pvm.md`](docs/pvm.md).

The explicit cloud design, remote workflow, and substrate guidance live in
[`docs/cloud.md`](docs/cloud.md).

## Hosted Control

Port's current executable path is still local, but the canonical hosted split
is now documented in [`docs/hosted.md`](docs/hosted.md).

- The `port` CLI remains the canonical operator surface in both local and
  hosted modes.
- `port node-agent serve` now ships the first live hosted node-runtime server
  for one execution node and runtime root, and it registers that ownership with
  the configured control plane.
- `port control-plane serve` now ships the matching hosted control-plane
  server for the single-node demo lane.
- The sample config now names that hosted API identity explicitly under
  `[control_planes.demo]`, with endpoint `https://port.example.internal`,
  audience `port-hosted-demo`, and a bearer token sourced from
  `PORT_DEMO_TOKEN`.
- Hosted inventory is now modeled explicitly through `[nodes.<name>]` and
  `[host_groups.<name>]`, with capability, node-agent `runtime_root`,
  explicit-membership placement, and deterministic-first-fit scheduler fields
  that later scheduler, monitoring, and services work can reuse.
- Hosted `machine list`, `status`, `monitor`, `top`, and `stop` are also
  modeled explicitly as control-plane plus node-agent contracts so the
  canonical machine verbs stay stable as Port moves from the local runtime to
  a hosted fleet. Those verbs now execute through the live hosted HTTP path to
  `port control-plane serve`, which routes to `port node-agent serve` for the
  demo lane without introducing hosted-only verbs.
- Hosted `machine launch` now uses that same control-plane plus node-agent path
  for prepared x86_64 PVM machines. Other hosted launch paths still return
  provider-aware guidance instead of pretending they are live.
- Registered hosted node workflow and durable fleet inspection for the current
  demo lane:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine list
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
```

- `port node-agent serve` refreshes live node registration and heartbeat state
  into `.port/hosted/demo/registered-nodes.json`.
- Imported inventory is now a real control-plane contract under
  `.port/hosted/demo/imported-inventory.json`; there is not yet a first-class
  `port inventory import` command, so the current operator path is to seed or
  sync that file and then inspect the result through `port machine list|status`.
- `port machine status --machine cloud-aws` now surfaces per-node configured,
  imported, registered, freshness, selected, and routing-eligibility state so
  operators can tell live, stale, and imported-only fleet members apart
  without reading runtime files directly.
- Control-plane restart recovery is now part of the supported repo-local hosted
  workflow: restarting `port control-plane serve` reloads both durable fleet
  files and preserves the canonical `port machine status` view.
- `port control-plane serve --node-binding <node>=<endpoint>,<token>` remains
  available only as a bootstrap or debug override when a node cannot
  self-register yet.
- Hosted guest `exec`, `copy`, `pty`, `logs`, and `forward` are now modeled as
  a control-plane-authorized attach followed by node-agent guest brokerage to
  the in-guest `port-guest-agent`. The canonical `guest` verbs and guest
  protocol frames stay unchanged. Hosted `guest exec|copy|pty|logs` now execute
  through that live HTTP path for the demo lane without introducing hosted-only
  aliases.
- Hosted `guest copy` now transfers bytes through the control-plane and
  node-agent path using the shared guest copy protocol, so the demo lane no
  longer assumes the client host paths are visible on the selected node.
- Hosted `guest forward` now supports foreground and detached lifecycle modes
  through the hosted control-plane and node-agent path. `--list`, `--stop`,
  and `--name` now manage node-owned detached forward state under the selected
  node runtime root instead of falling back to repo-local runtime files.
- Hosted `port service` now has a repository-local multi-node workflow through
  explicit host groups and node bindings: `service apply --host-group
  aws-secondary` selects one eligible node, while `service list|status|stop`
  surface the selected node, target host group, scheduler, and runtime state
  through the canonical service output.
- Hosted inventory that lacks a matching node runtime binding currently
  fails with explicit control-plane and route context instead of being silently
  dropped.
- Current hosted fleet limits remain explicit: no autoscaling, no broader fleet
  policy, and no first-class `port inventory import` command yet.
- `port machine monitor` and `port machine top` now make the hosted monitoring
  boundary explicit: they inspect node-agent-owned runtime state, detached
  forwards, and live processes, but they are not yet a full metrics,
  secrets/services, or sandbox execution product.
- `port service` is now the canonical secrets/services/sandboxes family. It
  uses the same resolved runtime ownership as `machine` and `guest`, with
  sandboxes expressed as `--kind sandbox` instead of a second runtime model.
- Secret values are currently stored as runtime-owned JSON files under the
  resolved machine runtime root. Treat that as a bootstrap operator workflow,
  not as a hardened secret backend.
- `port-sdk` now publishes typed hosted request builders plus live response
  execution helpers for canonical machine, guest, and service operations so
  SDK/API consumers can follow the same surface as the CLI.
- `port-hosted-protocol` now publishes the shared hosted HTTP route, auth, and
  route-context contract that later control-plane and node-agent servers will
  reuse instead of inventing a second hosted path model.
- Local `machine list`, `status`, `monitor`, `top`, and `stop` now publish the
  control-contract fields that future hosted routing will reuse: inventory
  scope, inventory owner, lifecycle owner, status source, and per-verb route.

Repository-local hosted demo proof:

```bash
export PORT_DEMO_TOKEN=demo-token
bash scripts/hosted-demo.sh
```

That demo script prepares temporary hosted server and client configs, starts
`port-guest-agent`, `port node-agent serve`, and `port control-plane serve`,
waits for node registration, then runs canonical hosted `port machine list`,
`port machine status`, `port guest exec`, `port guest copy`, `port guest
logs`, and hosted detached `port guest forward` start, list, and stop
commands end-to-end. Detached forward lifecycle now stays on the same
control-plane and node-agent path as the foreground hosted guest bridge.

## Multi-Node Hosted Service Workflow

The first hosted multi-node service slice stays on the canonical `port service`
surface instead of introducing a scheduler-only command family.

Repository-local workflow:

```bash
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node-b --bind 127.0.0.1:9235 --token node-secret-b
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service secret put --machine cloud-aws --name demo-token --value s3cr3t
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service apply --machine cloud-aws --host-group aws-secondary --name api --kind service --secret API_TOKEN=demo-token -- /bin/sh -lc 'trap '\''exit 0'\'' TERM; while :; do sleep 1; done'
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service list --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service status --machine cloud-aws --name api
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml service stop --machine cloud-aws --name api
```

What operators should expect:

- `service apply --host-group aws-secondary` selects one eligible member of
  that explicit host group and stores the placement under the selected node
  runtime root.
- Both node agents register themselves with the control plane before placement
  starts; manual `--node-binding` is not the default operator path anymore.
- `service list`, `status`, and `stop` surface the selected node, target host
  group, scheduler, and runtime state through the same canonical service
  output.
- If the control plane later loses the selected node binding, `service list`,
  `status`, and `stop` still surface the stored placement and explain the
  stale-binding failure instead of collapsing into a generic hosted error.

Current hosted service limits:

- No autoscaling or rescheduling yet.
- Deterministic-first-fit is the only shipped scheduler policy.
- No higher-level fleet manager or broader service orchestration yet.
- Durable node registration and imported inventory now exist, but placement
  still depends on the current deterministic-first-fit policy plus explicit
  node inventory instead of a richer fleet manager.

## Linux Local Workflow

The supported end-to-end Linux MVP workflow is:

```bash
cargo run -p port-cli -- doctor
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest --architecture native
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
cargo run -p port-cli -- machine list
cargo run -p port-cli -- machine status --machine demo
cargo run -p port-cli -- machine monitor --machine demo
cargo run -p port-cli -- machine top --machine demo
cargo run -p port-cli -- machine stop --machine demo
```

What that produces:

- deterministic artifact variants under `artifacts/<kind>/<name>/<architecture>/<substrate>/<protection-mode>/`
- host validation through `port doctor`
- Firecracker runtime state, logs, and manifest files under the chosen runtime root
- lifecycle and monitoring surfaces through `port machine list`, `status`,
  `monitor`, `top`, and `stop`

The lifecycle commands above intentionally switch from model-driven launch to
runtime-state-driven management. After `port machine launch` creates
`runtime/<machine>/`, use `port machine list`, `port machine status --machine
<name>`, `port machine monitor --machine <name>`, `port machine top --machine
<name>`, and `port machine stop --machine <name>` to inspect and control that
local Port-managed runtime state without going through Firecracker directly.
That local ownership model is also the basis for the hosted control design: the
same verbs remain canonical even when the hosted node agent and control plane
own the runtime on behalf of the CLI. The commands now surface that contract
explicitly with local values such as `local-runtime-root`,
`local-port-runtime`, `runtime-manifest-and-host-process`, and
`direct-local-runtime`.

If the required tools are not available, `port doctor` may report missing
prerequisites such as `firecracker` on `PATH`, and `port machine launch` is
expected to fail until that environment is corrected.

The launched-VM guest transport is now the canonical path for the guest CLI:

- `port guest exec`, `port guest copy`, `port guest pty`, `port guest logs`,
  and `port guest forward` all connect to launched Firecracker VMs through the
  machine model's configured guest control port.
- `port guest copy` now transfers bytes across the real host/guest boundary
  instead of assuming the guest can see host filesystem paths directly.
- `port guest forward` is a foreground host-side proxy session. It binds on the
  host, connects each inbound client to the guest target through the guest
  transport, and runs until you stop it.

The first service/sandbox surface builds on that same runtime ownership model:

- `port service secret put|list|remove` stores machine-scoped secret references
  under the resolved runtime owner.
- `port service apply --kind service|sandbox` stores a guest command plus
  secret bindings under the same runtime owner and, for hosted machines,
  executes the resulting managed process through the live control-plane and
  node-agent path instead of inventing a second hosted surface.
- `port service list|status|stop` lets operators inspect or change the desired
  state for those definitions and surfaces the canonical runtime-state
  contract, including the node-owned runtime record path.
- Managed guest-process `start|list|status|stop` is an internal runtime
  contract, not a second hosted-only CLI surface.
- Secret values are still stored as runtime-owned JSON for the demo lane, and
  restart policy, health checks, scheduler policy, and hardened secret
  backends remain follow-on work.

## SDK And API Clients

Port now ships the in-repo [`port-sdk`](crates/port-sdk/src/lib.rs) crate as
the supported hosted client surface. It now covers both typed request
construction and live JSON response execution against the hosted control-plane
API.

The shared hosted HTTP route and auth contract now lives in
[`crates/port-hosted-protocol/src/lib.rs`](crates/port-hosted-protocol/src/lib.rs).

- `HostedClient::from_machine` derives the hosted endpoint, audience, and auth
  header shape from the shared Port model and `port-hosted-protocol`.
- `HostedClient::from_machine_env` and `HostedClient::from_control_plane_env`
  derive the token source from the model and read the configured environment
  variable automatically.
- `machines()` mirrors `port machine list|status|monitor|top|stop`.
- `guest()` mirrors `port guest exec|copy|pty|logs|forward` and reuses the
  existing `port-agent-protocol` payloads.
- `services()` mirrors `port service secret put|list|remove` and
  `port service apply|list|status|stop`.
- `HostedClient::execute_json` sends those typed requests over HTTPS/HTTP and
  decodes either the success payload or structured hosted route errors.

See [`docs/sdk.md`](docs/sdk.md) for the request-path contract and
[`crates/port-sdk/examples/hosted-sdk.rs`](crates/port-sdk/examples/hosted-sdk.rs)
for a minimal example.

## Artifact Workflow

Port artifacts now have one canonical identity model:

- a logical reference, for example `demo-fs/port/demo-kernel:v1`
- one or more variants selected by `architecture`, `substrate`, and
  `protection-mode`
- a `push` backend, a `pull` backend, and a local cache root

Build and validate the native sample variants through the canonical CLI:

```bash
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest --architecture native
```

Publish and fetch one selected variant:

```bash
cargo run -p port-cli -- --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64
rm -f artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux
cargo run -p port-cli -- --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64
```

Hosted backend proof:

- Copy `examples/port.toml` to a temp config and replace the existing
  `[control_planes.demo]`,
  `[artifacts.kernels.demo-kernel.distribution.push]`, and
  `[artifacts.kernels.demo-kernel.distribution.pull]` sections with the
  hosted-api snippet in [`docs/artifacts.md`](docs/artifacts.md).
- Start the control plane with `PORT_DEMO_TOKEN=demo-token`.
- Build, push, remove the local kernel path, then pull the same variant back
  through the canonical CLI.
- Hosted artifact bytes persist under
  `.port/hosted/<control-plane>/artifacts/...` on the control-plane owner, not
  under the caller's local file-backed store root.

Artifact contracts:

- `demo-kernel` fetches a pinned Firecracker-compatible kernel from the official
  Firecracker CI bucket and validates its architecture-specific sha256 digest.
  In the sample model, the native build lands at
  `artifacts/kernel/demo/<architecture>/firecracker/standard/vmlinux`, and the
  PVM foundation lane additionally materializes
  `artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux`.
- `demo-guest` builds a deterministic ext4 rootfs containing BusyBox userspace,
  `/init`, and the `port-guest-agent` binary. The guest init path reads
  `port.guest_control_port` from the kernel cmdline and launches the guest
  agent on that vsock port, then validates the filesystem layout with `e2fsck`
  and `debugfs`. Its native output lands at
  `artifacts/guest/demo/<architecture>/firecracker/standard/rootfs.ext4`, and
  the PVM foundation lane additionally materializes
  `artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4` with explicit
  protection-mode markers.
- The sample config uses a file-backed store at `artifact-store/demo-fs/` and a
  cache root at `.port/cache/`. `push` writes the selected variant into that
  store and warms the cache; `pull` restores the selected variant from the
  store into both the cache and the canonical local path used by launch.
- The hosted artifact backend is now executable through the control plane when
  an artifact distribution backend is switched to `hosted-api`. Hosted pushes
  and pulls use bearer-token auth from `PORT_DEMO_TOKEN` and persist under
  `.port/hosted/<control-plane>/artifacts/...`.
- OCI artifact distribution remains follow-on work.

Detailed contracts, inputs, outputs, and validation expectations live in
[`docs/artifacts.md`](docs/artifacts.md).

## Guest Agent Workflow

For launched Firecracker VMs, the canonical `port guest ...` commands now use
the live guest transport automatically:

- Port resolves the machine's configured guest control port from the model.
- The host runtime connects to `<runtime-root>/<machine>/guest.vsock`.
- Firecracker tunnels that host-side socket into the guest-side
  `port-guest-agent` vsock listener.

The runtime Unix socket at `<runtime-root>/<machine>/guest-agent.sock` remains
the explicit local shim path when present, mainly for tests and host-local
debugging, but launched VMs no longer depend on it.

Example flows:

```bash
cargo run -p port-cli -- --config examples/port.toml guest exec \
  --machine demo --runtime-root /tmp/port-runtime -- \
  /bin/sh -lc 'uname -a'

cargo run -p port-cli -- --config examples/port.toml guest copy \
  --machine demo --runtime-root /tmp/port-runtime \
  --direction host-to-guest \
  --source ./host.txt --destination /workspace/host.txt

cargo run -p port-cli -- --config examples/port.toml guest pty \
  --machine demo --runtime-root /tmp/port-runtime -- \
  /bin/sh -lc 'printf pty-ok'

cargo run -p port-cli -- --config examples/port.toml guest logs \
  --machine demo --runtime-root /tmp/port-runtime \
  --path /var/log/port-agent.log --tail-lines 50

cargo run -p port-cli -- --config examples/port.toml guest logs \
  --machine demo --runtime-root /tmp/port-runtime \
  --path /var/log/port-agent.log --follow

cargo run -p port-cli -- --config examples/port.toml guest forward \
  --machine demo --runtime-root /tmp/port-runtime \
  --listen 127.0.0.1:8080 --target 127.0.0.1:80

cargo run -p port-cli -- --config examples/port.toml guest forward \
  --machine demo --runtime-root /tmp/port-runtime \
  --listen unix:/tmp/port-demo.sock --target unix:/var/run/app.sock

cargo run -p port-cli -- --config examples/port.toml guest forward \
  --machine demo --runtime-root /tmp/port-runtime \
  --listen 127.0.0.1:8081 --target 127.0.0.1:80 \
  --lifecycle detached --name demo-web
```

Current forward lifecycle:

- `port guest pty` now keeps a real streamed PTY session open until the guest
  command exits with an explicit exit frame.
- `port guest logs --follow` now keeps the guest log stream open until the
  stream ends or the operator interrupts the command.
- `port guest forward` is a foreground host-side proxy. The command prints the
  bound listener address, keeps serving until you interrupt it, and opens one
  guest transport connection per inbound client.
- `port guest forward --lifecycle detached` starts the same forwarding model in
  a detached Port-managed daemon process. Use the same `port guest forward`
  command with `--list` to inspect detached sessions and `--stop --name <name>`
  to terminate one.
- `--listen` and `--target` accept TCP addresses such as `127.0.0.1:8080` and
  Unix-socket addresses written as `unix:/path/to/socket`.
- Guest-side `--target` addresses still depend on guest networking being up.
  In the sample guest image, bring loopback up before targeting
  `127.0.0.1`, for example with
  `port guest exec --machine demo -- /bin/sh -lc 'busybox ifconfig lo up'`.

Hosted guest transport uses the same command family with explicit boundaries:

```bash
PORT_DEMO_TOKEN=demo-token cargo run -p port-cli -- --config /tmp/port-hosted.toml guest pty \
  --machine cloud-aws -- /bin/sh -lc 'printf hosted-pty-ok'

PORT_DEMO_TOKEN=demo-token cargo run -p port-cli -- --config /tmp/port-hosted.toml guest logs \
  --machine cloud-aws --path /var/log/app.log --follow

PORT_DEMO_TOKEN=demo-token cargo run -p port-cli -- --config /tmp/port-hosted.toml guest copy \
  --machine cloud-aws --direction host-to-guest \
  --source ./host.txt --destination /workspace/host.txt

PORT_DEMO_TOKEN=demo-token cargo run -p port-cli -- --config /tmp/port-hosted.toml guest forward \
  --machine cloud-aws --listen 127.0.0.1:8081 --target 127.0.0.1:80
```

- Hosted `guest pty`, `guest logs --follow`, and `guest copy` now route through
  the live control-plane and node-agent path instead of requiring direct access
  to the selected node runtime root.
- Hosted `guest forward` now starts a node-owned listener and returns the
  remote listen address through the same canonical command family.
- Hosted `guest forward --list`, `--stop`, `--lifecycle detached`, and `--name`
  now execute through the live control-plane and node-agent path and inspect or
  mutate node-owned detached forward state for the selected machine.

## Model And Example Config

Port keeps one canonical machine model for artifacts, hosts, and machines. The
initial sample model lives at [`examples/port.toml`](examples/port.toml).

The host model now carries explicit provider identity:

- `provider = "local"` for the supported local Linux launch lane and the native
  AVF sample lane, distinguished by `platform = "linux"` versus
  `platform = "macos"`
- `provider = "generic-linux"` for future remote Linux control
- `provider = "aws"` and `provider = "gcp"` for the justified future cloud lanes
- `provider = "azure"` for the explicitly unsupported MVP lane

The machine and artifact model now also carries explicit compatibility terms:

- `substrate = "firecracker" | "cloud-hypervisor" | "avf"`
- `protection_mode = "standard" | "pvm"`
- `architecture = "native" | "x86_64" | "aarch64"`
- artifact references, variant selectors, and mobility backends for local build
  outputs plus future remote distribution

The workspace crates are:

- `port-model`: serializable artifact, host, and machine definitions
- `port-agent-protocol`: shared guest-agent request and response types
- `port-cli`: the `port` binary and help/argument parsing layer
- `port-runtime`: host preflight, runtime layout, Firecracker config generation,
  and local launch orchestration

You can inspect the current surface with:

```bash
cargo run -p port-cli -- --help
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64
cargo run -p port-cli -- doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
```

The checked-in example config points at deterministic artifact variant paths,
store roots, and cache roots. Build or pull the sample kernel and guest image
variants first, then use the same config to run `port doctor` and
`port machine launch`.

## Current Platform Boundary

- Linux is the only platform with a shipped executable runtime lane today.
- macOS now has a first-class planned lane through Apple Virtualization
  Framework, but Port does not execute it yet.
- Windows operators are still expected to use Linux or WSL-backed workflows and
  let `port doctor` confirm whether the selected Linux lane is actually usable.

`port doctor` is the canonical entrypoint for surfacing those boundaries in the
CLI, including provider-aware cloud guidance and unsupported substrate or
protection-mode combinations.

## Development

Repository automation and planning workflow live in [AGENTS.md](AGENTS.md).
The repository development environment is defined in [flake.nix](flake.nix).

The current Rust verification command is:

```bash
cargo test
```

Common repository tasks are also available through `just` once you enter the
dev shell:

```bash
just doctor
just flow
just test
just demo-doctor
just demo-build-kernel protection=pvm architecture=x86-64
just demo-build-guest protection=pvm architecture=x86-64
```

## License

Port is available under the [MIT License](LICENSE).
