# Hosted Detached Forward Lifecycle - Software Design Description

> Finish hosted guest forward with node-owned list, stop, name, and detached lifecycle semantics through the control plane and node agent.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage finishes hosted guest forward by moving the rest of the detached
lifecycle onto the same live hosted runtime path already used for hosted
machine verbs and streamed guest operations.

The key design choice is to keep detached forward state where it already
belongs: under the runtime-owning node's runtime root. The control plane gains
new request and response routes for detached start, list, and stop, but it does
not become a second forward registry. The node agent localizes the hosted
machine, reuses the existing detached manifest shape, and performs the actual
start or stop action against node-owned runtime state.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────────────┐
│                          This Voyage                               │
│                                                                     │
│   ┌──────────────┐      hosted HTTP routes      ┌──────────────┐   │
│   │  port CLI /  │ ───────────────────────────▶ │ control plane │   │
│   │  port-sdk    │                              │    server     │   │
│   └──────────────┘                              └──────┬───────┘   │
│                                                        │            │
│                                           node HTTP    │            │
│                                           forward API  ▼            │
│                                                  ┌──────────────┐   │
│                                                  │  node agent  │   │
│                                                  │    server    │   │
│                                                  └──────┬───────┘   │
│                                                         │            │
│                                        runtime root / forwards/*.json│
│                                        + detached daemon processes   │
└─────────────────────────────────────────────────────────────────────┘
               ↑                                   ↑
         hosted auth + routing              existing manifest model,
                                            forward daemon, monitor/top
```

Out of scope:

- new scheduler policy
- general service/sandbox background process management
- a second control-plane-owned forward registry

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-hosted-protocol` | Internal crate | Shared hosted detached-forward route and payload contract | workspace |
| `port-runtime` | Internal crate | Runtime-root manifest loading, forward session prep, and hosted transport execution | workspace |
| `port-cli` | Internal crate | Canonical `guest forward` operator surface | workspace |
| `port-sdk` | Internal crate | Hosted client request builders for detached lifecycle routes | workspace |
| Existing detached manifest files | Runtime contract | Source of truth for forward inventory and stop actions | runtime `<machine>/forwards/*.json` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Detached session state owner | Keep node runtime root as the only source of truth | Avoids inventing a second registry and preserves monitor/top integration |
| Hosted transport shape | Add explicit detached forward routes instead of overloading the streamed start path | List and stop are lifecycle operations, not byte streams |
| Naming model | Reuse the existing `--name` manifest identity end to end | Preserves CLI discoverability and makes stop deterministic |
| Start implementation | Node agent launches the detached daemon using localized config/runtime ownership | Reuses the current forward daemon and manifest shape |
| Operator model | Extend `port guest forward` rather than adding hosted-only verbs | Keeps one product surface across local and hosted modes |

## Architecture

The voyage adds three layers:

1. Shared detached-forward contracts
   - hosted route types for start, list, and stop
   - result payloads for manifest summaries and stop acknowledgements

2. Hosted server behavior
   - control plane authenticates and forwards detached forward actions
   - node agent localizes the machine and operates on node-owned forward state

3. Client routing and docs
   - CLI and SDK send hosted detached forward lifecycle requests remotely
   - help text and docs describe detached start/list/stop semantics explicitly

## Components

### Shared Detached Forward Contract

Purpose:

- define the hosted request/response shape for detached forward lifecycle
  operations

Behavior:

- start returns the detached forward manifest summary
- list returns stable per-forward inventory sorted by name
- stop returns the stopped forward identity and state outcome

### Control Plane Detached Forward Routes

Purpose:

- authenticate hosted clients and route detached forward lifecycle actions to
  the owning node agent

Behavior:

- resolve the hosted machine to a node and runtime owner
- proxy detached forward start/list/stop requests
- preserve route context on success and failure

### Node Agent Detached Forward Owner

Purpose:

- perform detached forward lifecycle actions against node-owned runtime state

Behavior:

- localize the hosted config to the owning node
- start detached forward daemons with the existing manifest shape
- load manifest inventory from the machine forward state directory
- stop named forward daemons and clean up manifest/socket state

### CLI / SDK Hosted Forward Lifecycle

Purpose:

- keep hosted detached forward lifecycle discoverable through the canonical
  Port surfaces

Behavior:

- local machines keep using direct runtime ownership
- hosted machines route detached lifecycle actions through the hosted control
  plane
- operator-facing help and docs describe both the start path and lifecycle
  management path

## Interfaces

### CLI

- `port guest forward --machine <name> --listen ... --target ... --lifecycle detached [--name ...]`
- `port guest forward --machine <name> --list`
- `port guest forward --machine <name> --stop --name <forward>`

### Hosted Control Plane / Node Routes

- detached start route returning a manifest summary
- detached list route returning forward inventory for one machine
- detached stop route targeting one named forward

The exact route vocabulary should live in `port-hosted-protocol` so the CLI,
SDK, control plane, and node agent stay aligned.

## Data Flow

Detached start:

1. Operator runs hosted `port guest forward ... --lifecycle detached`.
2. CLI resolves the machine to hosted mode and sends the detached start request
   to the control plane.
3. Control plane authenticates, resolves the node, and forwards the request to
   the node agent.
4. Node agent starts the detached forward daemon under the node-owned runtime
   root and waits for the manifest.
5. Manifest details flow back through the control plane to the CLI.

Detached list / stop:

1. Operator runs hosted `--list` or `--stop --name <forward>`.
2. CLI sends the lifecycle request to the control plane.
3. Control plane forwards it to the owning node agent.
4. Node agent reads or mutates the runtime-root manifest state.
5. Result flows back with route context and machine/node ownership detail.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted machine has no selected node | Control-plane placement resolution fails | Return route context with control plane, machine, and candidate node detail | Fix hosted config or node availability |
| Named detached forward does not exist | Node agent cannot read the requested manifest | Return machine, node, runtime-root, and forward-name context | Use `--list` or repair the node runtime state |
| Detached daemon fails to start | Node agent does not observe a valid manifest or process start | Return start failure with node/runtime information | Inspect node logs and correct the listen/target/runtime issue |
| Stop cleanup cannot remove Unix socket or manifest | Node agent cleanup step fails | Return explicit cleanup failure with path context | Remove stale runtime artifacts and retry |
