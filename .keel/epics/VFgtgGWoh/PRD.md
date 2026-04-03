# Shell Driver Integration Contract For Upstream Control Planes - Product Requirements

## Problem Statement

Port exposes canonical guest exec, pty, and forward verbs, but upstream control planes still need a sealed integration contract that proves those guest-backed flows remain consumable as one shell driver without inventing a second shell protocol or hiding runtime failures.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Seal one upstream integration contract for guest-backed shell behavior on Port. | Upstream systems can consume guest `exec`, `pty`, and `forward` as one Port shell driver contract. | One canonical integration contract and proof path |
| GOAL-02 | Preserve the existing Port guest protocol and verb model as the canonical transport surface. | No new creator-specific shell protocol is required for upstream integration. | One contract that reuses existing Port verbs and protocol frames |
| GOAL-03 | Keep provider and runtime failures explicit for higher-level product surfaces. | Wrong lane, missing host kit, and missing artifacts surface actionable errors without fallback. | One documented and tested failure contract for upstream control planes |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Upstream Control Plane | The higher-level service integrating Port-backed shell access into a creator product. | A stable contract for launching, streaming, and auditing guest-backed shell sessions without building a second protocol. |
| Port Runtime Maintainer | The engineer preserving canonical guest behavior across hosted and local execution paths. | Guardrails that keep new integration work from fragmenting guest exec, pty, or forward semantics. |

## Scope

### In Scope

- [SCOPE-01] The integration contract for guest-backed `exec`, `pty`, and `forward` on the hosted AWS x86_64 PVM lane.
- [SCOPE-02] Lifecycle, streaming, and attach semantics required so upstream systems can treat those verbs as one shell driver.
- [SCOPE-03] Provider-aware failure behavior, documentation, and automated coverage for the integration contract.

### Out of Scope

- [SCOPE-04] Creator-specific auth, policy, tenancy, or product UX concerns outside Port.
- [SCOPE-05] Alternate shell protocols, generic multi-provider rollout, or a non-AWS first delivery.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define one canonical upstream shell-driver contract for guest-backed `exec`, `pty`, and `forward` on the hosted AWS PVM lane. | GOAL-01 | must | Upstream control planes need one integration surface rather than verb-specific special cases. |
| FR-02 | The integration contract must preserve the existing Port guest protocol frames and verb model rather than inventing a second creator-specific shell protocol. | GOAL-01, GOAL-02 | must | Protocol fragmentation would violate the mission constraint and increase downstream complexity. |
| FR-03 | Port must document and verify the lifecycle semantics that upstream systems can rely on for streamed PTY/forward and command-style exec behavior. | GOAL-01 | must | Higher-level product surfaces need explicit expectations for behavior and lifecycle. |
| FR-04 | Wrong lane, missing host kit, missing artifacts, or unsupported hosted prerequisites must fail with actionable guidance and no silent fallback. | GOAL-03 | must | Downstream systems need honest failures instead of hidden provider drift. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The contract must remain consumable through the canonical Port CLI/runtime surfaces so local and hosted behavior stay comparable. | GOAL-01, GOAL-02 | must | Upstream integrators and operators both benefit from one canonical surface. |
| NFR-02 | Verification must cover both successful guest-backed shell flows and explicit provider-aware failure surfaces. | GOAL-01, GOAL-03 | must | Upstream contracts are only safe if success and failure behavior are both proven. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Shell-driver behavior | Automated tests across hosted guest `exec`, `pty`, and `forward` paths | Story-level regression artifacts for canonical behavior |
| Integration contract | Docs and CLI proof review | Human-reviewable transcript or rendered proof of the upstream contract |
| Failure guidance | Focused failure-path tests and manual spot checks | Evidence for wrong-lane, missing-host-kit, and missing-artifact guidance |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing guest protocol framing is sufficient for creator-platform shell integration if its contract is made explicit. | The work could expand into a larger transport redesign. | Keep the first decomposition focused on contract surfacing, not protocol replacement. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which streamed-lifecycle guarantees does the creator platform need beyond the current Port guest semantics? | Epic owner | Open |
| Downstream infrastructure may still assume the old standard-lane fallback behavior instead of the canonical AWS hosted PVM lane. | Epic owner | Active risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Upstream systems can consume guest-backed `exec`, `pty`, and `forward` as one Port shell-driver contract on hosted AWS PVM.
- [ ] The contract reuses the canonical Port guest protocol and verb model rather than introducing a second shell protocol.
- [ ] Failure surfaces for wrong lane or missing hosted prerequisites are explicit and actionable.
<!-- END SUCCESS_CRITERIA -->
