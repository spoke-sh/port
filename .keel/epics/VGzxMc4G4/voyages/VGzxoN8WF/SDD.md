# Tier-3 Signal Persistence, Unfence Reset, And End-To-End Proof - Software Design Description

> Persist `awaiting_tier_3_host_recycle` across control-plane restarts; land `port machine unfence` as the manual operator reset; auto-clear on a successful operator-driven launch that produces a Live guest-agent heartbeat. Cover the full ladder with deterministic end-to-end tests that observe the tier-3 signal rather than driving any cloud action.

**SRS:** [SRS.md](SRS.md)

## Overview

Once the ladder escalates to `awaiting_tier_3_host_recycle` (set by voyage VGzxnR97R), the machine sits in that state waiting for an external consumer to recycle the host. Port's job at that point is purely to (1) persist the state so a control-plane restart doesn't re-arm the ladder, (2) offer humans a clean manual reset via `port machine unfence`, and (3) opportunistically auto-clear when an operator-driven launch produces a fresh heartbeat — which covers the common "operator SSHed the host, rebooted it, then relaunched the machine" path.

The end-to-end proof drives simulated wedges through the whole ladder. The tier-3 test observes the `tier_3_escalation` event and then simulates host return via fresh heartbeats — no fake cloud clients, no fake reboot calls. That directly reflects the no-cloud-in-Port boundary.

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

1. **Persisted recovery state (`port-runtime::hosted_control_plane`)** — each machine's `recovery_state` and `recovery_attempts` are serialised to `runtime/recovery/<machine>.json` alongside existing registered-node state. The runner loads this file on startup so a mid-escalation restart does not re-arm tier-1 against a machine already in `awaiting_tier_3_host_recycle`.
2. **`port machine unfence` CLI + handler** — a new subcommand under `port machine`, routed to the control plane via a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. The handler clears any non-`ok` `recovery_state`, resets counters, and emits `recovery_unfenced`. The command is explicitly NOT an alias for `launch` — it makes no runtime changes, only state changes. Docs make this clear.
3. **Auto-clear on successful launch** — the existing `launch` path grows a post-launch hook that, if the machine was in `awaiting_tier_3_host_recycle`, waits up to a documented budget for a Live guest-agent heartbeat; if one arrives, clear the state and emit `recovery_unfenced_via_launch`. If the launch produces no heartbeat within the budget, the state stays as-is.

Integration tests use `tokio::time::pause` / injectable clocks plus channel-based event hooks on the runner so they observe the ladder deterministically. The tier-3 path observes the emitted `tier_3_escalation` event and fakes host return by delivering fresh heartbeats — no fake cloud client is involved.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| Persisted recovery state | Per-machine `recovery_state` + `recovery_attempts` survive restarts. | Stored in `runtime/recovery/<machine>.json`. |
| `port machine unfence` command | Operator reset for any non-`ok` `recovery_state`. | `POST /v1/machines/{machine}/recovery:unfence` on the control plane. |
| Post-launch auto-clear hook | Convenience reset when the operator already fixed the host and relaunched. | Fires from the existing `launch` success path. |
| End-to-end test harness | Drives simulated wedges and asserts ladder behaviour without wall-clock sleeps. | Uses injectable clocks and channel-based event hooks — no cloud fakes. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
