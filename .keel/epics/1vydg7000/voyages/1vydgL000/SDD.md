# Local Linux CLI Runtime - Software Design Description

> Deliver a coherent local Linux Firecracker workflow through the Port CLI, including artifact contracts, launch orchestration, guest agent reachability, and operator-facing documentation for the first MVP path.

**SRS:** [SRS.md](SRS.md)

## Overview

Port will be implemented as a Rust workspace with a shared domain model and two
execution surfaces:

- a host-side CLI/runtime that validates the environment, resolves artifacts,
  launches Firecracker, and proxies guest operations; and
- a guest-side agent that boots inside the VM, speaks a narrow RPC protocol over
  vsock, and provides `exec`, `copy`, `pty`, `logs`, and `forward`.

The host remains the control plane. The guest agent remains the data-plane
entrypoint for in-guest actions. Artifact generation is treated as product code,
not external setup.

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                         Port Host CLI                         │
│                                                               │
│  clap commands  ->  model loader  ->  host preflight         │
│                                    ->  artifact resolver      │
│                                    ->  firecracker runtime    │
│                                    ->  guest transport proxy  │
└───────────────────────────────────────────────────────────────┘
                │                       │
                │ state/logs            │ vsock
                ▼                       ▼
        host filesystem          ┌──────────────────┐
                                 │ Port Guest Agent │
                                 │ exec/copy/pty    │
                                 │ logs/forward     │
                                 └──────────────────┘
                                          │
                                          ▼
                                    guest userspace
```

Out of scope for this voyage: cloud control-plane orchestration beyond keeping
the model extensible and the docs explicit about what is currently local-only.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Firecracker | Runtime | Local microVM execution | Nix package `firecracker` 1.14.x |
| Linux KVM | Host capability | Hardware virtualization for local MVP path | `/dev/kvm` |
| Vsock | Guest transport | Host/guest RPC without requiring SSH in the guest | Linux AF_VSOCK |
| `iproute2` and firewall tooling | Host tooling | TAP, NAT, and forwarding setup | Host package set |
| Rust workspace | Implementation | Shared model, host CLI, guest agent, and tests | Stable Rust 1.94+ |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical control surface | `port` CLI backed by serializable Rust domain types | Keeps operator and automation semantics aligned |
| Guest transport | Vsock RPC between host and guest agent | Avoids requiring a second remote-control stack like SSH for MVP |
| Guest image shape | Minimal Linux userspace with BusyBox init and the Port guest agent | Small enough for reproducible artifact generation while still supporting shell-like operations |
| Artifact ownership | Kernel and guest-image build/validation live in repo and are invoked by Port-aware commands/scripts | Artifact production is an MVP requirement, not an external prerequisite |
| Platform contract | Only Linux runs Firecracker locally; macOS and Windows target Linux hosts and are documented as operator workflows | Matches Firecracker/KVM constraints and prevents misleading local promises |

## Architecture

The workspace is split into coarse modules:

- `port-model`: serializable host, artifact, and machine definitions plus CLI
  translation helpers.
- `port-cli`: the canonical `port` binary with subcommands for `doctor`,
  `artifacts`, `machine`, and `guest`.
- `port-runtime`: local host preflight, Firecracker process management, runtime
  directories, networking, and state persistence.
- `port-agent-protocol`: the shared request/response protocol, message framing,
  and tests.
- `port-guest-agent`: the guest binary that services RPC requests inside the VM.

## Components

- `doctor` command:
  validates platform support, checks `/dev/kvm`, required binaries, and
  artifact readiness. This command is also the entrypoint for macOS/Windows
  users to learn why local launch is unsupported.
- Artifact pipeline:
  produces a bootable kernel artifact and guest-image artifact, then validates
  them before runtime launch. Outputs are declared in the model and docs.
- Runtime launcher:
  prepares runtime directories, writes Firecracker config, creates a TAP device,
  starts Firecracker, and records state paths and logs.
- Guest proxy:
  translates CLI requests into protocol messages over vsock and handles stream
  bridging for PTY and forwarding.
- Guest agent:
  executes commands, manages file transfer, allocates PTYs, tails log files, and
  proxies TCP connections from guest to host-side listeners.

## Interfaces

- Model files:
  TOML or JSON representations of artifacts, machines, and host profiles. One
  canonical schema is kept in code and docs.
- CLI surface:
  `port doctor`
  `port artifacts build`
  `port artifacts validate`
  `port machine launch`
  `port guest exec|copy|pty|logs|forward`
- Guest protocol:
  request/response RPC over vsock with streaming channels for PTY, logs, file
  copy, and forwarding.

## Data Flow

1. Operator invokes a `port` command.
2. CLI loads model/config, validates arguments, and optionally runs host
   preflight.
3. Artifact resolution either confirms existing artifacts or triggers the build
   pipeline.
4. Runtime launcher writes Firecracker config, prepares networking/state, and
   starts the VM.
5. Once the guest agent is reachable on vsock, guest subcommands exchange
   protocol messages for `exec`, `copy`, `pty`, `logs`, or `forward`.
6. Runtime metadata and logs are persisted so later CLI commands and docs can
   point operators to the same canonical state locations.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Host is not Linux or `/dev/kvm` is unavailable | Preflight checks before launch | Return a clear unsupported-host error | Operator fixes host or uses a remote Linux workflow |
| Required tool or artifact is missing | Preflight or artifact validation | Print actionable remediation with exact command | Run documented build/install command |
| Firecracker boot fails | Runtime process exits or API setup fails | Persist logs and return error with log paths | Inspect logs, rebuild artifacts, retry |
| Guest agent never becomes reachable | Vsock handshake timeout | Return timeout with console/log references | Inspect guest boot logs or artifacts |
| Guest command/stream fails mid-flight | Protocol error or EOF | Surface command-specific failure and preserve partial logs/state | Retry or relaunch depending on operation |
