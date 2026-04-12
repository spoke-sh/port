# Expose Hosted Cluster Status Schema - SRS

## Summary

Epic: VGcgU7q58
Goal: Publish one canonical hosted-cluster status contract for downstream rollout and inspection consumers.

## Scope

### In Scope

- [SCOPE-01] Canonical hosted-cluster status schema for machines, placements,
  managed services, and legacy-runtime drift.
- [SCOPE-02] Exposure of that schema through existing cluster status surfaces.
- [SCOPE-03] Authored downstream contract and proof posture for paired infra
  consumption.

### Out of Scope

- [SCOPE-04] Rewriting downstream `infra` consumers.
- [SCOPE-05] Hosted lifecycle enforcement beyond the adjacent epic.
- [SCOPE-06] Non-hosted runtime classes.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port shall model hosted machine identity, placement, managed-service state, and legacy detached-runtime drift in one canonical status payload. | SCOPE-01 | FR-01 | future stories |
| SRS-02 | Port shall expose the canonical hosted status payload through existing cluster status surfaces instead of a one-off diagnostic command. | SCOPE-02 | FR-02 | future stories |
| SRS-03 | Port shall document the downstream contract and proof expectations for hosted status consumers. | SCOPE-03 | FR-04 | future stories |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The hosted status contract shall remain machine-readable and stable enough for downstream consumers to adopt without schema forks. | SCOPE-01, SCOPE-02 | NFR-01 | future stories |
| SRS-NFR-02 | The voyage shall not introduce a second contradictory truth path for hosted cluster state. | SCOPE-01, SCOPE-02 | NFR-02 | future stories |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
