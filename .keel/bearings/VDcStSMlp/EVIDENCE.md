---
id: VDcStSMlp
---

# K3s And Kubernetes Workloads — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-open | https://docs.slicervm.com/examples/ha-k3s/ | 2026-03-11 | 2026-03-11 | medium | high | Slicer's HA k3s example shows a concrete three-node cluster outcome that humans immediately understand. |
| SRC-02 | web | manual:web-open | https://docs.slicervm.com/examples/autoscaling-k3s/ | 2026-03-11 | 2026-03-11 | medium | high | The autoscaling example shows how a VM platform can frame Kubernetes as a higher-level hosted product surface. |
| SRC-03 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/.keel/epics/1vzTQB000/PRD.md | 2026-03-11 | 2026-03-11 | high | high | Existing hosted-fleet planning explicitly excluded a full Slicer-class cluster manager in one voyage. |
| SRC-04 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/.keel/epics/1vzSbL000/PRD.md | 2026-03-11 | 2026-03-11 | high | high | Port already has host-group and multi-node service-planning primitives that a future k3s lane could reuse. |

## Feasibility

Feasible, but only as a layered follow-on to the existing hosted fleet and
placement foundation. The current repo does not need a brand-new control model;
it needs a narrow cluster bootstrap and lifecycle layer on top of the hosted
primitives it already built.

## Findings

### 1. HA k3s is a legible product outcome

Slicer's HA k3s example turns VM orchestration into an immediately legible
human outcome: a three-node cluster with optional API load balancing [SRC-01].
That is exactly the kind of artifact that makes a mission understandable beyond
raw throughput or scheduler internals.

### 2. Port already has some of the right substrate and fleet pieces

Port's hosted-fleet and scheduler planning already defined registered nodes,
deterministic placement, host groups, and multi-node service semantics. Those
primitives are not yet a cluster manager, but they are the right building
blocks for a first k3s lane [SRC-03][SRC-04].

### 3. The first Port k3s slice should stay opinionated

The right starting point is not "support all Kubernetes." The right starting
point is an opinionated Port-managed k3s outcome such as HA control planes,
worker joins, and one proof workload with human-readable artifacts [SRC-01][SRC-02].

## Open Technical Risks

- Cluster bootstrap can sprawl if Port tries to own every Kubernetes concern in
  the first slice.
- Node lifecycle may need tighter remote-SSH and storage contracts before a
  cluster demo is robust.
- A weak proof artifact could make the work look like another low-level fleet
  feature instead of a recognizable platform capability.

## Key Findings

1. HA k3s is an excellent human-facing proof target for Port [SRC-01].
2. Port already has several hosted-fleet primitives that a k3s lane can reuse
   [SRC-03][SRC-04].
3. The first k3s story should be opinionated and narrow rather than a generic
   Kubernetes platform promise [SRC-01][SRC-02].

## Unknowns

- Should the first k3s slice target one cluster per host group, or one cluster
  spanning multiple providers or hosts?
- Which bootstrap path should be canonical first: guest exec, SSH, userdata, or
  a small Port-specific helper?
