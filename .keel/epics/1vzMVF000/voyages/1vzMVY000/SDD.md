# Streamed Guest Control Transport - Software Design Description

> Deliver streamed guest shell and logs workflows plus real hosted copy and forward transport through the canonical Port surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage promotes Port's guest control path from mostly request/response
operations into a stream-capable transport that works across local runtimes and
the hosted control-plane split. The design keeps one guest command family and
one guest protocol, but extends the protocol and hosted attach path so long-
lived PTY/log sessions and hosted byte streams become real first-class flows.

## Context & Boundaries

### In Scope

- stream-capable guest protocol framing for PTY and logs
- CLI/runtime support for interactive PTY and log-follow
- hosted byte-stream transport for copy and forward
- help/docs/SDK updates for the streamed workflows

### Out of Scope

- scheduler or host-group policy
- real hosted service execution and teardown
- new substrate programs such as Cloud Hypervisor

```
┌──────────────────────────────────────────────────────────┐
│           Streamed Guest Control Transport              │
│                                                          │
│  CLI / SDK ──> runtime / hosted client ──> attach path   │
│      │                        │                  │        │
│      └──── help/docs ─────────┴──── protocol ────┘        │
└──────────────────────────────────────────────────────────┘
            ↑                             ↑
      guest session UX         control-plane / node-agent relay
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-agent-protocol` | internal | shared guest message framing | current guest protocol crate |
| `port-sdk` and `port-hosted-protocol` | internal | hosted route construction and typed client surface | current hosted HTTP contract |
| existing control-plane / node-agent serve paths | internal | hosted ownership and auth boundaries | current runtime HTTP servers |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Guest session model | extend the existing guest protocol with explicit streaming lifecycle semantics instead of adding a shell-specific side channel | preserves one guest API and one CLI vocabulary |
| Hosted attach path | keep the control plane as the route resolver and auth boundary, but make the node agent own streamed byte transport | aligns with current hosted lifecycle ownership |
| CLI behavior | make `guest pty` and `guest logs --follow` truly streamed while preserving completed-output behavior for non-follow logs and non-PTY exec | improves operator UX without changing command names |
| Hosted copy / forward | remove node-host path and repo-local listener assumptions from hosted flows in the same slice | closes the biggest hosted-product transport gap directly |

## Architecture

The voyage touches four layers:

1. guest protocol framing and guest-agent stream lifecycle
2. local runtime stream adapters for PTY and log-follow
3. hosted control-plane and node-agent streaming transport for copy and forward
4. CLI, SDK, and documentation updates for discoverability and proof

## Components

### Streamed Guest Protocol

- Purpose: define attach, payload, EOF, exit, and failure semantics for streamed
  PTY and logs sessions.
- Interface: `RequestEnvelope` / `ResponseEnvelope` plus stream-oriented payload
  handling in `port-agent-protocol`.
- Behavior: keep `exec`, `copy`, `pty`, `logs`, and `forward` as the only
  operator verbs while allowing PTY and logs-follow to remain open.

### Guest Agent Stream Handlers

- Purpose: make PTY and log-follow long-lived guest operations real instead of
  transcript snapshots.
- Interface: `port-guest-agent` request handlers.
- Behavior: emit incremental output, close deterministically, and surface exit
  or failure state explicitly.

### Runtime Stream Adapter

- Purpose: bridge the shared guest protocol onto local sockets, AVF guest
  sockets, and hosted attach paths.
- Interface: runtime guest-operation helpers and CLI plumbing.
- Behavior: present the same canonical CLI surface across local and hosted
  ownership while choosing the correct transport owner underneath.

### Hosted Stream Relay

- Purpose: make hosted copy and forward use real byte streams through node-owned
  transport instead of bootstrap assumptions.
- Interface: hosted protocol routes, control-plane resolution, node-agent guest
  transport handlers, and SDK request builders.
- Behavior: the control plane resolves route ownership; the node agent handles
  the actual streamed bytes and returns explicit route context.

### Operator Surface

- Purpose: keep the streamed workflows discoverable and auditable.
- Interface: `port --help`, README, `docs/hosted.md`, `docs/sdk.md`, and proof
  scripts or tapes.
- Behavior: publish how to use streamed PTY/log-follow and the hosted copy/
  forward boundaries without inventing separate command families.

## Interfaces

- extended `port-agent-protocol` framing for streamed PTY and logs
- runtime helpers for long-lived guest stream handling
- hosted control-plane and node-agent routes for streamed guest transport
- `port-sdk` request constructors for streamed hosted guest operations
- canonical CLI help and command behavior for streamed guest workflows

## Data Flow

1. The operator starts `port guest pty` or `port guest logs --follow`.
2. Port resolves the machine's runtime owner and guest attach contract.
3. For local lanes, Port opens the guest transport directly and maintains the
   stream lifecycle until EOF or exit.
4. For hosted lanes, the control plane resolves route ownership and the node
   agent owns the long-lived byte stream or hosted transport session.
5. CLI and SDK surfaces receive incremental payloads plus explicit completion or
   failure state.

Hosted copy / forward follow the same ownership rule:

1. The CLI or SDK resolves the hosted route through the control plane.
2. The node agent opens the guest-side stream and relays bytes.
3. Port surfaces explicit route context, ownership, and cleanup detail to the
   operator.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Stream handshake cannot upgrade into a long-lived session | protocol / runtime adapter | fail fast with attach-specific detail | inspect route owner and retry after fix |
| PTY or log-follow stream ends without explicit completion | guest agent / runtime adapter | surface abnormal EOF to CLI and SDK | relaunch session and inspect guest agent |
| Hosted copy still depends on unresolved node-host paths | control-plane / node-agent validation | return explicit hosted transport error | switch to the streamed hosted path or fix node state |
| Hosted forward cannot bind or relay under node ownership | node-agent stream relay | surface listener or relay-specific detail | inspect node runtime state and retry |
| Existing local or substrate lane regresses while streaming lands | automated tests / proofs | block story completion | repair regression before advancing |
