# Ship Workspace Scratch Builder Runtime Class - Product Requirements

## Problem Statement

Spoke and infra define a workspace-scratch-builder contract, but Port does not yet provide a canonical runtime class with isolated writable state, bounded trust posture, and reusable execution proof surfaces for that builder lane.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship one canonical Port runtime class for workspace scratch builders. | Operators can launch and inspect a workspace-bound scratch builder through Port instead of relying on repo-local runtime assumptions. | First builder-runtime slice |
| GOAL-02 | Preserve tenant isolation and bounded trust posture for that runtime class. | A scratch builder proves isolated writable state, no publish credentials, and explicit execution identity across local and AWS lanes. | First trustworthy scratch-builder slice |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Workspace Authoring Operator | Needs a runtime lane for freeform Nix authoring and candidate builds. | One canonical Port runtime class for that lane. |
| Infra Platform Maintainer | Consumes Port runtime seams downstream. | A reusable builder runtime contract instead of hidden local conventions. |
| Security Reviewer | Reviews isolation and credential boundaries for workspace execution. | Clear proof that scratch builders stay untrusted and tenant-bound. |

## Scope

### In Scope

- [SCOPE-01] One Port runtime class for `workspace-scratch-builder`.
- [SCOPE-02] Workspace-bound writable state, identity, and execution-proof
  surfaces for that class.
- [SCOPE-03] Local/AWS contract parity for the builder runtime class.

### Out of Scope

- [SCOPE-04] Trusted publication or promotion-runner behavior.
- [SCOPE-05] Spoke control-plane policy, admission, or creator-facing UX.
- [SCOPE-06] Cache signing or blessed publication ownership.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define one canonical runtime class for `workspace-scratch-builder` rather than relying on ad hoc machine or guest conventions. | GOAL-01 | must | Downstream systems need a stable runtime contract to target. |
| FR-02 | The scratch builder must carry workspace-bound writable state for store, source, and temp roots without sharing that writable state across unrelated workspaces. | GOAL-01, GOAL-02 | must | Tenant isolation is the core substrate guarantee. |
| FR-03 | The runtime class must fail fast if publish credentials, signing keys, or privileged cluster/admin credentials would cross into the builder lane. | GOAL-02 | must | Scratch must stay untrusted by construction. |
| FR-04 | Port must expose execution identity and proof surfaces for this builder class so downstream operator tools can inspect what ran, where, and with which workspace binding. | GOAL-01, GOAL-02 | should | The lane has to be inspectable, not only launchable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The contract must remain the same across local and AWS lanes even when the runtime substrate differs. | GOAL-01, GOAL-02 | must | Downstream policy should not branch by environment semantics. |
| NFR-02 | Scratch builders must not imply publish trust or hidden promotion rights. | GOAL-02 | must | The builder lane exists specifically below the trust boundary. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Builder launch and identity | Targeted automated tests plus CLI proof | Story-level logs and execution-proof artifacts |
| Isolation posture | Contract review plus failure-path proof | Story-level negative-path evidence |
| Structural integrity | `keel doctor --status` | No board drift after decomposition and execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Downstream systems can consume a runtime class contract without Port inventing a second creator-facing policy layer. | The epic could blur ownership boundaries. | Recheck against the verified Spoke and `infra` planning missions during decomposition. |
| Existing Port proof surfaces can be extended to cover builder execution identity cleanly. | The epic might need additional proof primitives first. | Validate during voyage design. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which runtime surface should be canonical first for this class: machine launch, cluster-owned lane, or a new builder-focused verb? | Epic owner | Open |
| How much network/fetch posture should Port enforce directly versus accept from downstream config? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port provides one canonical `workspace-scratch-builder` runtime class.
- [ ] Scratch builders are workspace-bound and isolated from publish/admin
  credentials.
- [ ] Downstream systems can inspect builder execution identity and proof
  surfaces through Port-authored contracts.
<!-- END SUCCESS_CRITERIA -->
