# Seal Managed Hosted Service Ownership - Product Requirements

## Problem Statement

Hosted K3s machines can still drift into stale placement or legacy detached service paths, which leaves worker recovery and downstream inspection dependent on implicit runtime behavior instead of managed Port service ownership.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Keep hosted K3s lifecycle under explicit Port-managed service ownership across launch, reuse, and recovery. | Workers and servers remain represented as managed services instead of legacy detached processes. | First fully owned hosted lifecycle slice |
| GOAL-02 | Make hosted worker stability provable over the observed 60-90 minute failure window. | Soak-oriented proof exists for managed hosted workers and their recovery posture. | First durability proof slice |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Maintainer | Owns hosted runtime behavior and service management. | One durable ownership model for hosted K3s services. |
| Infra Operator | Suffers the blast radius of worker loss downstream. | Confidence that hosted workers remain under managed recovery and placement truth. |

## Scope

### In Scope

- [SCOPE-01] Durable placement and service-record persistence for hosted K3s.
- [SCOPE-02] Rejection or replacement of legacy detached hosted K3s paths.
- [SCOPE-03] Soak-oriented proof for hosted worker stability and recovery.

### Out of Scope

- [SCOPE-04] Non-hosted runtime classes.
- [SCOPE-05] Broader control-plane HA work outside service ownership.
- [SCOPE-06] Downstream `infra` consumption logic.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must persist hosted placement and managed-service records durably enough that reuse, service status, and recovery do not depend on transient launch-time state. | GOAL-01 | must | Worker stability suffers when placement/service truth evaporates. |
| FR-02 | Port must reject, replace, or otherwise eliminate legacy detached hosted K3s paths in favor of managed-service ownership. | GOAL-01 | must | Detached K3s processes are the failure mode we need to remove. |
| FR-03 | Port must provide proof that hosted worker lifecycle remains healthy over the observed 60-90 minute drift window or recovers correctly when it does not. | GOAL-02 | must | The current pain is time-based worker loss, not just launch success. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The managed-service ownership model must remain explicit in runtime artifacts and service-status surfaces rather than hidden in shell behavior. | GOAL-01 | must | Explicit ownership is the operational contract. |
| NFR-02 | Durability proof must be reviewable without private workstation lore. | GOAL-02 | should | The evidence should survive the original debugging session. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Placement/service durability | Automated coverage plus CLI proof | Story-level runtime-state tests |
| Legacy-path elimination | CLI proof and regression inspection | Story-level managed-service status evidence |
| Soak stability | Long-running proof | Story-level soak artifact and mission log linkage |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Managed-service ownership is the correct canonical path for hosted K3s lifecycle. | The epic could target the wrong recovery model. | Validate against the current hosted runtime contract. |
| The 60-90 minute worker-loss pattern is reproducible enough to prove against. | Soak proof may need a different trigger or metric. | Revisit during the proof story if the pattern changes. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which runtime artifact best proves a worker stayed managed throughout a soak window? | Epic owner | Open |
| How should Port represent automatic recovery after a transient hosted service failure? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted K3s services stay under explicit Port-managed ownership across
  launch, reuse, and service status.
- [ ] Legacy detached hosted K3s paths are no longer treated as valid runtime
  state.
- [ ] Port has reviewable proof for worker stability or managed recovery over
  the observed 60-90 minute drift window.
<!-- END SUCCESS_CRITERIA -->
