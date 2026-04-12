# Ship Multi-Host Control Plane Placement For Hosted AWS PVM Clusters - Product Requirements

## Problem Statement

Hosted AWS PVM K3s currently proves cluster ownership and multi-node placement, but it does not yet satisfy the stricter real-HA contract of control-plane microVM spread across distinct execution hosts.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make Port capable of placing HA control-plane microVMs across distinct AWS execution hosts. | A hosted AWS PVM cluster can request real HA and Port will either satisfy the spread requirement or fail honestly. | First real-HA placement slice |
| GOAL-02 | Keep HA claims truthful in scheduling, status, and failure surfaces. | Operators can tell whether a cluster is truly HA-capable or still effectively single-host. | First truthful HA reporting slice |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Hosted Cluster Operator | Needs real HA instead of a stronger single-host demo. | Control-plane spread across distinct execution hosts. |
| Infra Bootstrap Consumer | Depends on Port for cluster ownership and kubeconfig handoff. | HA semantics without downstream orchestration forks. |
| On-call Maintainer | Needs clear failure-domain visibility under host loss. | Honest HA status and placement output. |

## Scope

### In Scope

- [SCOPE-01] Control-plane placement and admission for real HA on hosted AWS
  PVM.
- [SCOPE-02] Distinct execution-host spread and failure-domain visibility.
- [SCOPE-03] Truthful status and failure behavior when the required spread
  cannot be satisfied.

### Out of Scope

- [SCOPE-04] Stable HA API endpoint and failover proof; that belongs to the
  adjacent epic.
- [SCOPE-05] Generic multi-provider or arm64 PVM HA work.
- [SCOPE-06] Downstream `infra` orchestration changes beyond the existing
  cluster contract.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must allow a hosted AWS PVM cluster to request a control-plane topology that requires placement across distinct execution hosts. | GOAL-01 | must | Real HA begins with topology and placement, not only guest count. |
| FR-02 | The hosted scheduler and admission flow must reject HA requests when distinct eligible execution hosts are unavailable. | GOAL-01, GOAL-02 | must | False HA claims are worse than explicit failure. |
| FR-03 | Port status surfaces must report which control-plane microVMs are placed on which execution hosts and whether the cluster satisfies the HA spread contract. | GOAL-02 | must | Operators need visibility into failure domains. |
| FR-04 | Worker placement must remain compatible with the HA control-plane topology rather than collapsing back to the single-host bootstrap story. | GOAL-01 | should | HA placement needs a coherent whole-cluster topology. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Port must not claim HA unless control-plane spread truly crosses distinct execution hosts. | GOAL-01, GOAL-02 | must | The docs already define HA more strictly than multi-node. |
| NFR-02 | The first HA slice stays AWS `x86_64` PVM-specific and does not broaden provider promises prematurely. | GOAL-01, GOAL-02 | must | Scope control matters here. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Placement and admission | Automated tests plus CLI proof | Story-level placement and failure-path artifacts |
| HA status truth | Manual review of status/doctor output | Story-level proof logs and review artifacts |
| Structural integrity | `keel doctor --status` | No board drift after decomposition and execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The hosted AWS rollout and placement substrate can scale to the required host count. | The epic could stall on infrastructure availability rather than runtime logic. | Validate against the existing multi-cell infrastructure work before decomposition. |
| Existing cluster verbs can carry HA topology semantics without inventing a second operator workflow. | The epic might need a broader CLI redesign. | Revisit only if the current contract proves insufficient. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the smallest honest HA control-plane size for the first slice? | Epic owner | Open |
| Which placement and status fields must become stable for downstream consumers? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port can place HA control-plane microVMs across distinct AWS execution
  hosts or fail honestly when it cannot.
- [ ] Port status surfaces report real HA placement and failure-domain truth.
- [ ] The single-host story is no longer mislabeled as HA.
<!-- END SUCCESS_CRITERIA -->
