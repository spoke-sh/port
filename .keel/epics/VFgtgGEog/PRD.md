# Guest Session Identity And Audit Surface - Product Requirements

## Problem Statement

Port can already broker guest-backed shell flows, but upstream creator systems still lack a stable Port-owned session identity and driver metadata surface for treating those flows as one audited shell driver across hosted AWS PVM execution.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Surface a stable Port-owned guest-session identity for creator-facing shell flows. | Upstream systems can resolve one durable Port session identifier across hosted guest `exec`, `pty`, and `forward` workflows. | One canonical guest-session identity contract with proof coverage |
| GOAL-02 | Expose driver metadata that lets upstream systems classify Port as one audited shell driver. | Port surfaces one driver identity/metadata shape rather than per-verb ad hoc labels. | One canonical driver metadata contract used across guest-backed shell flows |
| GOAL-03 | Keep the audit boundary inside Port without absorbing creator policy or auth concerns. | The surfaced metadata is runtime/driver scoped and excludes creator-domain policy fields. | One explicit boundary in docs and automated coverage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Upstream Control Plane | A creator-platform service that launches and tracks Port-backed shell sessions. | One stable session identifier and driver metadata surface that can be audited and stored without parsing transport-specific details. |
| Port Operator | The maintainer proving and debugging hosted AWS PVM shell flows. | Clear visibility into how session identity and driver metadata map onto the canonical Port guest surfaces. |

## Scope

### In Scope

- [SCOPE-01] Session identity fields and driver metadata for guest-backed hosted shell flows on AWS x86_64 PVM.
- [SCOPE-02] Canonical CLI/runtime/status or protocol surfaces required so upstream systems can treat Port sessions as one audited shell driver.
- [SCOPE-03] Automated tests and operator-facing docs that prove the metadata contract and its scope boundary.

### Out of Scope

- [SCOPE-04] Creator-platform policy, domain ownership, user auth, or audit-retention product decisions outside Port.
- [SCOPE-05] New cloud providers, arm64 hosted PVM, or non-guest-backed shell implementations.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must emit a stable guest-session identity for guest-backed hosted shell workflows so one launched session can be correlated across `exec`, `pty`, and `forward` operations. | GOAL-01 | must | Upstream systems need a durable audit key instead of transport-specific or transient identifiers. |
| FR-02 | Port must surface driver metadata that identifies the session as one Port-owned audited shell driver rather than a set of unrelated per-verb transports. | GOAL-02 | must | The creator platform needs one driver contract to integrate against. |
| FR-03 | Session identity and driver metadata must be exposed through canonical Port surfaces rather than a parallel creator-specific API. | GOAL-01, GOAL-02 | must | The existing Port command and protocol model is part of the mission constraint. |
| FR-04 | Missing, stale, or unsupported session metadata must fail with explicit guidance and no silent fallback to anonymous or ambiguous session state. | GOAL-01, GOAL-03 | must | Audit surfaces are unsafe if Port can degrade silently. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Session identity and driver metadata must remain stable across repeated guest-backed operations for the same underlying hosted AWS PVM session. | GOAL-01, GOAL-02 | must | Upstream audit and driver correlation break if identifiers drift between operations. |
| NFR-02 | Verification must include focused automated coverage plus at least one human-reviewable proof surface showing the metadata contract. | GOAL-01, GOAL-02, GOAL-03 | must | This contract is consumed downstream and needs durable proof, not chat-only intent. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Session identity | Focused automated tests across guest-backed hosted flows | Story-level test artifacts proving identifier stability |
| Driver metadata | CLI or protocol proof plus docs review | Human-reviewable transcript or rendered proof of the surfaced metadata contract |
| Boundary control | Planning and docs review with targeted tests | Evidence that creator auth/policy concerns remain outside Port |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The hosted AWS PVM runtime contract from `VFgcM1Zpu` is a valid substrate and does not need to be replanned here. | This epic could sprawl back into infrastructure/runtime work instead of session identity. | Keep AWS runtime verification referenced as a prerequisite rather than reopening it. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which exact metadata fields must be stable enough for the creator platform to treat Port as one audited shell driver? | Epic owner | Open |
| Downstream consumers may already assume transport-specific identifiers that conflict with the canonical Port session surface. | Epic owner | Active risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted guest-backed shell flows expose one stable Port session identifier that upstream systems can correlate.
- [ ] Port surfaces one driver metadata contract that describes guest-backed shell sessions as one audited shell driver.
- [ ] Failure cases for missing or unsupported metadata are explicit and do not silently degrade to ambiguous session state.
<!-- END SUCCESS_CRITERIA -->
