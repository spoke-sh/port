# Boot Live Local Cluster And Fix Packaged Guest Validation - SRS

> Make the shipped local single-node cluster lane boot live on Linux, hand off
> a usable kubeconfig, and make guest artifact validation work from the
> installed Port contract.

**Epic:** [VFH7YspJx](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Repairing the checked-in local single-node cluster lane so
  `port cluster up` boots the shipped guest successfully on Linux.
- [SCOPE-02] Making `port cluster status` and `port cluster kubeconfig` hand
  off a live healthy cluster directly to downstream tooling.
- [SCOPE-03] Fixing the shipped guest artifact validate path and verifying that
  downstream `spoke infra` can treat Port as the owner of cluster handoff
  readiness without extra bootstrap glue.

### Out of Scope

- [SCOPE-04] AWS, hosted cluster, or multi-node cluster expansion.
- [SCOPE-05] Recorder upgrades, proof UX work, or ATXT migration.
- [SCOPE-06] Reverting to downstream `guest exec`, join-token choreography,
  manual kubeconfig rewriting, or broader Kubernetes platform features as the
  blessed path.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current local guest image or its boot wiring can be repaired without replacing the cluster-first operator surface. | dependency | The mission would need a larger redesign than a runtime-correctness slice. |
| Downstream `spoke infra` only needs a healthy cluster plus kubeconfig from Port before its own bootstrap work begins. | assumption | The voyage would need to absorb infra bootstrap semantics too early. |
| The installed CLI can be made to resolve guest artifact validation scripts from shipped paths or runtime-safe contracts. | dependency | The packaged artifact lane would remain source-checkout-only and block downstream use. |

## Constraints

- Keep the operator surface cluster-first; do not move bootstrap or handoff back
  into raw machine or guest choreography.
- Single-node local Linux is the only supported execution shape in this voyage.
- Fix runtime and artifact correctness first; do not compensate with docs-only
  or recorder-only work.
- Keep downstream infra thin: Port owns boot, readiness, and kubeconfig
  handoff; downstream repos own later GitOps or app bootstrap.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The shipped `examples/port.toml` local cluster workflow must boot a real single-node cluster on Linux through `port cluster up --cluster demo --runtime-root <tmp> --format json`. | SCOPE-01 | FR-01 | live command proof + runtime log inspection |
| SRS-02 | `port cluster status --format json` and `port cluster kubeconfig --format json` must hand off a live healthy cluster whose returned kubeconfig works with `kubectl get nodes -o wide` without manual rewrite. | SCOPE-02, SCOPE-03 | FR-02 | live command proof + downstream handoff review |
| SRS-03 | `port artifacts validate --artifact demo-guest --architecture x86-64` must succeed from the shipped CLI contract instead of resolving validation scripts under `/build/...`. | SCOPE-03 | FR-03 | installed CLI command proof |
| SRS-04 | Port must preserve the single-node local boundary and reject or defer AWS, hosted cluster, and multi-node expansion while this runtime-correctness slice lands. | SCOPE-01, SCOPE-03 | NFR-01 | inspection + regression proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Verification for this voyage must include live local runtime proof, packaged artifact validation proof, and at least one downstream handoff check rather than only docs or mock proofs. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | board evidence + command proof |
| SRS-NFR-02 | The fix must keep Port as the owner of cluster boot, readiness, and kubeconfig handoff without regressing to downstream guest choreography. | SCOPE-02, SCOPE-03 | NFR-03 | inspection + live workflow review |
| SRS-NFR-03 | The voyage must stay bounded to local single-node runtime and artifact correctness; AWS, hosted cluster, and multi-node work remain explicit follow-on scope. | SCOPE-01, SCOPE-03 | NFR-01 | planning review + regression proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VFH8C0wHN](../../../../stories/VFH8C0wHN/README.md) Repair Local Cluster Guest Boot Path | SRS-01 |
| [VFH8C1KHM](../../../../stories/VFH8C1KHM/README.md) Restore Live Cluster Status And Kubeconfig Handoff | SRS-02, SRS-NFR-02 |
| [VFH8C1fHP](../../../../stories/VFH8C1fHP/README.md) Fix Packaged Guest Artifact Validation Contract | SRS-03 |
| [VFH8C1xHO](../../../../stories/VFH8C1xHO/README.md) Verify Downstream Local Cluster Handoff | SRS-04, SRS-NFR-01, SRS-NFR-03 |
