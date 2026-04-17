# Tier-2 Overlay Recreate And Tier-3 Escalation Signal - Software Design Description

> Deliver tier-2 overlay recreate (Port owns the action) and a tier-3 escalation signal (Port emits, external consumer acts). Port never calls cloud-provider APIs and never shells out over SSH as part of recovery.

**SRS:** [SRS.md](SRS.md)

## Overview

Tier-2 is the last action Port takes. It's a surgical "force a clean filesystem" step: drop the rootfs overlay and relaunch; graceful skip for machines not using an overlay. When tier-1 and tier-2 both exhaust, Port escalates by *signaling* rather than acting — `recovery_state` flips to `awaiting_tier_3_host_recycle` and a structured `tier_3_escalation` event is emitted. The consumer of the signal (spoke-sh/infra, an operator on call, a systemd unit watching the event log) decides whether and how to recycle the host.

This split keeps cloud-provider credentials and SSH sessions out of Port entirely. When the host comes back, Port notices automatically via node-agent re-registration plus a fresh guest heartbeat, transitions `recovery_state` back to `ok`, and emits `tier_3_host_returned`. No response path from the consumer to Port is required.

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

Three logical pieces, all on the Port side:

1. **Promotion logic** — the runner's existing per-machine state machine gains two more transitions: `tier_1 → tier_2` (on `recovery_attempts.tier_1 >= tier_2_after_attempts` within `window_seconds`) and `tier_2 → tier_3` (on cumulative attempts reaching `tier_3_after_attempts`). The tier-3 transition sets `recovery_state = "awaiting_tier_3_host_recycle"` and emits a structured `tier_3_escalation` event — nothing else.
2. **Tier-2 executor** — a thin helper on the node-agent side that removes `runtime/<machine>/overlay` (idempotent: missing overlay is not an error) and invokes the existing launch path. The filesystem operation is isolated so it can be unit-tested against a fake runtime root.
3. **Tier-3 auto-clear observer** — the existing detector already clears `wedged_since` when node-agent and guest heartbeats return to fresh. This voyage extends that transition so it also flips `recovery_state` from `awaiting_tier_3_host_recycle` back to `ok`, emits `tier_3_host_returned`, and resets `recovery_attempts` to zero. No host-level lock is needed because Port takes no concurrent action against the host.

**Boundary check:** a CI-level test inspects the `port-runtime` recovery code path and fails the build if it finds any `aws-sdk-*`, `reqwest` call to a cloud provider endpoint, or remote shell execution (`Command::new("ssh")`, `openssh`, `russh`) introduced by recovery code. This is the machine-checkable form of the "no cloud logic inside Port" rule.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| Recovery runner promotion | State machine transitions `tier_1 → 2`, `tier_2 → 3`. | Reads `recovery_attempts` and `window_seconds`; writes `last_recovery_action`, `recovery_state`. |
| Tier-2 overlay executor | Drops `runtime/<machine>/overlay` then relaunches. Idempotent. | Node-agent-local; no new external API. |
| Tier-3 escalation signaller | Sets `recovery_state = "awaiting_tier_3_host_recycle"` and emits `tier_3_escalation` event. Takes no action on host or machine. | Pure state mutation + event emission. |
| Tier-3 auto-clear observer | When node-agent re-registration + fresh guest heartbeat observed, flips `recovery_state` back to `ok` and emits `tier_3_host_returned`. | Reads existing heartbeat sidecars; writes recovery state. |
| Recovery-code boundary test | CI guard that no cloud SDK or SSH invocation enters the recovery code path. | Static scan — fails the build on violation. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
