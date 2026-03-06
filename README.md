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

Port is being designed for Linux-based Firecracker execution and for operator
workflows from:

- Linux
- macOS
- Windows via WSL

The goal is a consistent CLI experience whether you are managing a local
machine, a lab environment, or cloud-hosted Linux capacity.

## Project Status

Port is early-stage and under active development. The repository now contains
the canonical Rust workspace, shared model, and the first public CLI surface,
but Firecracker launch, guest-agent behavior, and artifact production are still
being implemented story by story.

## CLI Surface

The canonical binary is `port`. The current command tree is:

```text
port doctor
port artifacts build --artifact <name>
port artifacts validate --artifact <name>
port machine launch --machine <name>
port guest exec --machine <name> -- <command...>
port guest copy --machine <name> --source <path> --destination <path>
port guest pty --machine <name>
port guest logs --machine <name>
port guest forward --machine <name> --listen <addr> --target <addr>
```

Use `port --help` or any nested `--help` command to inspect the current command
model and examples.

## Model And Example Config

Port keeps one canonical machine model for artifacts, hosts, and machines. The
initial sample model lives at [`examples/port.toml`](examples/port.toml).

The workspace crates are:

- `port-model`: serializable artifact, host, and machine definitions
- `port-agent-protocol`: shared guest-agent request and response types
- `port-cli`: the `port` binary and help/argument parsing layer

You can inspect the current surface with:

```bash
cargo run -p port-cli -- --help
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
```

## Current Platform Boundary

- Linux is the only platform expected to run Firecracker locally.
- macOS operators are expected to target remote Linux hosts.
- Windows operators are expected to use a Linux or WSL-backed workflow when a
  local Linux environment is required.

`port doctor` is the canonical entrypoint for surfacing those support
boundaries in the CLI.

## Development

For the current development environment:

```bash
nix develop
```

Repository automation and planning workflow live in [AGENTS.md](AGENTS.md).

The current Rust verification command is:

```bash
cargo test
```

## License

Port is available under the [MIT License](LICENSE).
