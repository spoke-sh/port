# Export Canonical Hosted Cluster Status Contract - Product Requirements

## Problem Statement

Downstream infra still has to infer hosted cluster truth from placements, node readiness, and ad hoc service checks because Port does not yet expose one canonical hosted status contract with machine, service, and legacy-runtime truth.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Expose one canonical hosted-cluster status contract that downstream consumers can trust for machines, placements, managed services, and legacy-runtime drift. | Downstream `infra` no longer needs to reconstruct hosted cluster truth from ad hoc probes. | First canonical hosted status cutover |
| GOAL-02 | Make hosted status explicit enough to drive downstream rollout, inspection, and reuse decisions safely. | The same Port status payload can power lifecycle and operator consumption without bespoke side channels. | First downstream-ready contract |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Maintainer | Owns hosted runtime and control-plane contracts. | One explicit status surface instead of implicit runtime state spread across files and commands. |
| Infra Maintainer | Consumes Port hosted lifecycle downstream. | A machine-readable contract that removes rollout and inspection guesswork. |
| Infra Operator | Diagnoses hosted-cluster incidents in prod. | One source of truth for machine placement and service state. |

## Scope

### In Scope

- [SCOPE-01] Canonical hosted-cluster status schema for machines, placements,
  managed services, and legacy-runtime drift.
- [SCOPE-02] Downstream-ready JSON or typed status surfaces exposed through
  existing cluster status entrypoints.
- [SCOPE-03] Proof and documentation for the downstream status contract.

### Out of Scope

- [SCOPE-04] Rewriting `infra` consumption logic in this repo.
- [SCOPE-05] Non-hosted runtime classes unrelated to the hosted K3s contract.
- [SCOPE-06] Hosted lifecycle enforcement details beyond the adjacent epic.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must expose one canonical hosted-cluster status contract that includes machine identity, placement, managed-service state, and legacy detached-runtime drift. | GOAL-01 | must | Downstream simplification starts with explicit upstream truth. |
| FR-02 | The canonical hosted status contract must be available through existing cluster status surfaces rather than a one-off diagnostic path. | GOAL-01, GOAL-02 | must | Downstream consumers should not need special hidden commands. |
| FR-03 | The hosted status contract must remain explicit enough for downstream consumers to distinguish healthy managed services from legacy detached K3s artifacts. | GOAL-02 | must | Worker stability depends on rejecting the wrong runtime shape. |
| FR-04 | Port must document the downstream contract and proof expectations for the hosted status payload. | GOAL-02 | should | Cross-repo ownership only holds if the contract is authored, not tribal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The hosted status contract must stay machine-readable and stable enough to be consumed without downstream schema forks. | GOAL-01, GOAL-02 | must | One contract is only useful if consumers can rely on it. |
| NFR-02 | The new surface must not introduce a second contradictory truth source for hosted lifecycle state. | GOAL-01 | must | Parallel status paths would preserve the current ambiguity. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Status schema | Targeted automated coverage plus CLI proof | Story-level parser/schema evidence |
| Downstream readiness | Cross-repo manual proof | Story-level `port cluster status` evidence and paired infra consumption checks |
| Contract documentation | Authored doc review | Story-level contract docs and mission references |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing `port cluster status` surfaces are the right seam for downstream hosted truth. | A new command surface might be required. | Validate during the first implementation story. |
| `infra` can consume a richer hosted status payload without taking over runtime ownership. | Cross-repo simplification would stall. | Validate against paired mission `VGcfT59ur`. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which hosted status fields belong in the stable downstream contract versus best-effort diagnostics? | Epic owner | Open |
| How much historical status detail is needed before the payload becomes noisy? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port exposes one canonical hosted-cluster status contract for machines,
  placements, managed services, and legacy-runtime drift.
- [ ] Downstream consumers can rely on the existing cluster status seam instead
  of bespoke diagnostic commands.
- [ ] The hosted status contract is documented and provable for paired infra
  consumption.
<!-- END SUCCESS_CRITERIA -->
