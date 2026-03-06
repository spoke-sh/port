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

Port is early-stage and under active development. Interfaces, deployment
patterns, and runtime details are expected to change as the core architecture
lands.

## Development

For the current development environment:

```bash
nix develop
```

Repository automation and planning workflow live in [AGENTS.md](AGENTS.md).

## License

Port is available under the [MIT License](LICENSE).
