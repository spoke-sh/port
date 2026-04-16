# Guest-Agent Heartbeat And Age Surface - Software Design Description

> Introduce a guest-agent heartbeat probe and per-machine guest_refresh_age_seconds so the hosted control plane can tell apart a guest-side wedge (node-agent healthy, guest-agent silent) from a node-side wedge.

**SRS:** [SRS.md](SRS.md)

## Overview

The existing `refresh_age_seconds` field (shipped in `adb97d8`) measures node-agent → control-plane heartbeat freshness. This voyage mirrors that shape one layer deeper: the **node-agent** periodically probes each of its microVM guests via `port-agent-protocol`, and publishes a per-machine `guest_refresh_age_seconds` up to the control plane through the same status path that already carries `refresh_age_seconds`.

The protocol gains a minimal `Ping`/`Pong` frame pair — small enough that adding it does not require renegotiating any existing streamed operation. The probe loop is a per-machine task inside the node-agent, not a per-operation piggyback, so a long-running `Exec` or `Pty` does not mask a wedged guest. On the node-agent side, the last successful pong stamps a monotonic `Instant` in an in-memory sidecar keyed by machine name; the status-render path computes `Instant::now().saturating_duration_since(sidecar)` exactly as the control plane does for node-agent refresh today.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

Three layers touch this change, each thin:

1. **`port-agent-protocol`** — add the `Ping`/`Pong` frame pair (or the request/response envelope equivalent) and its serde coverage. No changes to the existing streamed-operation state machine; `Ping` is a standalone control frame with no follow-on stream.
2. **`port-guest-agent`** — the guest-side agent handler loop grows a match arm for `Ping` that responds with `Pong` immediately, without interacting with any running managed service or PTY.
3. **`port-runtime` (node-agent role)** — each machine registration spawns a periodic probe task (interval tracked alongside the existing registration TTL). Successful pongs write into a `RwLock<BTreeMap<String, Instant>>` sidecar on the node-agent's state. The machine status handler reads the sidecar and includes `guest_refresh_age_seconds` in its response; the control-plane status aggregator forwards the field through unchanged.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `port-agent-protocol::StreamRequestFrame::Ping` | Wire marker requesting a liveness response. | Serialized on the existing vsock/unix control stream. |
| `port-agent-protocol::StreamResponseFrame::Pong` | Paired response proving the guest-agent read-loop is awake. | Same transport. |
| `port-guest-agent` ping handler | Immediate match arm in the agent read-loop that writes `Pong` back. | Internal; no new public API. |
| `port-runtime` guest-probe task | Per-machine timer that opens a short-lived transport, writes `Ping`, awaits `Pong` within budget, records success or drops the timestamp on failure. | Spawned once per registered machine; cancelled on deregistration. |
| Node-agent `guest_heartbeat_instants` sidecar | In-memory `RwLock<BTreeMap<String, Instant>>` mirroring the control-plane's `node_receipt_instants` pattern. | Read by the machine status handler. |
| Machine status contract | Adds `guest_refresh_age_seconds: Option<u64>` alongside the existing `refresh_age_seconds`. | JSON-serialized, skipped when `None`. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
