# Define Real-HA Control Plane Placement Truth - Software Design Description

> Tighten the hosted AWS PVM K3s contract so real-HA placement remains a
> truthful property of the current cluster model instead of a loose marketing
> label applied to multi-node control planes.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the current hosted AWS PVM K3s scheduler and placement model
into an honest real-HA contract. The design has three goals:

1. derive real-HA eligibility from the existing cluster model instead of
   inventing a second operator workflow
2. persist control-plane execution-host spread as inspectable placement truth
3. surface HA satisfaction or failure explicitly through cluster-facing output

The work deliberately stops short of endpoint failover proofs or external
load-balancer ownership. This voyage is about placement truth.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                        This Voyage                         │
│                                                             │
│  cluster config ──> HA admission ──> hosted placement       │
│       │                 │                  │                │
│       │                 └────> fail-honest └──> status truth│
│       │                                                     │
│       └─────────────────────> existing cluster verbs        │
└─────────────────────────────────────────────────────────────┘
             ↑                               ↑
      host-group inventory             adjacent endpoint epic
```

### In Scope

- hosted AWS PVM real-HA admission rules
- execution-host spread evidence for control-plane placements
- cluster-facing HA satisfaction and failure-domain reporting

### Out of Scope

- endpoint front-end ownership or external LB automation
- multi-provider HA generalization
- disaster recovery or multi-region topology

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` K3s cluster config and validation | internal model | derive honest HA eligibility from the existing cluster contract | current workspace |
| `port-runtime` hosted control-plane placement state | internal runtime | persist machine-to-execution-host spread and rejection detail | current workspace |
| `port-cli` cluster rendering | internal CLI | expose HA satisfaction and failure-domain truth to operators | current workspace |
| Hosted node registration and imported inventory | internal hosted substrate | supply the eligible execution-host pool for spread admission | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| HA request surface | Derive real-HA intent from the existing cluster contract: at least three control-plane machines plus `control_plane_scheduler = "spread"` on hosted AWS PVM. | This keeps `port cluster ...` canonical and avoids a second HA-only API. |
| Truth source | Reuse the hosted placement state as the source of execution-host truth rather than synthesizing HA from machine count alone. | Operators need to inspect actual failure domains, not inferred topology. |
| Failure posture | Reject or mark unsatisfied HA when spread cannot be achieved, instead of silently reusing an occupied host. | The mission charter explicitly rejects degraded-but-labeled-HA behavior. |
| Scope boundary | Keep external LB/VIP/DNS ownership out of this voyage. | Placement truth must land before endpoint failover proof. |

## Architecture

The voyage touches three layers:

1. `port-model` classifies when a hosted AWS PVM cluster is eligible to claim
   real HA.
2. `port-runtime` admission and hosted placement state preserve the
   control-plane-to-execution-host spread contract.
3. `port-cli` and cluster-facing runtime reports render HA satisfaction,
   selected hosts, and exhausted/rejected candidates explicitly.

## Components

### HA Topology Admission

- Purpose: define when the existing cluster model can honestly claim real HA.
- Interface: K3s cluster validation and any derived runtime eligibility checks.
- Behavior: require a three-server control plane, hosted ownership, AWS
  `x86_64` PVM compatibility, and spread scheduling before the cluster is
  treated as HA-capable.

### Placement Evidence

- Purpose: preserve actual failure-domain truth for control-plane placements.
- Interface: hosted control-plane placement state and lifecycle reporting.
- Behavior: record which execution host each control-plane machine landed on
  plus the candidate or rejection context that explains failures.

### Cluster Status Truth

- Purpose: show whether the cluster currently satisfies the real-HA contract.
- Interface: cluster status/list/report rendering surfaces.
- Behavior: render the stable endpoint, control-plane machine-to-host mapping,
  and HA-satisfied or HA-unsatisfied posture without inventing new verbs.

## Interfaces

- Config surface: existing `k3s_clusters.*.server_machines`,
  `control_plane_scheduler`, host-group, and machine ownership fields
- Validation surface: config validation and hosted admission failures
- Inspection surface: cluster status/report output that exposes host spread and
  HA satisfaction explicitly

## Data Flow

1. Operator or downstream config loads a hosted AWS PVM K3s cluster.
2. `port-model` validates whether the cluster can honestly claim real HA.
3. Hosted placement resolves eligible execution hosts from registered/imported
   fleet state and records the selected hosts or rejection detail.
4. Cluster-facing runtime and CLI output render the current failure-domain
   truth rather than guest-count optimism.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cluster tries to claim HA with fewer than three control-plane machines or without spread scheduling | config validation | reject the configuration with explicit cluster and field detail | repair the cluster topology contract and rerun |
| Hosted placement cannot find distinct eligible execution hosts | scheduler admission | fail the operation with selected, candidate, and rejected-host context | add capacity or adjust the host-group and retry |
| Placement state cannot prove current host spread | cluster status/report inspection | surface HA as unsatisfied or unknown rather than optimistic | repair placement state or relaunch through the hosted control plane |
| A non-AWS or non-PVM lane drifts into the real-HA code path | targeted tests or contract review | fail the story; scope has broadened incorrectly | restore explicit AWS PVM gating |
