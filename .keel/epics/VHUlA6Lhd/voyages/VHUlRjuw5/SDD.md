# Hosted Guest Recovery Fidelity - Software Design Description

> Make hosted guest recovery safe for production by restarting unhealthy K3s services, removing false guest-wedge inference, and surfacing trustworthy wedge evidence for downstream automation.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage hardens the hosted recovery loop in three places. First, the hosted K3s managed-service policy will opt into unhealthy restarts so the guest agent can restart a dead or wedged `k3s-agent`/`k3s-server` even when the process record still looks live. Second, hosted wedge classification will stop synthesizing guest-heartbeat age from machine placement age and will instead rely on real heartbeat age or explicit managed-runtime failure evidence. Third, the machine wedge endpoint will export that evidence directly so operators and downstream automation can see why Port thinks a machine is wedged and whether recovery is already in flight.

## Context & Boundaries

### In Scope

- `crates/port-runtime/src/lib.rs` hosted K3s service policy.
- `crates/port-runtime/src/hosted_control_plane.rs` wedge detection, recovery action gating, and wedge endpoint response shaping.
- Tests covering hosted recovery policy and wedge endpoint fidelity.

### Out of Scope

- Cloud-provider APIs and tier-3 host recycle consumers.
- `infra` rollout/reconciliation logic.
- Storage or workload-migration behavior after a node loss.

```
┌───────────────────────────────────────────────────────────┐
│                  Hosted Recovery Fidelity                 │
│                                                           │
│  hosted_k3s_service_policy  -> guest agent supervisor    │
│  hosted wedge classifier    -> recovery action selector   │
│  machine /wedge endpoint    -> downstream automation      │
└───────────────────────────────────────────────────────────┘
            ↑                              ↑
    managed service health         guest heartbeat metadata
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-guest-agent` managed service runtime | internal crate/runtime contract | Executes unhealthy restart policy inside the guest | workspace |
| Hosted control-plane recovery config | runtime config | Supplies guest thresholds and settle windows | current hosted recovery schema |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Guest wedge evidence source | Prefer real guest heartbeat age plus explicit hosted K3s runtime failure evidence | Machine placement age is not a guest-heartbeat signal and caused false positives in prod |
| Endpoint compatibility | Add evidence fields instead of replacing existing wedge state fields | Keeps the route safe for current consumers while giving automation higher-fidelity data |
| Recovery settle gating | Suppress repeated action selection while recovery is `InProgress` inside the settle window | Prevents repeated restarts/recreates from compounding transient or already-remediating failures |

## Architecture

`port-runtime` already owns hosted control-plane detection and recovery state. This voyage keeps that boundary intact:
- `hosted_k3s_service_policy(...)` expresses whether unhealthy healthchecks should trigger guest-agent restarts.
- Hosted wedge classification builds a `MachineWedgeStatus` from heartbeat age, node age, and hosted K3s managed-service runtime evidence.
- The wedge route serializes that status, including new evidence payloads, for humans and downstream automation.

## Components

- Hosted K3s Service Policy
  Purpose: tell the guest agent whether unhealthy hosted K3s services should restart.
  Behavior: set `restart_on_unhealthy` for hosted agent/server services.

- Hosted Wedge Classifier
  Purpose: classify wedge type from real evidence.
  Behavior: stop using placement age as a guest heartbeat surrogate; incorporate hosted K3s runtime/service evidence when available.

- Recovery Action Gate
  Purpose: decide whether Port should take another recovery action.
  Behavior: treat `InProgress` inside settle window as still settling, not ready for another action.

- Machine Wedge Endpoint
  Purpose: expose wedge state plus evidence to operators and automation.
  Behavior: serialize guest refresh age and hosted K3s runtime/service facts alongside existing wedge metadata.

## Interfaces

- Managed service policy interface between `port-runtime` and `port-guest-agent`.
- HTTP JSON response for `/v1/machines/<name>/wedge`.

## Data Flow

1. Hosted runtime defines managed K3s service policy and healthcheck command.
2. Guest agent evaluates service health and restarts unhealthy hosted K3s services when policy allows.
3. Hosted control-plane reads guest heartbeat age, node age, and managed service runtime state.
4. Wedge classifier produces wedge class plus evidence.
5. Recovery action selector checks current recovery state and settle window before issuing another action.
6. `/v1/machines/<name>/wedge` returns wedge state plus evidence for consumers.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted K3s service healthcheck fails while process handle still exists | Guest agent health evaluation | Restart unhealthy service when policy allows | Service restart via managed-service supervisor |
| Guest heartbeat metadata absent on a healthy running machine | Missing `guest_refresh_age_seconds` | Do not infer a guest wedge from placement age | Endpoint reports missing heartbeat age without classifying a guest wedge |
| Recovery remains `InProgress` during settle window | Recovery state + timestamps | Suppress additional restart/recreate action | Re-evaluate after settle window expires |
