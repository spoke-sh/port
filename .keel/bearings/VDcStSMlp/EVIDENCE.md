---
id: VDcStSMlp
---

# K3s And Kubernetes Workloads — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-open | https://docs.slicervm.com/examples/ha-k3s/ | 2026-03-11 | 2026-03-11 | medium | high | Slicer's HA k3s example shows a concrete three-node cluster outcome that humans immediately understand. |
| SRC-02 | web | manual:web-open | https://docs.slicervm.com/examples/autoscaling-k3s/ | 2026-03-11 | 2026-03-11 | medium | high | The autoscaling example shows how a VM platform can frame Kubernetes as a higher-level hosted product surface. |
| SRC-03 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/install.md | 2026-03-12 | 2026-03-12 | high | high | Port now ships a narrow installable CLI contract for Linux and macOS, which removes part of the old "repo-local only" objection to higher-level workload workflows. |
| SRC-04 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/operators.md | 2026-03-12 | 2026-03-12 | high | high | Port now publishes one canonical operator vocabulary across local, hosted, and SSH lanes plus an explicit attached-volume boundary. |
| SRC-05 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/hosted.md | 2026-03-12 | 2026-03-12 | high | high | Hosted docs now publish explicit node, host-group, deterministic placement, and live multi-node service workflows that a first K3s lane could reuse. |
| SRC-06 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/CONFIGURATION.md | 2026-03-12 | 2026-03-12 | high | high | The sample config and workflow already model hosted service nodes and host groups through canonical `port service` commands. |
| SRC-07 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/crates/port-model/src/lib.rs | 2026-03-12 | 2026-03-12 | high | high | The shared model now carries explicit `ssh`, hosted host-group, scheduler, and hosted service contracts rather than leaving these as prose-only ideas. |

### Feasibility

Feasible now, but only as a narrow hosted-first follow-on to the foundations
Port already shipped. The repo still does not need a brand-new control model;
it needs one opinionated cluster bootstrap and lifecycle layer on top of the
existing hosted placement, service, and operator contracts.

## Findings

### 1. The main blockers behind the original park decision have changed

Port now ships a versioned installable CLI contract for Linux and macOS, one
canonical local/hosted/SSH operator vocabulary, and an explicit attached-volume
boundary [SRC-03][SRC-04]. Those changes remove much of the earlier argument
that a K3s lane would be stranded behind repo-local setup and unresolved route
or storage semantics.

### 2. Port already has the strongest substrate needed for a first K3s lane

Port already publishes hosted node inventory, host groups, deterministic
placement, live hosted service execution, and shared hosted/SSH route
contracts [SRC-04][SRC-07]. Those primitives are not a cluster manager, but
they are now concrete product contracts instead of only older planning intent
[SRC-05][SRC-06].

### 3. Port still lacks any productized K3s workflow

There is still no shipped K3s or Kubernetes workflow in the docs, CLI help,
proof scripts, or runtime code. The first slice must therefore define a
canonical operator path, proof artifact, and explicit scope boundary rather
than inferring them from the existing hosted docs alone [SRC-04][SRC-05][SRC-07].

### 4. HA K3s remains the clearest human-readable product proof

Slicer's HA K3s examples still show why this work matters: cluster bring-up and
service reachability are far more legible platform outcomes than low-level VM
and scheduler internals [SRC-01][SRC-02].

## Open Technical Risks

- Cluster bootstrap can sprawl if Port tries to own every Kubernetes concern in
  the first slice instead of defining one canonical hosted-first workflow.
- Attached-volume support is still local-only today, so the first hosted K3s
  slice cannot imply persistent-volume or durable control-plane storage support
  beyond the guest image and current runtime boundary.
- A weak proof artifact could make the work look like another low-level fleet
  feature instead of a recognizable workload-platform capability.

## Key Findings

1. The installable, hybrid, and storage foundations that previously kept K3s
   parked are now materially stronger [SRC-03][SRC-04].
2. Port already has concrete hosted node, host-group, scheduler, and service
   primitives that a narrow K3s lane can reuse [SRC-05][SRC-06][SRC-07].
3. Port still lacks any shipped K3s workflow, so the first slice must be
   explicit, opinionated, and proof-backed rather than a generic Kubernetes
   promise [SRC-01][SRC-02][SRC-04].

## Unknowns

- Should the first K3s slice stay within one hosted control plane and one
  explicit host group, or span multiple groups or providers immediately?
- Which bootstrap path should be canonical first: guest exec, SSH orchestration,
  userdata, or a small Port-specific helper?
- How much persistence, ingress, or API load-balancer setup belongs in the
  first proof versus a follow-on slice?
