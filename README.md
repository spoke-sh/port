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
- macOS: work locally if you want, but run the canonical `port` commands on a
  Linux host because Firecracker local launch requires Linux and `/dev/kvm`.
- Windows: use WSL for the repository and CLI if helpful, but treat `port doctor`
  as the gate for whether local Firecracker launch is actually available in that
  Linux environment. Otherwise run the same `port` commands on a remote Linux
  host.

Detailed operator workflows live in [`docs/operators.md`](docs/operators.md).
Cloud-provider boundaries and the current remote Linux support matrix live in
[`docs/cloud.md`](docs/cloud.md). The Firecracker/PVM host-kit contract lives in
[`docs/pvm.md`](docs/pvm.md). The Apple Virtualization Framework contract lives
in [`docs/avf.md`](docs/avf.md).

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
| Firecracker | `pvm` | `x86_64` | Planned / partial design | Strategic lane for cloud cost control; requires dedicated host-kernel, VMM, and artifact work |
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

## AVF Contract

Port's macOS lane follows the same rule as the Linux and hosted lanes:

- keep the canonical `machine` and `guest` verbs
- map guest transport onto AVF virtio sockets
- map console/log capture onto AVF serial ports
- keep directory sharing and Rosetta as optional operator workflows, not as a
  replacement for the guest-agent protocol

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
port machine stop --machine <name> [--runtime-root <path>]
port guest exec --machine <name> -- <command...>
port guest copy --machine <name> --direction <host-to-guest|guest-to-host> --source <path> --destination <path>
port guest pty --machine <name> -- <command...>
port guest logs --machine <name> --path <path> [--tail-lines <n>] [--follow]
port guest forward --machine <name> --listen <addr> --target <addr>
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
- `port machine stop --machine <name>` stops a local Port-managed machine and
  cleans up runtime ownership details so the next launch is deterministic.
- Those same lifecycle verbs are also the groundwork for the hosted product:
  the operator model stays the same while runtime ownership moves from local
  files plus processes to a hosted control plane and node-local agent.

Current behavior:

- `port artifacts build` and `port artifacts validate` now run real in-repo
  kernel and guest-image pipelines for the selected artifact variant.
- `port artifacts push` and `port artifacts pull` now use the artifact's
  configured mobility backend. The sample config ships a file-backed
  registry/cache contract, while OCI and hosted backends are modeled but still
  reserved.
- `port doctor` performs real host checks for Linux, `/dev/kvm`, `firecracker`,
  `ip`, and `iptables`. When you pass `--config`, it also validates the native
  Firecracker/standard artifact variant paths required on the current host.
- `port machine launch` now writes a Firecracker config plus runtime metadata
  and console/log files under the chosen runtime root before invoking
  Firecracker with `--config-file`.
- `port machine list`, `port machine status`, and `port machine stop` now use
  Port-managed manifests plus live PID inspection instead of relying on the
  Firecracker REST API. Once a machine is launched, these commands operate on
  the runtime root and do not require the model file again.
- The hosted direction keeps those same lifecycle and guest verbs; the
  published contract moves long-lived runtime ownership from the short-lived
  CLI process to a node agent plus control plane rather than inventing a second
  operator model.
- `port doctor` also reports provider-aware support boundaries for
  `generic-linux`, `aws`, `gcp`, and `azure` hosts when they are present in the
  config.
- `port machine launch` still supports only local Linux launch in the MVP and
  returns provider-specific guidance for remote Linux or cloud hosts.
- `port guest exec`, `copy`, `pty`, `logs`, and `forward` now speak the shared
  guest-agent protocol through the canonical CLI and return structured results
  rendered as human-readable CLI output.

## Cloud Linux Support

Port keeps one canonical host model for local Linux and remote Linux/cloud
targets, but provider is no longer the only planning axis. Today the executable
cloud-facing lane is still Firecracker with `standard` protection on Linux
hosts. The current provider matrix for that lane is:

| Provider | Example machine | MVP status | Current command behavior |
|----------|-----------------|------------|--------------------------|
| `local` | `demo` | Supported | `port doctor` performs local preflight; `port machine launch --machine demo` can launch Firecracker on Linux |
| `generic-linux` | `cloud-generic` | Partial | `port doctor` reports the future remote Linux lane; `port machine launch` tells you to run Port on that Linux host directly |
| `aws` | `cloud-aws` | Partial | `port doctor` reports AWS as a justified future lane; `port machine launch --machine cloud-aws` fails with AWS-specific guidance |
| `gcp` | `cloud-gcp` | Partial | `port doctor` reports GCP as a justified future lane; `port machine launch` fails with GCP-specific guidance |
| `azure` | `cloud-azure` | Unsupported | `port doctor` reports Azure as unsupported for Firecracker MVP and `port machine launch` rejects it immediately |

The remote Linux workflow is intentionally limited today:

```bash
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine cloud-aws
```

The first command surfaces the provider-aware support matrix through the CLI.
The second command is expected to fail with an AWS-specific message because the
MVP does not yet implement remote launch orchestration.

The explicit cloud design, remote workflow, and substrate guidance live in
[`docs/cloud.md`](docs/cloud.md).

## Hosted Control

Port's current executable path is still local, but the canonical hosted split
is now documented in [`docs/hosted.md`](docs/hosted.md).

- The `port` CLI remains the canonical operator surface in both local and
  hosted modes.
- A future node agent owns host-local lifecycle, manifests, transport bridges,
  and artifact materialization on one execution node.
- A future control plane owns fleet inventory, routing, and durable API
  identity.
- The sample config now names that hosted API identity explicitly under
  `[control_planes.demo]`, with endpoint `https://port.example.internal`,
  audience `port-hosted-demo`, and a bearer token sourced from
  `PORT_DEMO_TOKEN`.
- Guest `exec`, `copy`, `pty`, `logs`, and `forward` keep the current guest
  protocol semantics; the hosted layer brokers them instead of replacing them.
- Local `machine list`, `status`, and `stop` now publish the control-contract
  fields that future hosted routing will reuse: inventory scope, inventory
  owner, lifecycle owner, status source, and per-verb route.

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
cargo run -p port-cli -- machine stop --machine demo
```

What that produces:

- deterministic artifact variants under `artifacts/<kind>/<name>/<architecture>/<substrate>/<protection-mode>/`
- host validation through `port doctor`
- Firecracker runtime state, logs, and manifest files under the chosen runtime root
- lifecycle inspection and stop surfaces through `port machine list`, `status`,
  and `stop`

The lifecycle commands above intentionally switch from model-driven launch to
runtime-state-driven management. After `port machine launch` creates
`runtime/<machine>/`, use `port machine list`, `port machine status --machine
<name>`, and `port machine stop --machine <name>` to inspect and control that
local Port-managed runtime state without going through Firecracker directly.
That local ownership model is also the basis for the hosted control design: the
same verbs remain canonical even when a future node agent and control plane own
the runtime on behalf of the CLI. The commands now surface that contract
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

Artifact contracts:

- `demo-kernel` fetches a pinned Firecracker-compatible kernel from the official
  Firecracker CI bucket and validates its architecture-specific sha256 digest.
  In the sample model, the native build lands at
  `artifacts/kernel/demo/<architecture>/firecracker/standard/vmlinux`.
- `demo-guest` builds a deterministic ext4 rootfs containing BusyBox userspace,
  `/init`, and the `port-guest-agent` binary. The guest init path reads
  `port.guest_control_port` from the kernel cmdline and launches the guest
  agent on that vsock port, then validates the filesystem layout with `e2fsck`
  and `debugfs`. Its native output lands at
  `artifacts/guest/demo/<architecture>/firecracker/standard/rootfs.ext4`.
- The sample config uses a file-backed store at `artifact-store/demo-fs/` and a
  cache root at `.port/cache/`. `push` writes the selected variant into that
  store and warms the cache; `pull` restores the selected variant from the
  store into both the cache and the canonical local path used by launch.
- OCI and hosted artifact backends are modeled explicitly in the shared config
  and CLI story, but the current runtime only implements the file-backed
  backend.

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

cargo run -p port-cli -- --config examples/port.toml guest logs \
  --machine demo --runtime-root /tmp/port-runtime \
  --path /var/log/port-agent.log --tail-lines 50

cargo run -p port-cli -- --config examples/port.toml guest forward \
  --machine demo --runtime-root /tmp/port-runtime \
  --listen 127.0.0.1:8080 --target 127.0.0.1:80
```

Current forward lifecycle:

- `port guest forward` is a foreground host-side proxy. The command prints the
  bound listener address, keeps serving until you interrupt it, and opens one
  guest transport connection per inbound client.
- Guest-side `--target` addresses still depend on guest networking being up.
  In the sample guest image, bring loopback up before targeting
  `127.0.0.1`, for example with
  `port guest exec --machine demo -- /bin/sh -lc 'busybox ifconfig lo up'`.

## Model And Example Config

Port keeps one canonical machine model for artifacts, hosts, and machines. The
initial sample model lives at [`examples/port.toml`](examples/port.toml).

The host model now carries explicit provider identity:

- `provider = "local"` for the supported local Linux launch lane
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

## License

Port is available under the [MIT License](LICENSE).
