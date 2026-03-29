# Replace Demo API With GitOps-Capable Local K3s Runtime - SRS

> Upgrade Port's local single-node cluster lane from a demo API to a real
> GitOps-capable K3s control plane that supports normal kubeconfig handoff,
> Kubernetes discovery, Flux install, Helm operator install, and unchanged
> downstream infra bootstrap and health.

**Epic:** [VFHmKH5XR](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Replacing the shipped demo local cluster lane with a real
  single-node local K3s control plane.
- [SCOPE-02] Handing off a kubeconfig that normal Kubernetes clients can use
  directly without downstream rewriting.
- [SCOPE-03] Ensuring Kubernetes API discovery exposes the resources required
  by Flux, Helm, and the Pulumi Kubernetes Operator install path.
- [SCOPE-04] Proving host-side `flux install` and
  `helm upgrade --install pulumi-kubernetes-operator ...` against the Port
  kubeconfig, plus unchanged downstream `infra bootstrap --env local` and
  `infra health --env local` against the Port-owned cluster handoff.

### Out of Scope

- [SCOPE-05] AWS, hosted-cluster, or multi-node cluster work.
- [SCOPE-06] Ingress, load balancer, or broader traffic-management features
  unless they are strictly required for a functioning local single-node K3s
  control plane.
- [SCOPE-07] Recorder migrations, proof UX work, or downstream guest-exec
  workarounds that bypass Port's cluster contract.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current local cluster lane can host a real K3s control plane without expanding beyond single-node local scope. | dependency | The voyage would need a broader provider or topology redesign and should yield. |
| Port can keep owning kubeconfig handoff and API reachability without requiring downstream `infra` rewrites or guest choreography. | assumption | The GitOps-ready cluster-first contract would regress. |
| Flux, Helm, and the Pulumi operator only require a real Kubernetes API surface plus the handed-off kubeconfig; no additional downstream contract changes are intended in this slice. | dependency | The consumer proof would need new mission scope rather than a runtime fix. |

## Constraints

- Keep the cluster-first Port contract intact; do not reintroduce raw machine
  orchestration, join-token choreography, or kubeconfig rewriting as the
  supported path.
- Keep execution local-only and single-node only.
- Fix runtime and bootstrap correctness first; do not mask a stub control plane
  with docs-only or proof-only work.
- Treat unchanged downstream `infra` flows as the integration boundary for this
  voyage.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` must boot a real single-node local K3s control plane rather than the current demo or stub API path. | SCOPE-01 | FR-01 | live command proof + runtime inspection |
| SRS-02 | `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` must hand off a kubeconfig that works for normal Kubernetes clients without downstream rewriting. | SCOPE-02 | FR-02 | live command proof + host-side client proof |
| SRS-03 | `kubectl api-resources -o name` against the handed-off kubeconfig must include `deployments.apps`, `namespaces`, `serviceaccounts`, `secrets`, `configmaps`, and `customresourcedefinitions.apiextensions.k8s.io`. | SCOPE-03 | FR-03 | live discovery proof |
| SRS-04 | `flux install` and `helm upgrade --install pulumi-kubernetes-operator ...` must both succeed against the same handed-off kubeconfig. | SCOPE-04 | FR-04 | live host-side bootstrap proof |
| SRS-05 | Downstream `infra bootstrap --env local` and `infra health --env local` must pass unchanged against the Port-provided cluster handoff. | SCOPE-04 | FR-05 | downstream repo proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Verification for this voyage must include live Kubernetes discovery, Flux install, Helm install, and downstream infra proof rather than Port-only surface checks. | SCOPE-03, SCOPE-04 | NFR-03 | board evidence + command proof |
| SRS-NFR-02 | Port must remain the owner of cluster bring-up, readiness, kubeconfig handoff, and API reachability; downstream repos do not rewrite kubeconfig or reintroduce manual guest orchestration. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-02 | inspection + live workflow review |
| SRS-NFR-03 | The voyage must stay bounded to local single-node GitOps readiness; AWS, hosted-cluster, multi-node, and unrelated platform expansion remain follow-on scope. | SCOPE-01, SCOPE-04 | NFR-01 | planning review + regression proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VFHn1OVki](../../../../stories/VFHn1OVki/README.md) Replace Demo Local Cluster Stub With Real K3s Control Plane | SRS-01, SRS-NFR-02 |
| [VFHn1Ozkj](../../../../stories/VFHn1Ozkj/README.md) Harden Kubeconfig Handoff And Kubernetes Discovery | SRS-02, SRS-03 |
| [VFHn1PHka](../../../../stories/VFHn1PHka/README.md) Prove Flux And Pulumi Operator Install Against Port Kubeconfig | SRS-04, SRS-NFR-01 |
| [VFHn1Pslh](../../../../stories/VFHn1Pslh/README.md) Verify Unchanged Downstream Infra GitOps Handoff | SRS-05, SRS-NFR-03 |
