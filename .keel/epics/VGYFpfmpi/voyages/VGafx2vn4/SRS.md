# Define Stable Endpoint Handoff And Failover Proof - SRS

> Keep the hosted AWS PVM HA claim tied to one stable `api_endpoint` handoff and
> one reviewable failover proof instead of guest-specific kubeconfig drift or
> prose-only assurances.

**Epic:** [VGYFpfmpi](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Treat `k3s_clusters.*.api_endpoint` as the canonical stable
  endpoint contract for eligible hosted AWS PVM HA clusters.
- [SCOPE-02] Surface stable-endpoint readiness, HA posture, and missing
  failover prerequisites through cluster-facing output and proof surfaces.
- [SCOPE-03] Capture one human-reviewable failover proof for a supported
  control-plane loss scenario behind that stable endpoint.

### Out of Scope

- [SCOPE-04] External load-balancer, VIP, DNS, TLS issuance, or ingress
  ownership.
- [SCOPE-05] Control-plane spread logic itself; that belongs to the adjacent
  placement epic.
- [SCOPE-06] Multi-region, disaster-recovery, or broad hosted HA endpoint
  promises.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The adjacent placement voyage delivers distinct execution-host truth for the first supported HA topology. | dependency | The endpoint proof could overfit a non-HA control plane and become misleading. |
| Port's existing `api_endpoint` field is the right stable-endpoint contract instead of inventing a second cluster address field. | assumption | The voyage would expand into cluster API redesign before proving HA handoff. |
| The repo's current proof stack can capture one supported failover scenario in a human-reviewable artifact. | dependency | The voyage would need proof-substrate work before endpoint claims can close. |

## Constraints

- Preserve the existing downstream contract on `port cluster up`, `status`, and
  `kubeconfig`.
- Do not let a guest-specific IP or one selected control-plane machine become
  the claimed stable endpoint for the HA lane.
- Fail honestly when endpoint continuity still depends on manual downstream
  rewrites or unsupported external prerequisites.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port cluster up`, `port cluster status`, and `port cluster kubeconfig` must treat the configured `api_endpoint` as the canonical stable endpoint for eligible hosted AWS PVM HA clusters and must not hand off a control-plane guest IP as the stable address. | SCOPE-01, SCOPE-02 | FR-01 | automated Rust tests + CLI proof |
| SRS-02 | Cluster-facing inspection and diagnosis surfaces must report the stable endpoint's HA posture, supported failover condition, and missing prerequisites explicitly. | SCOPE-02 | FR-02 | automated Rust tests + command proof |
| SRS-03 | Port must provide one canonical failover proof showing that the stable endpoint remains usable through one supported control-plane host-loss or guest-replacement scenario on hosted AWS PVM. | SCOPE-03 | FR-03 | command proof + review artifact |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | HA endpoint proofs must remain human-reviewable through Port's canonical proof surfaces rather than relying on unstored ad hoc notes. | SCOPE-03 | NFR-01 | proof artifact review |
| SRS-NFR-02 | Port must not claim a stable HA endpoint when kubeconfig or recovery still requires manual downstream rewrites or unsupported operator intervention. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated tests + negative-path proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| Stable endpoint handoff in cluster output | SRS-01, SRS-02, SRS-NFR-02 |
| Hosted AWS PVM failover proof | SRS-03, SRS-NFR-01, SRS-NFR-02 |
