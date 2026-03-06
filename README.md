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

## Project Status

Port is early-stage and under active development. The repository now contains
the canonical Rust workspace, a working local Linux Firecracker launch path,
the first guest-agent-backed CLI workflows, and in-repo kernel plus guest-image
artifact pipelines. Cloud host support is still being implemented story by
story.

## CLI Surface

The canonical binary is `port`. The current command tree is:

```text
port doctor
port artifacts build --artifact <name>
port artifacts validate --artifact <name>
port machine launch --machine <name>
port guest exec --machine <name> -- <command...>
port guest copy --machine <name> --direction <host-to-guest|guest-to-host> --source <path> --destination <path>
port guest pty --machine <name> -- <command...>
port guest logs --machine <name> --path <path> [--tail-lines <n>] [--follow]
port guest forward --machine <name> --listen <addr> --target <addr>
```

Use `port --help` or any nested `--help` command to inspect the current command
model and examples.

Current behavior:

- `port artifacts build` and `port artifacts validate` now run real in-repo
  kernel and guest-image pipelines for the MVP sample config.
- `port doctor` performs real host checks for Linux, `/dev/kvm`, `firecracker`,
  `ip`, and `iptables`. When you pass `--config`, it also validates referenced
  artifact paths.
- `port machine launch` now writes a Firecracker config plus runtime metadata
  and console/log files under the chosen runtime root before invoking
  Firecracker with `--config-file`.
- `port guest exec`, `copy`, `pty`, `logs`, and `forward` now speak the shared
  guest-agent protocol through the canonical CLI and return structured results
  rendered as human-readable CLI output.

## Linux Local Workflow

The supported end-to-end Linux MVP workflow is:

```bash
nix develop
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
```

What that produces:

- deterministic artifacts under `artifacts/`
- host validation through `port doctor`
- Firecracker runtime state, logs, and manifest files under the chosen runtime root

The current guest-command workflow is still separate from the launched VM path:
`port guest ...` targets the runtime guest-agent socket at
`<runtime-root>/<machine>/guest-agent.sock`. The built guest image now embeds
`port-guest-agent`, but the canonical host-side guest CLI transport is still the
runtime socket workflow until the host/guest transport is unified.

## Artifact Workflow

Build the sample artifacts through the canonical CLI:

```bash
nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel
nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel
nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest
nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest
```

Artifact contracts:

- `demo-kernel` fetches a pinned Firecracker-compatible kernel from the official
  Firecracker CI bucket and validates its architecture-specific sha256 digest.
- `demo-guest` builds a deterministic ext4 rootfs containing BusyBox userspace,
  `/init`, and the `port-guest-agent` binary, then validates the filesystem
  layout with `e2fsck` and `debugfs`.

Detailed contracts, inputs, outputs, and validation expectations live in
[`docs/artifacts.md`](docs/artifacts.md).

## Guest Agent Workflow

The current MVP guest-agent transport is a Unix socket at
`<runtime-root>/<machine>/guest-agent.sock`. The `port guest ...` commands use
that path by default with `--runtime-root runtime`.

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

Current limitation:

- The guest image now embeds and launches `port-guest-agent`.
- The canonical host-side `port guest ...` commands still target the runtime
  socket workflow rather than a launched Firecracker guest transport.

## Model And Example Config

Port keeps one canonical machine model for artifacts, hosts, and machines. The
initial sample model lives at [`examples/port.toml`](examples/port.toml).

The workspace crates are:

- `port-model`: serializable artifact, host, and machine definitions
- `port-agent-protocol`: shared guest-agent request and response types
- `port-cli`: the `port` binary and help/argument parsing layer
- `port-runtime`: host preflight, runtime layout, Firecracker config generation,
  and local launch orchestration

You can inspect the current surface with:

```bash
cargo run -p port-cli -- --help
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel
cargo run -p port-cli -- doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
```

The checked-in example config points at deterministic artifact output paths.
Build the sample kernel and guest image first, then use the same config to run
`port doctor` and `port machine launch`.

## Current Platform Boundary

- Linux is the only platform expected to run Firecracker locally.
- macOS operators are expected to target Linux hosts for actual Firecracker
  execution.
- Windows operators are expected to use Linux or WSL-backed workflows and let
  `port doctor` confirm whether local launch is available in that environment.

`port doctor` is the canonical entrypoint for surfacing those support
boundaries in the CLI.

## Development

For the current development environment:

```bash
nix develop
```

Repository automation and planning workflow live in [AGENTS.md](AGENTS.md).

On Linux, `nix develop` now includes Firecracker plus the host networking tools
needed by the local launch path, along with the artifact-tooling dependencies
used by the sample kernel and guest-image pipelines.

The current Rust verification command is:

```bash
cargo test
```

## License

Port is available under the [MIT License](LICENSE).
