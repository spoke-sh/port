# Tier-1 Guest Restart And Attempt Accounting - Software Design Description

> Deliver tier-1 guest restart action and attempt accounting: when guest-side wedge trigger fires on an opted-in cluster, serialize against human lifecycle ops, stop-then-launch the machine, increment recovery_attempts.tier_1, and emit structured events. Default off.

**SRS:** [SRS.md](SRS.md)

## Overview

First action on the recovery ladder. Given a guest-side wedge flagged by the detector (voyage VGzxlScKS) on a cluster that has opted in via `recovery.enabled = true`, the node-agent that owns the wedged machine drives a stop-then-launch sequence against the same runtime root. Attempt accounting lives in the same per-machine state the detector writes to, and each transition emits a structured event so operators can correlate with Kubernetes NodeNotReady transitions.

The voyage keeps its blast radius small: it does not add tier-2 or tier-3 actions, does not define `recovery_exhausted`, and defaults `enabled = false` so a shipped release takes no action until an operator explicitly opts a cluster in.

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

Three layers:

1. **Config (`port-model`)** — `ClusterRecoveryConfig` adds `enabled: bool` and `settle_seconds: u64`. Lives alongside the detection config from voyage VGzxlScKS but is a separate block so detection and recovery can move independently.
2. **Recovery runner (`port-runtime`)** — a per-cluster task on the control plane watches the detector's `wedge_state` map. On each observed `(machine, wedge_class = "guest")` transition, it asks the owning node-agent to execute tier-1 via an existing hosted-control-plane RPC extended with a `recovery_action: Tier1Restart` payload. The node-agent takes the per-machine lifecycle lock (same lock human `port machine stop/launch` already holds; `try_lock` with immediate skip on contention), performs `machine_stop` → `machine_launch`, releases the lock, and returns a structured result. The runner updates `recovery_attempts.tier_1`, `last_recovery_action`, and `recovery_state` and emits the corresponding event.
3. **Event sink** — a small JSON-per-line writer pointed at `runtime/recovery/events.log` by default (configurable path). One writer per control plane; events include a monotonic sequence number so downstream consumers can detect gaps.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `ClusterRecoveryConfig` | Parsed `[clusters.<name>.recovery]` block with `enabled`, `settle_seconds`, and tier-1-relevant knobs. | Loaded at control-plane startup; revalidated on reload. |
| Recovery runner task | Control-plane task driving tier-1 actions based on detector output. | Reads `wedge_state`; writes `recovery_state`, `recovery_attempts`, `last_recovery_action`. |
| Per-machine lifecycle lock | Shared mutex between human `machine stop/launch` and tier-1 recovery. | `try_lock()` — recovery skips on contention, never blocks. |
| Recovery event sink | JSON-per-line writer of tier transitions. | File-backed; durable across control-plane restarts. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
