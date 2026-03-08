# Implement Hosted Control Plane Demo Lane - Software Design Description

> Deliver a single-node hosted control plane and node-agent demo that executes
> canonical machine and guest verbs over authenticated HTTP transport.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns Port's hosted design into a live runnable lane by adding two
long-lived processes:

- a control-plane HTTP server that authenticates clients, resolves hosted
  inventory, and routes requests
- a node-agent HTTP server that owns one node runtime root and performs the
  actual machine or guest work

The CLI and SDK remain the public entry points. When a machine resolves to
`hosted-control-plane` mode, they stop inspecting runtime roots directly and
instead call the hosted control plane. The control plane forwards the work to
the correct node agent using the already-modeled node inventory. The node agent
then reuses existing runtime and guest transport logic rather than inventing a
second execution stack.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────────────┐
│                          This Voyage                               │
│                                                                     │
│   ┌──────────────┐      HTTP + bearer auth      ┌──────────────┐   │
│   │  port CLI /  │ ───────────────────────────▶ │ control plane │   │
│   │  port-sdk    │                              │    server     │   │
│   └──────────────┘                              └──────┬───────┘   │
│                                                        │            │
│                                   HTTP + node token    │            │
│                                                        ▼            │
│                                                  ┌──────────────┐   │
│                                                  │  node agent  │   │
│                                                  │    server    │   │
│                                                  └──────┬───────┘   │
│                                                         │            │
│                                               existing runtime +     │
│                                               guest attach logic     │
└─────────────────────────────────────────────────────────────────────┘
               ↑                                   ↑
        local Firecracker                     runtime root,
        execution lane                        guest sockets, logs
```

Out of scope for this voyage:

- multi-node scheduling policy
- PVM host-kit execution
- AVF execution
- full service runtime execution

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | Internal crate | Hosted control-plane, node, and machine identity contracts | workspace |
| `port-runtime` | Internal crate | Existing machine inspection, stop, guest attach, and forward logic to reuse inside the node agent | workspace |
| `port-agent-protocol` | Internal crate | Canonical guest operation payloads | workspace |
| `tokio` | Library | Async server/client execution | workspace / existing |
| HTTP server/client crate(s) | Library | Implement control-plane and node-agent HTTP transport | selected during implementation |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Hosted demo topology | Single control plane plus single node agent first | Proves the ownership split before multi-node scheduling complexity |
| Public operator surface | Keep `port machine ...` and `port guest ...` unchanged | Preserves the product model already established locally |
| Guest payload shape | Reuse `port-agent-protocol` over HTTP bodies | Avoids a second guest command model |
| Runtime ownership | Node agent owns runtime roots and live process inspection | Matches the documented hosted ownership split |
| Auth model | Bearer-token auth for control-plane clients and explicit node token for control-plane to node-agent calls | Good enough for the first live lane without pretending RBAC is solved |
| Transport scope | Request/response HTTP first, with explicit notes where streaming fidelity is partial | Gets the live hosted lane running before websocket or multiplexed streaming work |

## Architecture

The voyage adds three implementation layers:

1. Shared hosted transport contracts
   - route types, request/response bodies, auth/header rules
   - shared between CLI, SDK, control plane, and node agent

2. Hosted daemon layer
   - `port control-plane serve`
   - `port node-agent serve`
   - inventory resolution, auth checks, request forwarding

3. Client routing layer
   - hosted machine and guest operations use HTTP when the resolved machine
     targets a control plane
   - local mode continues to use direct runtime ownership

## Components

### Shared Hosted HTTP Contract

Purpose:

- define canonical route and body shapes for live hosted execution
- keep the CLI, SDK, and servers aligned

Behavior:

- machine routes mirror the existing SDK path contract
- guest routes accept existing `GuestOperation` payloads
- responses return structured machine or guest results plus route context

### Control Plane Server

Purpose:

- authenticate CLI and SDK requests
- resolve which node owns a hosted machine
- proxy machine and guest work to the correct node agent

Behavior:

- serves `/v1/machines`, `/v1/machines/{machine}`, `/monitor`, `/top`, `:stop`
- serves `/v1/machines/{machine}/guest:*`
- validates hosted control-plane token
- looks up node endpoint plus runtime ownership from config-backed inventory

### Node Agent Server

Purpose:

- own one node runtime root
- inspect or stop machine state
- attach to guest transport and execute guest operations

Behavior:

- exposes internal machine and guest routes for the control plane
- reuses existing runtime-root and guest attach logic
- returns explicit runtime-root, machine, and node context on failures

### CLI / SDK Hosted Client Path

Purpose:

- keep the operator-facing verbs stable while changing transport

Behavior:

- local machines still call direct runtime logic
- hosted machines build and execute HTTP requests against the control plane
- SDK remains the typed request constructor and may gain a minimal execution
  helper for the demo lane if needed

## Interfaces

### CLI Commands

- `port control-plane serve --bind <addr>`
- `port node-agent serve --node <name> --bind <addr>`
- existing `port machine ...` and `port guest ...` commands gain live hosted
  transport when the target machine resolves to hosted mode

### Control Plane HTTP

- `GET /v1/machines`
- `GET /v1/machines/{machine}`
- `GET /v1/machines/{machine}/monitor`
- `GET /v1/machines/{machine}/top`
- `POST /v1/machines/{machine}:stop`
- `POST /v1/machines/{machine}/guest:exec|copy|pty|logs|forward`

Auth:

- client bearer token in the configured header from the hosted identity
  contract

### Node Agent HTTP

Internal routes mirror the machine and guest actions the control plane needs.
They are not a second public API product; they are a control-plane-to-node
contract.

Auth:

- explicit node-agent token configured for the demo lane

## Data Flow

1. Operator starts `port node-agent serve` for a configured node.
2. Operator starts `port control-plane serve`.
3. Operator runs a hosted `port machine` or `port guest` command.
4. CLI resolves the machine to hosted mode and sends an authenticated HTTP
   request to the control plane.
5. Control plane resolves the machine to its candidate node and forwards the
   request to that node agent.
6. Node agent uses existing runtime or guest transport logic against its
   runtime root.
7. Result flows back through the control plane to the CLI or SDK caller.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Client token missing or invalid | Control-plane auth check | Return auth failure with configured header and control-plane context | Operator supplies the correct token source |
| Hosted machine has no resolvable node | Inventory lookup fails | Return malformed or unavailable machine state with node and host-group context | Fix config or node-agent registration |
| Node agent unavailable | Control-plane forward fails | Return control-plane route plus node endpoint failure | Start or repair the node agent |
| Runtime root lacks machine state | Node agent cannot resolve runtime owner | Return machine and runtime-root context in the error | Launch or repair the target machine |
| Guest socket missing | Existing guest attach fails | Return node, runtime-root, and guest-socket path | Repair guest runtime or relaunch the machine |
| Partial streaming support for PTY or forward | Route reaches an unsupported fidelity path | Return explicit “not yet streamed” guidance instead of silent truncation | Implement richer streaming in a follow-on voyage |
