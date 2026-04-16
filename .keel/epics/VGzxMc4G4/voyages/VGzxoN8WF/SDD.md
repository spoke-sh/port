# Recovery Exhaustion Reset And End-To-End Proof - Software Design Description

> Deliver the sticky recovery_exhausted terminal state, the port machine unfence reset path, and auto-clear on a successful operator-driven launch that produces a Live guest-agent heartbeat. Cover the end-to-end ladder with an integration test that converges a simulated wedge under tier-1 and another under tier-3.

**SRS:** [SRS.md](SRS.md)

## Overview

The ladder needs a deliberate terminal state and a way out of it. `recovery_exhausted` is sticky across window rollovers so a permanently broken machine does not cycle tier-1 → tier-1 → ... forever; operators resolve it either by running `port machine unfence --machine X` or by performing a successful operator-driven launch that produces a Live guest-agent heartbeat (recognising that most manual fixes end in a launch anyway, so requiring a separate unfence step would be friction).

This voyage also lands the end-to-end proof: three integration tests exercising the full ladder against simulated wedges so the mission can HALT with concrete evidence. Tests use injectable clocks and event hooks so CI does not depend on wall-clock `sleep`.

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

Three pieces:

1. **Exhausted state (`port-runtime::hosted_control_plane`)** — `recovery_state = "exhausted"` lives in the same per-machine recovery record the runner already owns, but is persisted to disk alongside `registered_node_state.json` so it survives control-plane restarts. The recovery runner's state machine grows a terminal transition: on tier-3 completion without convergence (or suppression with no further tiers), it writes `exhausted` and then stops acting on the machine even as window rollovers reset counters.
2. **`port machine unfence` CLI + handler** — a new subcommand under `port machine`, routed to the control plane via a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. The handler clears the exhausted flag, resets counters, and emits `recovery_unfenced`. The command is explicitly NOT an alias for `launch` — it makes no runtime changes, only state changes. Docs make this clear.
3. **Auto-clear on successful launch** — the existing `launch` path grows a post-launch hook that, if the machine was in `exhausted`, waits up to a documented budget for a Live guest-agent heartbeat; if one arrives, clear `exhausted` and emit `recovery_unfenced_via_launch`. If the launch produces no heartbeat within the budget, the state stays as-is.

Integration tests use `tokio::time::pause` / injectable clocks plus channel-based event hooks on the runner so they observe the ladder deterministically.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| Persisted `recovery_exhausted` flag | Sticky terminal state surviving restarts. | Stored in `runtime/recovery/<machine>.json` alongside existing registered-node state. |
| `port machine unfence` command | Operator reset. | `POST /v1/machines/{machine}/recovery:unfence` on the control plane. |
| Post-launch auto-clear hook | Convenience reset when the operator already fixed it. | Fires from the existing `launch` success path. |
| End-to-end test harness | Drives simulated wedges and asserts ladder behaviour without wall-clock sleeps. | Uses injectable clocks and fake `HostRebootClient`. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
