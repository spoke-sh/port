# Ship Cluster-First Bootstrap UX - Product Requirements

> Port already has enough machine, guest, hosted-control-plane, and K3s
> bootstrap primitives to prove cluster behavior, but it still makes operators
> and downstream infra assemble those low-level steps themselves. This epic
> turns that latent capability into one explicit cluster-first operator
> contract, with single-node local as the first honest lane.

## Problem Statement

Port's current K3s workflow is still a low-level choreography:

- operators define `k3s_clusters.*` in config
- start control-plane and node-agent daemons manually
- launch machines explicitly
- run guest-side install commands
- read the join token manually
- fetch kubeconfig through `guest exec`
- manage API forwarding or kubeconfig rewriting outside Port

That contract was good enough for the first hosted-K3s proof, but it is not
good enough for deployment to real infrastructure. The latest downstream infra
exercise proved the gap clearly: infra had to own raw Port choreography itself,
while the local VM could not even reach `get.k3s.io` because the guest had no
routable NIC. Port now needs one simpler first-class cluster surface that makes
single-node local the first healthy path, owns bootstrap inputs inside Port,
and returns a usable cluster outcome without requiring infra-side glue.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Expose one canonical cluster-first Port surface for a healthy single-node local K3s cluster. | A named local cluster can be brought up, inspected, and torn down through a cluster-oriented `port` workflow without manual `machine`, `guest exec`, join-token, or API-forward steps. | First local cluster slice complete |
| GOAL-02 | Move bootstrap ownership under Port instead of infra-side glue or guest-side live network fetches. | The canonical workflow stages K3s inputs through Port-owned artifacts or guest-profile inputs and does not rely on `curl https://get.k3s.io` inside the guest. | First local cluster slice complete |
| GOAL-03 | Make the downstream infra seam thin and explicit. | The blessed handoff reduces to "ask Port for a healthy cluster and kubeconfig", with raw daemon or guest choreography removed from the operator contract. | Operator docs and proof path updated |
| GOAL-04 | Keep the first cluster slice honest about what follows later. | Docs, help, and validation make single-node local the first lane while modeling multi-node, AWS, and richer guest networking as explicit follow-on work. | Planning artifacts and proof publish the boundary clearly |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Infrastructure Maintainer | Owns the downstream environment that needs a working Kubernetes control plane but does not want infra to orchestrate Port's raw VM primitives directly. | One stable cluster-facing Port contract that yields a healthy cluster and kubeconfig. |
| Port Maintainer | Needs to turn Port's existing low-level K3s primitives into a more deployable product surface without overpromising the platform. | A narrow first slice that is easier to ship and verify than a full multi-node platform. |
| Platform Engineer | Evaluates whether Port can become the cluster substrate for internal infrastructure. | Explicit boundaries, health ownership, and follow-on scope for networking or hosted expansion. |

## Scope

### In Scope

- [SCOPE-01] One named cluster-oriented Port surface for the first local K3s
  lane, including create-or-define, up, status, kubeconfig, and down behavior.
- [SCOPE-02] One single-node local K3s contract that does not require inter-node
  networking or a second VM before the first cluster is healthy.
- [SCOPE-03] Port-owned bootstrap inputs for the first cluster lane, including
  artifact-driven staging or a kube-ready guest profile instead of guest-side
  live network fetches.
- [SCOPE-04] Cluster health, kubeconfig, and operator-facing boundary output
  that make Port the owner of cluster readiness.
- [SCOPE-05] Docs, help, and one proof-backed operator path that show the thin
  infra handoff and retire raw K3s choreography as the blessed workflow.

### Out of Scope

- [SCOPE-90] Multi-node local clusters, HA control planes, or worker-node join
  orchestration in the first slice.
- [SCOPE-91] AWS, hosted, or cross-node cluster orchestration that depends on
  real guest networking, stable addressing, or provider routing.
- [SCOPE-92] Ingress, load balancers, public service exposure, or generic
  Kubernetes platform semantics.
- [SCOPE-93] Persistent volumes, CSI, or stateful workload guarantees.
- [SCOPE-94] Downstream infra repo implementation work beyond the Port contract
  and proof handoff this repo must publish.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must expose a cluster-oriented operator surface for one named local K3s cluster so operators no longer assemble raw `machine`, `guest exec`, join-token, or API-forward steps manually. | GOAL-01, GOAL-03 | must | The deployment-prep gap is primarily a missing product contract, not a missing infra wrapper. |
| FR-02 | Port must make single-node local the first healthy cluster lane and fail fast on unsupported multi-node, hosted, or AWS requests in this epic. | GOAL-01, GOAL-04 | must | The current local networking reality makes multi-node or hosted expansion an unsafe first target. |
| FR-03 | Port must own bootstrap inputs for the first lane through artifact-driven staging, guest-profile preparation, or equivalent Port-managed installation inputs, and the canonical workflow must not use guest-side `curl https://get.k3s.io`. | GOAL-02 | must | Live guest fetches are currently fragile and force infra to carry bootstrap knowledge Port should own. |
| FR-04 | Port must return a usable kubeconfig and cluster health through the cluster surface without requiring manual API forwarding or kubeconfig rewriting outside Port. | GOAL-01, GOAL-03 | must | Infra can only stay thin if Port returns a ready-to-consume cluster outcome. |
| FR-05 | Port must publish the thin infra handoff and first-slice boundaries through operator docs, help surfaces, and proof artifacts. | GOAL-03, GOAL-04 | should | The product contract is not credible until operators can review the intended downstream seam and the explicit limits. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The new cluster surface must preserve existing `machine`, `guest`, `service`, and hosted-K3s primitives as implementation substrate without silent regressions. | GOAL-01, GOAL-04 | must | The first cluster UX should simplify the operator contract, not destabilize the foundations already shipped. |
| NFR-02 | Verification for the first slice must remain repo-local and reviewable through tests, CLI proofs, and at least one human-reviewable artifact. | GOAL-02, GOAL-03, GOAL-04 | must | The new contract only matters if maintainers can reproduce and inspect it locally. |
| NFR-03 | Cluster health, ownership, and boundary output must remain explicit enough that downstream infra can tell whether a failure belongs to Port's cluster contract or to later GitOps/bootstrap work. | GOAL-03, GOAL-04 | must | Health ownership is one of the core seams this mission needs to clarify. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove cluster contract, local bootstrap, kubeconfig, and health behavior
  through story-level Rust tests and CLI command proofs.
- Publish the blessed local cluster workflow through docs and help surfaces,
  then record one human-reviewable proof artifact through the Keel proof system.
- Validate that the canonical workflow and operator docs no longer use
  guest-side `curl https://get.k3s.io` or downstream infra choreography.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Port's current local Firecracker `standard` lane is strong enough to host the first single-node K3s proof without waiting for multi-node networking. | The epic would stall on networking work before any cluster-first UX can ship. | Validate in the first voyage and keep count fixed at one. |
| Existing hosted-K3s runtime logic can be reused as internal implementation substrate even though the operator-facing contract must change. | The epic may need a deeper runtime rewrite before a cluster surface can appear. | Inspect runtime primitives during voyage design and keep the first slice local-first. |
| Downstream infra only needs a healthy cluster plus kubeconfig from Port in the first handoff. | The mission would need to absorb GitOps or infra-bootstrap semantics too early. | Keep the downstream seam explicit in docs and verify with proof review. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first cluster surface ship a `port cluster new` scaffold command, or is a config-defined cluster plus `cluster up|status|kubeconfig|down` enough for the first slice? | Epic owner | Open |
| How much guest networking must still exist inside the first local guest if Port stages K3s inputs offline? | Epic owner | Open |
| Should the first proof stop at healthy cluster plus kubeconfig, or include one thin downstream GitOps bootstrap smoke path? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port defines one canonical cluster-oriented surface for a named local
      single-node K3s cluster.
- [ ] The blessed bootstrap path is Port-owned and does not rely on
      guest-side `curl https://get.k3s.io`.
- [ ] Port returns kubeconfig and cluster health directly enough that infra can
      stay thin.
- [ ] Multi-node, AWS, networking-heavy expansion, and richer platform features
      are published as explicit follow-on work rather than leaking into this
      first slice.
<!-- END SUCCESS_CRITERIA -->
