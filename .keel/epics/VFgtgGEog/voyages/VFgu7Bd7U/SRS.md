# Guest Session Identity Contract - SRS

## Summary

Epic: VFgtgGEog
Goal: Define the stable guest-session identity and driver metadata contract that upstream creator systems can audit across hosted AWS PVM guest-backed shell flows.

## Scope

### In Scope

- [SCOPE-01] Stable guest-session identity expectations for hosted guest-backed `exec`, `pty`, and `forward` on `cloud-aws`.
- [SCOPE-02] Driver metadata required so upstream systems can classify Port as one audited shell driver.
- [SCOPE-03] Explicit failure behavior when identity or driver metadata is missing, stale, or unsupported.

### Out of Scope

- [SCOPE-04] Creator auth, tenancy, policy, or audit-retention semantics outside Port.
- [SCOPE-05] Non-AWS providers, arm64 hosted PVM, or a second shell protocol.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Hosted guest-backed `exec`, `pty`, and `forward` must surface one stable Port session identity that upstream systems can correlate across operations. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Port must expose one driver metadata contract that identifies those operations as one audited shell driver instead of verb-specific transports. | SCOPE-02 | FR-02 | automated |
| SRS-03 | Session identity and driver metadata must remain on canonical Port surfaces rather than a creator-specific API. | SCOPE-02 | FR-03 | automated |
| SRS-04 | Unsupported or missing identity and driver metadata must fail explicitly and must not silently degrade to ambiguous session state. | SCOPE-03 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Session identity and driver metadata must remain stable across repeated guest-backed operations for the same hosted AWS PVM session. | SCOPE-01, SCOPE-02 | NFR-01 | automated |
| SRS-NFR-02 | Verification must produce both automated coverage and a human-reviewable proof surface for the metadata contract. | SCOPE-02, SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
