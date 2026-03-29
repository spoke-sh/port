# Replace Demo Local Cluster Stub With Real K3s Runtime - Product Requirements

## Problem Statement

Port's current local cluster handoff is good enough for cluster up, kubeconfig, and kubectl get nodes, but it is still a demo API rather than a GitOps-capable single-node K3s control plane. Downstream infra can prove cluster handoff readiness, yet flux install, helm install, broad api-resources discovery, and unchanged infra bootstrap/health still fail or remain out of scope because the local lane is a stub runtime.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Replace the demo local cluster surface with a real single-node local K3s runtime that supports normal Kubernetes clients and GitOps bootstrap tooling. | `port` hands off a kubeconfig that supports Kubernetes discovery plus Flux and Helm installs | `flux install`, Helm operator install, and downstream `infra bootstrap/health` pass unchanged |
| GOAL-02 | Preserve the intentionally narrow local-first boundary while closing the GitOps-readiness gap. | All execution and proof remain local-only and single-node only | No AWS, hosted-cluster, or multi-node work lands in this epic |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Operator | The operator using `port cluster up/status/kubeconfig` directly from the Port repo. | A real local K3s runtime that behaves like a normal Kubernetes control plane instead of a stub API. |
| Downstream Infra Operator | The operator consuming Port from `spoke infra`. | Unchanged `infra bootstrap` and `infra health` flows that can treat Port as a GitOps-ready cluster owner. |

## Scope

### In Scope

- [SCOPE-01] Replacing the local demo cluster stub with a real single-node local K3s control plane.
- [SCOPE-02] Handing off a kubeconfig that works for normal Kubernetes clients without downstream rewriting.
- [SCOPE-03] Ensuring Kubernetes API discovery exposes the resources needed by Flux, Helm, and the Pulumi operator install path.
- [SCOPE-04] Proving `flux install`, `helm upgrade --install pulumi-kubernetes-operator ...`, and unchanged downstream `infra bootstrap/health` against the handed-off kubeconfig.

### Out of Scope

- [SCOPE-05] AWS, hosted-cluster, or multi-node orchestration.
- [SCOPE-06] Ingress, load balancer, or broader traffic-management features that are not required for a real single-node local K3s control plane.
- [SCOPE-07] Recorder migrations, proof UX work, or downstream guest choreography workarounds.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `port cluster up --cluster demo` must bring up a real single-node local K3s control plane rather than the current demo API surface. | GOAL-01 | must | Port cannot be GitOps-ready while the local lane only simulates the Kubernetes API. |
| FR-02 | `port cluster kubeconfig --cluster demo --format json` must hand off a kubeconfig that works for standard Kubernetes clients without downstream rewriting. | GOAL-01 | must | Downstream tooling should consume Port’s kubeconfig directly. |
| FR-03 | `kubectl api-resources -o name` against the handed-off kubeconfig must include at least `deployments.apps`, `namespaces`, `serviceaccounts`, `secrets`, `configmaps`, and `customresourcedefinitions.apiextensions.k8s.io`. | GOAL-01 | must | These resources are the minimum API surface needed for GitOps and operator bootstrap. |
| FR-04 | `flux install` and `helm upgrade --install pulumi-kubernetes-operator ...` must succeed against the same handed-off kubeconfig. | GOAL-01 | must | Port’s local cluster contract must support real GitOps bootstrap clients, not only `kubectl get nodes`. |
| FR-05 | Downstream `spoke infra` must pass `infra bootstrap --env local` and `infra health --env local` unchanged against the Port-owned cluster handoff. | GOAL-01, GOAL-02 | must | The consumer repo is the real integration boundary for this epic. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Scope remains local-only and single-node only throughout this epic. | GOAL-02 | must | The GitOps-readiness gap should be solved without prematurely expanding provider or topology scope. |
| NFR-02 | Port remains the owner of cluster bring-up, readiness, kubeconfig handoff, and API reachability; downstream repos do not reintroduce manual guest orchestration or kubeconfig rewriting. | GOAL-01, GOAL-02 | must | Preserves the cluster-first operator contract established by the previous mission. |
| NFR-03 | Verification must include direct Kubernetes discovery, Flux install, Helm install, and downstream consumer proof rather than Port-only surface checks. | GOAL-01 | must | This epic is about GitOps readiness, so proof must cross the consumer boundary. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Real K3s control plane | Live `port cluster up/status/kubeconfig` plus `kubectl api-resources` and `kubectl get nodes` | Story evidence from the local Port workflow |
| GitOps bootstrap clients | Live `flux install` and Helm operator install against the handed-off kubeconfig | Story evidence from direct host-side client runs |
| Downstream consumer | Unchanged `infra bootstrap --env local` and `infra health --env local` | Story evidence from the `spoke infra` repo |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current local cluster lane can be upgraded from the demo API to a real single-node K3s runtime without expanding beyond local single-node scope. | The epic would require a larger topology or provider redesign. | Validate during voyage design and first execution slice. |
| Flux and the Pulumi operator can install successfully once the handed-off kubeconfig and API surface are real. | The mission would need additional cluster prerequisites or bootstrap changes. | Validate with direct client proof before claiming downstream readiness. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Does the local guest need additional networking or storage behavior to support a real K3s control plane and operator installs? | Epic owner | Open |
| Which existing demo-path shortcuts must be removed or replaced to avoid masking a non-GitOps-capable cluster? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `port cluster up` and `port cluster kubeconfig` hand off a real single-node local K3s control plane rather than a demo API.
- [ ] `kubectl api-resources -o name` includes the minimum resource types required for GitOps and operator bootstrap.
- [ ] `flux install` and the Pulumi operator Helm install both succeed against the same handed-off kubeconfig.
- [ ] Downstream `infra bootstrap --env local` and `infra health --env local` pass unchanged.
- [ ] The epic lands without AWS, hosted-cluster, or multi-node scope creep.
<!-- END SUCCESS_CRITERIA -->
