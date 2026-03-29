# Plan Single-Node Local Cluster Surface - SRS

> Define the first cluster-first local K3s lane with Port-owned bootstrap
> inputs, explicit cluster lifecycle and kubeconfig surfaces, and a thin
> downstream infra handoff.

**Epic:** [VFDhlRjOf](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] A named cluster-facing Port surface for one local K3s lane,
  including lifecycle and access commands for the first cluster contract.
- [SCOPE-02] A single-node local K3s cluster shape that does not depend on a
  second VM or inter-node networking before the first slice is healthy.
- [SCOPE-03] Port-owned offline bootstrap inputs, including artifact staging or
  kube-ready guest-profile work that removes guest-side live fetches.
- [SCOPE-04] Cluster health, kubeconfig, and boundary output that make Port the
  owner of cluster readiness for the first slice.
- [SCOPE-05] Docs, help, and proof artifacts that publish the thin infra seam
  and explicit follow-on boundaries.

### Out of Scope

- [SCOPE-90] Multi-node local, hosted, or AWS cluster orchestration.
- [SCOPE-91] Guest networking, CIDR allocation, or stable inter-node addressing
  beyond what the single-node local slice absolutely requires.
- [SCOPE-92] Ingress, public networking, load-balancer, or general Kubernetes
  platform claims.
- [SCOPE-93] Persistent volumes, CSI, or stateful workload guarantees.
- [SCOPE-94] Downstream infra repo delivery work beyond a proof-backed Port
  contract handoff.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The local Firecracker `standard` lane can host one healthy single-node K3s cluster without waiting on multi-node networking work. | dependency | The voyage would block on a broader networking mission before any cluster UX could ship. |
| Existing machine, guest, and hosted-K3s runtime primitives are strong enough to seed the first cluster coordinator internally. | dependency | The voyage would need a deeper runtime rewrite before a cluster command surface can appear. |
| A Port-owned artifact or guest-profile path can replace guest-side `curl https://get.k3s.io` for the first slice. | assumption | The voyage would fail its bootstrap-ownership goal and fall back into infra-side glue. |
| One healthy cluster plus usable kubeconfig is enough for the first downstream handoff proof. | assumption | The voyage would need to absorb GitOps or infra bootstrap convergence too early. |

## Constraints

- Keep the operator surface cluster-first, not a longer wrapper around raw
  `port guest exec`.
- Single-node local is the only supported execution shape in this voyage.
- The blessed bootstrap path must be Port-owned and offline-capable.
- Port must report cluster health and kubeconfig directly enough that infra can
  consume the result without manual API forwarding or kubeconfig rewriting.
- Multi-node, hosted, AWS, networking-heavy, and stateful expansion must remain
  explicit follow-on scope.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must expose a named cluster-facing operator surface for the first local K3s lane so operators do not assemble raw `machine`, `guest exec`, join-token, or API-forward steps themselves. | SCOPE-01, SCOPE-02 | FR-01 | automated test + CLI proof |
| SRS-02 | Port must bootstrap the first local cluster from Port-owned installation inputs and must not rely on guest-side `curl https://get.k3s.io` in the canonical workflow. | SCOPE-02, SCOPE-03 | FR-03 | automated test + command proof |
| SRS-03 | Port must provide cluster lifecycle, health, and kubeconfig surfaces for the first local cluster without manual API forwarding or kubeconfig rewriting outside Port. | SCOPE-01, SCOPE-04 | FR-04 | automated test + command proof |
| SRS-04 | Port must publish the thin infra handoff and explicit first-slice boundaries through docs, help, and proof artifacts. | SCOPE-05 | FR-05 | inspection + proof artifact |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The new cluster surface must preserve current machine, guest, service, and hosted-K3s primitives as implementation substrate without silent regressions. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-01 | automated regression test |
| SRS-NFR-02 | Verification for the voyage must remain repo-local and include Rust tests, CLI proofs, and one human-reviewable artifact through the proof system. | SCOPE-02, SCOPE-04, SCOPE-05 | NFR-02 | board review + command proof + recording |
| SRS-NFR-03 | Cluster-health and boundary output must clearly distinguish Port-owned cluster readiness from follow-on downstream bootstrap or networking work. | SCOPE-04, SCOPE-05 | NFR-03 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VFDk8fqnH](../../../../stories/VFDk8fqnH/README.md) Add Cluster CLI And Config Contract | SRS-01, SRS-NFR-01 |
| [VFDk8gGoC](../../../../stories/VFDk8gGoC/README.md) Stage Offline K3s Artifacts And Guest Profile | SRS-02, SRS-NFR-02 |
| [VFDk8gRoD](../../../../stories/VFDk8gRoD/README.md) Implement Cluster Lifecycle Health And Kubeconfig Surfaces | SRS-03, SRS-NFR-01, SRS-NFR-03 |
| [VFDk8ggoV](../../../../stories/VFDk8ggoV/README.md) Publish Cluster Operator Contract And Infra Handoff Proof | SRS-04, SRS-NFR-02, SRS-NFR-03 |
