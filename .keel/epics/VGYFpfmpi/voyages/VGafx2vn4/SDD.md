# Define Stable Endpoint Handoff And Failover Proof - Software Design Description

> Keep the AWS PVM HA endpoint contract honest by reusing the existing
> `api_endpoint` field for cluster handoff, surfacing failover posture
> explicitly, and proving one supported endpoint-survival scenario through a
> human-reviewable artifact.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage gives the hosted AWS PVM HA lane one stable endpoint story instead
of several half-truths. The design does three things:

1. keeps `api_endpoint` as the canonical handoff surface for HA clusters
2. reports endpoint readiness and failover posture explicitly in cluster output
3. captures one reviewable failover proof for a supported control-plane loss
   condition

The voyage intentionally does not take ownership of external LB or DNS
provisioning. Port proves the contract it hands off and inspects, not the full
cloud networking stack.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────┐
│                        This Voyage                        │
│                                                            │
│  api_endpoint ──> cluster handoff ──> endpoint posture     │
│       │                    │                    │          │
│       │                    └──────────────┐     └──> proof │
│       │                                   │                │
│       └──────── existing cluster verbs <──┘                │
└────────────────────────────────────────────────────────────┘
           ↑                                 ↑
   placement-truth voyage              external LB/VIP owner
```

### In Scope

- stable endpoint handoff through current cluster result surfaces
- explicit endpoint HA posture and failover-prerequisite reporting
- one supported failover proof with a human-reviewable artifact

### Out of Scope

- external endpoint provisioning or traffic-manager ownership
- reworking placement logic itself
- multi-region or disaster-recovery orchestration

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` K3s `api_endpoint` contract | internal model | anchor stable endpoint handoff to one existing field | current workspace |
| `port-runtime` cluster up/status/kubeconfig results | internal runtime | expose stable endpoint and failover posture | current workspace |
| `port-cli` cluster rendering and kubeconfig rewrite path | internal CLI | keep downstream handoff bound to the stable endpoint | current workspace |
| Placement-truth voyage (`VGafx2cmq`) | adjacent planning dependency | supply real multi-host control-plane truth for the HA claim | same mission |
| Repo proof tooling (`vhs`/`.gif`/`.cast`) | internal proof substrate | record the supported failover scenario | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Stable endpoint source | Reuse the configured `api_endpoint` as the canonical HA handoff contract. | The field already expresses the external stable address without adding a second cluster endpoint vocabulary. |
| Ownership boundary | Keep LB/VIP/DNS ownership outside Port while making endpoint posture and missing prerequisites explicit inside Port. | The mission charter keeps downstream orchestration seams intact. |
| First failover scope | Bound proof to one supported control-plane host-loss or guest-replacement scenario that preserves quorum. | This yields honest evidence without overcommitting to disaster recovery. |
| Evidence standard | Use a human-reviewable artifact as part of the canonical proof surface. | HA claims need operator-reviewable proof, not only green tests. |

## Architecture

The voyage touches three layers:

1. `port-model` continues to anchor stable endpoint identity in the cluster
   contract.
2. `port-runtime` and hosted cluster flows propagate endpoint posture and
   failover prerequisites into cluster result surfaces.
3. `port-cli` and proof workflows keep kubeconfig handoff and review artifacts
   pinned to the stable endpoint rather than a guest-specific address.

## Components

### Stable Endpoint Handoff

- Purpose: ensure the downstream cluster consumer receives one stable API
  endpoint contract.
- Interface: cluster up, status, and kubeconfig result surfaces.
- Behavior: return and rewrite against `api_endpoint`, not whichever
  control-plane guest happened to answer first.

### Endpoint Posture Reporting

- Purpose: explain whether the stable endpoint is backed by a real-HA topology
  and what failover assumptions remain.
- Interface: cluster-facing runtime reports and CLI output.
- Behavior: surface HA readiness, supported failover scenario, and missing
  prerequisites explicitly instead of implying them.

### Failover Proof Harness

- Purpose: record one supported failover scenario through the canonical proof
  surface.
- Interface: story-level proof command(s) plus human-reviewable artifact.
- Behavior: exercise stable endpoint use before and after a bounded
  control-plane loss condition and store the resulting evidence.

## Interfaces

- Config surface: `k3s_clusters.*.api_endpoint`
- Runtime surface: cluster up/status/kubeconfig result structs and any
  failover-posture reporting fields
- Proof surface: command proof plus `vhs`/`.gif`/`.cast`-style human-reviewable
  artifact

## Data Flow

1. Operator or downstream config declares a hosted AWS PVM cluster with
   `api_endpoint`.
2. Cluster up and kubeconfig flows hand off that stable endpoint directly.
3. Cluster status/report flows inspect placement truth and render endpoint HA
   posture plus missing prerequisites.
4. The proof harness exercises one supported failover path and records endpoint
   continuity through the canonical evidence surface.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cluster handoff tries to expose a guest-specific IP as the stable endpoint | automated test or contract review | fail the story; the endpoint contract has drifted | restore `api_endpoint` as the canonical handoff source |
| Placement truth or prerequisites do not support an HA endpoint claim | status/report inspection | surface endpoint posture as unsupported or degraded with explicit detail | repair placement or external front-end prerequisites and retry |
| Failover proof depends on manual kubeconfig rewrites or undocumented operator steps | proof review | keep the story open; the endpoint is not honestly stable yet | automate the missing step or narrow the supported proof scope |
| Proof artifact is not human-reviewable | evidence review | reject the proof as insufficient | recapture through the canonical recorder path |
