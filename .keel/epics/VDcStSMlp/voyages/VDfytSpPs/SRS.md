# Hosted Stateless K3s Foundations - SRS

> Define the first hosted-control-plane, stateless K3s lane with a fixed
> one-server-plus-worker topology, canonical cluster access, explicit hosted
> boundaries, and proof-backed operator review.

**Epic:** [VDcStSMlp](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Add a canonical hosted K3s cluster contract that binds one cluster
  to an explicit control plane, one host group, fixed node roles, and existing
  Port machine identities.
- [SCOPE-02] Bootstrap one K3s server node and join at least one worker node
  through hosted-control-plane machine lifecycle plus canonical guest-control
  surfaces.
- [SCOPE-03] Surface cluster access and visibility through existing operator
  paths, including kubeconfig or equivalent API handoff and node or workload
  status.
- [SCOPE-04] Publish hosted-only boundary guidance and one proof-backed
  operator workflow for cluster bring-up and review.

### Out of Scope

- [SCOPE-90] HA K3s or multi-server control-plane topologies.
- [SCOPE-91] Persistent volumes, hosted attached-volume routing, CSI, or
  durable stateful workload support.
- [SCOPE-92] Ingress, public service exposure, load balancers, or generic
  network productization.
- [SCOPE-93] SSH-first cluster ownership, multi-group or multi-provider
  clusters, or a generic Kubernetes distribution abstraction.
- [SCOPE-94] A second Kubernetes-only toolchain that bypasses canonical
  `port machine`, `port guest`, and hosted route surfaces.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted control-plane routing, host-group placement, and guest attach are strong enough to anchor the first K3s slice. | dependency | The voyage would need a broader runtime or protocol redesign before K3s delivery can start. |
| One K3s server node plus one worker node is enough to prove the first cluster outcome. | assumption | The voyage would expand into HA or broader scheduling work too early. |
| The first hosted K3s slice can remain stateless because hosted attached-volume support is still intentionally out of scope. | dependency | The voyage would block on hosted storage rather than shipping a narrow K3s lane first. |
| Human-reviewable proof can continue to use the repo's recording-backed proof pattern, including renderer scripts that emit cast and gif artifacts through the proof system when direct `vhs` capture is unstable. | dependency | The voyage would need a new proof strategy and mission review surface. |

## Constraints

- Keep one canonical Port operator vocabulary; K3s must reuse hosted
  `machine`, `guest`, and proof surfaces even if it introduces a small K3s
  contract in config.
- Keep the first topology fixed and explicit: one hosted K3s server plus at
  least one hosted worker in one host group.
- Keep the first slice stateless and hosted-only.
- Fail fast on unsupported HA, persistence, ingress, SSH-first, or broader
  multi-group cluster claims.
- Use repo-local verification techniques aligned to Keel recommendations:
  Rust tests, command proofs, and one recording-backed proof path through the
  proof system.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must add a canonical hosted K3s cluster contract that binds one cluster to a control plane, one host group, explicit server and worker machine roles, and K3s bootstrap metadata without replacing the current machine and guest model. | SCOPE-01 | FR-01 | automated test + config proof |
| SRS-02 | Port must bootstrap one hosted K3s server node and join at least one hosted worker node through canonical Port lifecycle and guest-control surfaces rather than a separate remote-only toolchain. | SCOPE-02 | FR-02 | automated test + command proof |
| SRS-03 | Port must expose cluster access and visibility through canonical operator paths, including kubeconfig or equivalent access handoff plus node or workload status that proves the cluster is usable. | SCOPE-03 | FR-03 | automated test + command proof |
| SRS-04 | Unsupported K3s requests and prerequisites must stay explicit, including missing host-group capacity, unsupported persistence, HA requests, ingress claims, and non-hosted ownership routes. | SCOPE-03, SCOPE-04 | FR-04 | automated test + command proof |
| SRS-05 | The voyage must publish the first hosted K3s operator workflow, including at least one human-reviewable proof artifact recorded through the proof system. | SCOPE-04 | FR-05 | inspection + recording |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted K3s placement and lifecycle output must keep control-plane, host-group, selected-node, candidate-node, and rejected-node detail explicit. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated test + command proof |
| SRS-NFR-02 | The first K3s slice must preserve current hosted machine, guest, service, local, and SSH workflows without silent behavior changes. | SCOPE-01, SCOPE-02 | NFR-02 | automated regression test |
| SRS-NFR-03 | Verification for this voyage must use repo-local techniques recommended by Keel for this repository: Rust tests, command proofs, and a recording-backed human proof path through the proof system. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | board review + command proof + recording |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VDfzLrZ4e](../../../../stories/VDfzLrZ4e/README.md) Introduce Hosted K3s Cluster Contract | SRS-01, SRS-NFR-02 |
| [VDfzOEtFN](../../../../stories/VDfzOEtFN/README.md) Implement Hosted K3s Bootstrap And Join Workflow | SRS-02 |
| [VDfzOEdFM](../../../../stories/VDfzOEdFM/README.md) Add Hosted K3s Access And Boundary Surfaces | SRS-03, SRS-04, SRS-NFR-01 |
| [VDfzOEeFL](../../../../stories/VDfzOEeFL/README.md) Publish Hosted K3s Operator Proof | SRS-05, SRS-NFR-03 |
