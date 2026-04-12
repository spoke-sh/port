# Seal Managed Hosted K3s Ownership - SRS

## Summary

Epic: VGcgU9T57
Goal: Keep hosted K3s lifecycle under explicit managed-service ownership and prove it across the observed worker-loss window.

## Scope

### In Scope

- [SCOPE-01] Durable hosted placement and service-record persistence.
- [SCOPE-02] Elimination of legacy detached hosted K3s paths.
- [SCOPE-03] Soak-oriented proof for hosted worker stability and recovery.

### Out of Scope

- [SCOPE-04] Non-hosted runtime classes.
- [SCOPE-05] Broader HA topology work.
- [SCOPE-06] Downstream `infra` consumption changes.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port shall persist hosted placement and managed-service records durably enough that reuse and service status do not depend on transient launch-time state. | SCOPE-01 | FR-01 | future stories |
| SRS-02 | Port shall reject, replace, or otherwise eliminate legacy detached hosted K3s paths in favor of managed-service ownership. | SCOPE-02 | FR-02 | future stories |
| SRS-03 | Port shall provide reviewable proof that hosted workers remain healthy or recover correctly across the observed 60-90 minute drift window. | SCOPE-03 | FR-03 | future stories |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Managed-service ownership shall remain explicit in runtime artifacts and service-status surfaces rather than hidden in shell-only behavior. | SCOPE-01, SCOPE-02 | NFR-01 | future stories |
| SRS-NFR-02 | Durability proof shall remain reviewable without depending on workstation-local lore. | SCOPE-03 | NFR-02 | future stories |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
