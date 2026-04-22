# Recover Hosted Placement Truth Without Read-Path Stall - Software Design Description

> Return live, partial hosted cluster truth when stored placement drifts or disappears, keep request paths non-blocking, and expose enough status fidelity for rollout and auto-recovery to trust the control plane.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage changes the hosted control plane from a stored-placement-first
router into a live-truth-first aggregator. The design separates three concerns
that are currently entangled:

- request-path resolution of machine/service/runtime truth
- maintenance of the placement cache on disk
- cluster readiness reporting versus kubeconfig handoff

The request path should answer from live node-agent truth whenever possible and
degrade explicitly when only partial truth is available. Placement persistence
becomes a background repair concern. Cluster readiness becomes a structured
report with multiple gates rather than one kubeconfig-shaped choke point.

## Context & Boundaries

### In Scope

- hosted control-plane machine, service, and cluster status/readiness paths
- placement-cache maintenance and canonicalization for hosted machines
- hosted observability needed to diagnose placement stalls

### Out of Scope

- scheduler-policy redesign
- downstream Flux or `infra` behavior
- hosted auth-model redesign

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  live resolver  placement reconcile    │
│  cluster readiness split               │
└─────────────────────────────────────────┘
        ↑                 ↑
  node-agent truth   stored placement cache
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted node-agent routes | Internal runtime contract | Supplies live machine/service/runtime truth for hosted machines | Current `/v1/node/...` API |
| Hosted machine placement state | Internal persistence | Keeps repaired placement cache for lifecycle and cross-process continuity | `machine-placements.json` |
| Hosted cluster status/kubeconfig CLI surfaces | Internal product surface | Consumes hosted control-plane truth during rollout and recovery | Current `port cluster ...` verbs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Placement authority | Treat stored placement as a cache, not the authority for read paths | Production showed live node-agent truth can stay healthy while the cache drifts |
| Repair timing | Reconcile placement on startup/registration/lifecycle hooks, not in synchronous reads | Write-on-read increases latency and makes failures recursive |
| Readiness model | Split cluster readiness from kubeconfig handoff | Operators need to see an API-ready cluster even when kubeconfig retrieval is the only failing seam |
| Failure posture | Prefer degraded partial truth over `malformed` when live runtime truth exists | The control plane should preserve truth, not overwrite it with cache damage |

## Architecture

The design introduces three cooperating seams:

1. A live placement resolver for machine/service routes.
   It combines stored placement, candidate-node metadata, and live node-agent
   probes into one `ResolvedPlacement`/`ResolvedHostedTruth` decision.

2. A placement reconcile layer.
   It updates `machine-placements.json` only from explicit lifecycle or node
   events and canonicalizes alias drift before persisting state.

3. A structured cluster-readiness builder.
   It evaluates machine/runtime visibility, API visibility, node visibility,
   and kubeconfig handoff independently so status can degrade without blocking.

## Components

- Hosted machine/service live resolver:
  chooses live node-agent truth first, then falls back to stored placement with
  explicit degraded detail if live refresh fails.
- Placement reconciler:
  rebuilds or repairs per-machine placement from startup snapshots, node
  registration, launch success, and stop/restart events.
- Cluster readiness model:
  returns separate readiness axes and preserves machine/API truth when
  kubeconfig handoff fails.
- Observability hooks:
  count timeouts, repairs, alias rewrites, and degraded responses so operators
  can see the difference between runtime failure and cache drift.

## Interfaces

- `/v1/machines`
- `/v1/machines/{machine}`
- `/v1/machines/{machine}/services/{service}`
- hosted guest-route endpoints that currently depend on placement
- `port cluster status --format json`
- `port cluster kubeconfig --format json`

## Data Flow

1. A request asks for hosted machine/service/cluster truth.
2. The handler reads cached placement and candidate metadata, but does not
   persist anything.
3. The live resolver probes the relevant node-agent route with a bounded
   timeout.
4. If live truth succeeds, the handler returns it and emits an asynchronous
   placement repair signal if the cache drifted.
5. If live truth fails but stored placement exists, the handler returns
   degraded stored truth with explicit route/detail context.
6. Startup, registration refresh, launch, and stop hooks drive the placement
   reconciler, which canonicalizes node names and updates the persisted cache.
7. Cluster readiness builds machine/runtime, API, node-visibility, and
   kubeconfig states independently, then renders a structured summary.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Stored placement missing while live node-agent still knows the machine | Live probe succeeds, cache lookup fails | Return live truth and schedule placement repair | Reconcile placement asynchronously |
| Stored placement points at stale or alias-drifted node identity | Canonicalization mismatch or stale binding | Return degraded status with canonical route detail | Persist repaired canonical placement outside read path |
| One machine or service route times out | Per-machine timeout expires | Return partial/degraded fleet truth, increment timeout metric | Retry next request or next reconcile tick |
| Kubeconfig guest-exec fails after machine/API readiness succeeds | Kubeconfig fetch step errors | `cluster status` reports degraded kubeconfig readiness; `cluster kubeconfig` surfaces the handoff error directly | Retry kubeconfig handoff without relaunching or hiding machine/API truth |
