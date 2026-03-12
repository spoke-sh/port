# K3s And Kubernetes Workloads - Product Requirements

> Port can become a more legible workload platform if it ships one hosted-first,
> stateless K3s lane through the existing control-plane, host-group, machine,
> guest, and proof surfaces instead of treating Kubernetes as a separate product
> or promising a full cluster platform on the first pass.

## Problem Statement

Port now has installable Linux and macOS packages, hosted node and host-group
placement, hosted service workflows, SSH-managed remote lifecycle, and a first
attached-volume contract. What is still missing is a recognizable higher-level
workload outcome. K3s is now a credible next step, but only if the first slice
stays narrow: one hosted-control-plane cluster lane, fixed node roles,
stateless workloads, explicit bootstrap and access semantics, and no claims yet
about HA control planes, persistent volumes, ingress, or generic Kubernetes
distribution support.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define one canonical hosted-first K3s cluster contract that reuses Port's existing control-plane, host-group, machine, guest, and proof vocabulary. | The epic PRD, voyage SRS/SDD, and downstream stories describe one K3s lane that builds on hosted Port primitives instead of inventing a separate orchestration model. | Planning artifacts keep K3s attached to the canonical `port` CLI and hosted route model. |
| GOAL-02 | Plan and deliver the first executable stateless hosted K3s workflow. | One voyage scopes a fixed hosted topology with control-plane bootstrap, worker join, cluster access, and proof-backed verification. | A first K3s slice is planned and ready for operator execution without requiring HA or persistence work first. |
| GOAL-03 | Keep the first K3s lane explicit about what it does not do yet. | Docs and planning artifacts fail fast on unsupported HA, persistence, ingress, multi-group, and SSH-first cluster claims. | Operators can tell the boundary between the first hosted K3s lane and later Kubernetes platform work. |
| GOAL-04 | Preserve the current local, hosted, SSH, machine, guest, and service surfaces while K3s is introduced. | The first K3s slice reuses canonical Port verbs and route semantics instead of introducing a second Kubernetes-only toolchain. | Existing Port workflows remain valid and K3s feels like one more product lane, not a separate system. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Solo Operator | Evaluates Port from a laptop or workstation and wants one concrete workload outcome that is easier to judge than raw VM or host-group primitives. | A narrow, reviewable K3s workflow that proves Port can bring up a recognizable cluster outcome. |
| Platform Engineer | Maintains hosted Linux nodes and wants K3s planning to stay explicit about placement, bootstrap, and ownership. | One hosted-first contract that reuses control-plane and host-group boundaries instead of hiding them behind generic cluster language. |
| Product Reviewer | Needs a human-readable proof that Port can move beyond infrastructure primitives without overpromising the whole Kubernetes platform. | A bounded K3s story with clear scope limits and reviewable artifacts. |

## Scope

### In Scope

- [SCOPE-01] One hosted-control-plane-first K3s contract that binds a cluster
  to an explicit Port control plane and one host group.
- [SCOPE-02] One fixed stateless topology for the first slice, with one K3s
  server node plus at least one worker node.
- [SCOPE-03] Bootstrap, join, and operator access flows that reuse canonical
  `port machine`, `port guest`, and hosted route surfaces.
- [SCOPE-04] One proof-backed operator workflow for cluster bring-up and
  cluster or workload visibility.
- [SCOPE-05] Explicit fail-fast boundaries for unsupported persistence, HA,
  ingress, SSH-first cluster ownership, and broader multi-provider rollout.

### Out of Scope

- [SCOPE-90] HA K3s or multi-server control planes.
- [SCOPE-91] Persistent volumes, CSI, hosted attached-volume routing, or
  durable cluster state beyond the current guest-image boundary.
- [SCOPE-92] Ingress, load balancer, public endpoint, or generic service
  exposure productization.
- [SCOPE-93] SSH-first cluster orchestration, multi-group or multi-provider
  clusters, or generic Kubernetes distribution abstraction.
- [SCOPE-94] A second Kubernetes-only CLI, compatibility bridge, or long-term
  support promise for every cluster topology.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define a canonical hosted K3s cluster contract that reuses the existing hosted control-plane, host-group placement, machine, and guest vocabulary rather than inventing a separate orchestration model. | GOAL-01, GOAL-04 | must | The first K3s slice only stays coherent if it builds on the product surface Port already ships. |
| FR-02 | Port must support one fixed, stateless hosted K3s workflow with control-plane bootstrap and worker-node join through canonical Port surfaces. | GOAL-02 | must | The epic exists to turn K3s from research into an executable hosted workload outcome. |
| FR-03 | Port must expose cluster access and status through existing operator surfaces, including kubeconfig or equivalent API access and visibility into node or workload state. | GOAL-02, GOAL-04 | must | A cluster that boots but cannot be inspected or used through the canonical workflow is not a credible product slice. |
| FR-04 | Port must keep unsupported K3s shapes explicit, including HA, persistence, ingress, SSH-first ownership, and broader multi-group rollout. | GOAL-03 | must | Scope control is the main risk in this epic, so unsupported shapes need first-class boundaries. |
| FR-05 | Port must publish a proof-backed operator workflow for the first hosted K3s lane, including at least one human-reviewable artifact path. | GOAL-02, GOAL-03 | should | The first K3s slice needs reviewable evidence that humans can understand quickly. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Hosted K3s placement and lifecycle output must keep control-plane, host-group, selected-node, and rejected-node detail explicit. | GOAL-01, GOAL-03, GOAL-04 | must | K3s planning becomes misleading if cluster placement looks opaque or magical. |
| NFR-02 | The first K3s slice must preserve current hosted machine, guest, service, local, and SSH workflows without silent behavior changes. | GOAL-04 | must | K3s cannot be allowed to destabilize the credible foundation Port already shipped. |
| NFR-03 | Verification for the first K3s slice must use repo-local techniques aligned with Keel recommendations, including Rust tests, command proofs, and one recording-backed human proof path through the proof system. | GOAL-02, GOAL-03 | must | The board should carry executable evidence, not only planning prose. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove shared cluster-contract, placement, and lifecycle behavior through
  story-level Rust tests mapped to voyage requirements.
- Use command proofs for bootstrap, cluster access, and boundary failures.
- Record at least one human-reviewable proof artifact through the proof system,
  using the same cast or gif review pattern already established for prior
  hosted and storage stories.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A hosted-control-plane-first K3s lane is the smallest credible first slice for Port. | The epic would need a different first topology or an SSH-first design before any delivery starts. | Validate in the first voyage and keep topology fixed there. |
| One server node plus one worker node is enough to prove the first cluster outcome. | The first slice would expand into HA or broader scheduling work too early. | Validate in voyage SRS and proof design. |
| Existing `port guest exec|copy|pty|logs|forward` and hosted machine placement surfaces are sufficient to bootstrap and inspect the first K3s lane. | The epic would need a broader CLI or protocol redesign before K3s delivery can begin. | Validate against current hosted docs, runtime, and story decomposition. |
| Hosted attached volumes remaining out of scope is acceptable for the first K3s slice if the workflow stays stateless. | The epic would need to block on hosted storage before any K3s execution work. | Keep persistence explicit as out of scope and verify that the first proof uses stateless workloads only. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first operator proof stop at `kubectl get nodes`, or include one stateless demo workload as part of the acceptance story? | Epic owner | Open |
| Should kubeconfig handoff happen through `guest copy`, a rendered artifact, or another canonical path that still avoids a second CLI family? | Epic owner | Open |
| Hosted placement is explicit today, but cluster bootstrap could still sprawl if implementation tries to absorb ingress, persistence, or generic distro concerns too early. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The epic defines one hosted-first, stateless K3s contract that reuses the
      shipped Port control-plane, host-group, machine, guest, and proof surfaces.
- [ ] At least one voyage plans a concrete hosted K3s slice with executable
      stories for cluster contract, bootstrap, access, and proof.
- [ ] The first K3s plan stays explicit about HA, persistence, ingress,
      SSH-first ownership, and multi-provider work remaining out of scope.
- [ ] The resulting K3s slice is reviewable through the same mission, story,
      and proof surfaces as other Port product lanes.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Findings

- The earlier reasons to park K3s were sequencing objections, and the
  installable, hybrid-execution, and storage-foundation missions are now
  reflected in shipped install, operator, and hosted contracts [SRC-03][SRC-04][SRC-05].
- Port now has explicit hosted node, host-group, placement, and service
  contracts plus repo-local proofs that a first K3s lane can reuse
  [SRC-04][SRC-05][SRC-07].
- The first slice should be a hosted, stateless, tightly scoped K3s workflow,
  not an HA cluster or generic Kubernetes platform promise
  [SRC-01][SRC-02][SRC-05].

### Opportunity Cost

Continuing to park K3s would leave one of the clearest human-readable platform
outcomes unexplored even though the enabling substrate is now present. The real
trade is no longer "K3s or foundations"; it is whether the first slice stays
narrow enough to defer HA control planes, persistent volumes, ingress, and SSH
parity while still proving Port can orchestrate a recognizable cluster outcome
[SRC-03][SRC-04][SRC-05].

### Dependencies

- Explicit installable, hybrid, and storage boundaries in the current product
  surface [SRC-03][SRC-04][SRC-05]
- Hosted node, host-group, placement, service, and guest-control contracts with
  executable proof [SRC-04][SRC-05][SRC-07]
- Human-facing cluster examples that define the expected outcome [SRC-01][SRC-02]

### Alternatives Considered

- Keep K3s parked until hosted storage and SSH parity exist. Rejected because
  the first slice can stay hosted-first and stateless without those
  dependencies [SRC-04][SRC-05][SRC-07].
- Treat Kubernetes as only another service template. Rejected because even a
  narrow K3s slice needs cluster bootstrap, worker join, access, and proof
  work beyond one service definition [SRC-01][SRC-04].
- Jump directly to HA or multi-provider K3s. Rejected because that would pull
  storage, ingress, and broader lifecycle work into the first slice before the
  narrower hosted contract is proven [SRC-02][SRC-05][SRC-07].

---

*This PRD was seeded from bearing `VDcStSMlp`. See `bearings/VDcStSMlp/` for original research.*
