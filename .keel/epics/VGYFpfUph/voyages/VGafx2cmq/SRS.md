# Define Real-HA Control Plane Placement Truth - SRS

> Tighten Port's hosted AWS PVM K3s contract so "real HA" means at least three
> control-plane microVMs placed across distinct execution hosts, with explicit
> admission and status truth when that contract is or is not satisfied.

**Epic:** [VGYFpfUph](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Define the real-HA admission contract for hosted AWS `x86_64` PVM
  K3s clusters using the existing `server_machines`,
  `control_plane_scheduler`, host-group, and hosted-control-plane ownership
  model.
- [SCOPE-02] Require distinct execution-host placement evidence for
  control-plane microVMs on the hosted AWS PVM lane.
- [SCOPE-03] Surface HA satisfaction, selected execution hosts, and rejected or
  exhausted candidate detail through cluster-facing operator output.

### Out of Scope

- [SCOPE-04] External load-balancer, VIP, DNS, or ingress ownership for the
  stable Kubernetes API endpoint.
- [SCOPE-05] Generic non-AWS, non-`x86_64`, non-PVM, or non-hosted HA claims.
- [SCOPE-06] Stable-endpoint failover proof; that belongs to the adjacent
  endpoint epic.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Port's existing host-group, registered-node, and imported-inventory contracts are still the right scheduling substrate for the first real-HA slice. | dependency | The voyage would stall in inventory redesign instead of clarifying HA truth. |
| The existing `control_plane_scheduler = "spread"` contract is the right starting point for real HA instead of introducing a second HA-specific scheduler vocabulary. | assumption | A new scheduling model would be needed before honest HA admission can ship. |
| `port cluster up`, `status`, and `kubeconfig` remain the canonical downstream handoff surfaces. | dependency | The voyage could drift into a parallel operator workflow and break the current downstream seam. |

## Constraints

- Count real HA only when at least three control-plane microVMs are placed
  across distinct execution hosts.
- Preserve the existing downstream contract on `port cluster up`, `status`, and
  `kubeconfig`; the first HA slice belongs inside Port's current cluster
  surface.
- Fail honestly when candidate host capacity, host diversity, or ownership
  constraints cannot satisfy the real-HA spread requirement.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must treat a hosted AWS PVM control plane as real-HA eligible only when the cluster contract specifies at least three control-plane machines, hosted ownership, and `control_plane_scheduler = "spread"` across distinct eligible execution hosts. | SCOPE-01, SCOPE-02 | FR-01 | automated Rust tests + config validation |
| SRS-02 | Hosted placement state and scheduling output must record which control-plane machine landed on which execution host and which candidate hosts were rejected or exhausted while satisfying the spread contract. | SCOPE-02, SCOPE-03 | FR-03 | automated Rust tests + inspection proof |
| SRS-03 | `port cluster status`, lifecycle reports, and related cluster-facing output must surface whether the cluster currently satisfies the real-HA spread contract instead of inferring HA from guest count alone. | SCOPE-03 | FR-03 | automated Rust tests + CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The first real-HA placement contract must stay explicitly scoped to hosted AWS `x86_64` PVM and must not broaden Port's product language into generic hosted HA. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | code inspection + targeted contract tests |
| SRS-NFR-02 | Admission and scheduling must fail honest: Port must not silently reuse an occupied execution host or collapse back to the single-host story while still presenting the cluster as HA. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated Rust tests + negative-path proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| Honest real-HA topology admission | SRS-01, SRS-NFR-02 |
| Cluster status spread and failure-domain truth | SRS-02, SRS-03, SRS-NFR-01 |
