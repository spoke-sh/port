# Port

Port is a CLI-first system for building, launching, and operating isolated
Linux workloads in microVMs across local and hosted environments.

It keeps one operator vocabulary across lanes:

- `port doctor` for host and lane checks
- `port artifacts` for build, validate, push, and pull
- `port machine` for lifecycle and status
- `port guest` for exec, copy, PTY, logs, and forward
- `port service` for secrets, services, and sandboxes

## Current Shape

- Default local lane: Firecracker with `standard` protection on Linux
- Hosted lane: control plane plus node agent with the same `machine`, `guest`,
  and `service` verbs
- Additional proof-backed lanes: Cloud Hypervisor `standard`, AVF `standard`,
  and prepared-node Firecracker/PVM on `x86_64`

## Quick Start

```bash
port doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
port --config examples/port.toml machine launch --machine demo
```

Use `examples/port.toml` for the checked-in repo workflow. Detailed config
edits and longer examples now live in [`CONFIGURATION.md`](CONFIGURATION.md).

## Mission Report

```bash
just mission
```

That shows a compact mission report with board-backed goal status, recent
achievements, and a human-facing artifact gallery for the active mission.

## Documentation Map

### Root Contracts

| Document | Purpose |
|----------|---------|
| [`CONSTITUTION.md`](CONSTITUTION.md) | Non-negotiable product and workflow rules |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | System boundaries, ownership, and major components |
| [`CONFIGURATION.md`](CONFIGURATION.md) | Config model, environment variables, and detailed workflow examples |
| [`RELEASE.md`](RELEASE.md) | Current release contract and validation checklist |
| [`EVALUATIONS.md`](EVALUATIONS.md) | Verification and evidence expectations |
| [`AGENTS.md`](AGENTS.md) | Shared AI-agent workflow contract |

### Focused Guides

| Document | Purpose |
|----------|---------|
| [`docs/operators.md`](docs/operators.md) | Operator-oriented overview and platform guidance |
| [`docs/hosted.md`](docs/hosted.md) | Hosted control-plane, node-agent, and service workflows |
| [`docs/cloud.md`](docs/cloud.md) | Cloud-provider and hosted-lane boundaries |
| [`docs/artifacts.md`](docs/artifacts.md) | Artifact references, variants, and backends |
| [`docs/pvm.md`](docs/pvm.md) | Firecracker/PVM host-kit and artifact-kit contract |
| [`docs/avf.md`](docs/avf.md) | Apple Virtualization Framework lane |
| [`docs/sdk.md`](docs/sdk.md) | Hosted SDK and typed client surface |

## Platform Summary

| Environment | What Port supports today |
|-------------|--------------------------|
| Linux | Full local Firecracker workflow plus hosted control-plane demos |
| macOS | AVF local workflow through the canonical `machine` and `guest` verbs |
| Windows | Linux-backed workflow through WSL or a remote Linux host |

Use [`docs/operators.md`](docs/operators.md) for the platform guide and
[`CONFIGURATION.md`](CONFIGURATION.md) for the detailed configuration and
workflow examples.
