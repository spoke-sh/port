# Ship Blessed Closure Promotion Runner Runtime Class - Product Requirements

## Problem Statement

Spoke and infra define a blessed-closure promotion runner, but Port does not yet provide the clean-room runtime class and execution-proof contract that trusted promotion needs across local and AWS lanes.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship one canonical Port runtime class for the trusted promotion runner. | Operators can launch and inspect a clean-room promotion runner across local and AWS lanes through one Port contract. | First promotion-runtime slice |
| GOAL-02 | Preserve the clean-room trust boundary between scratch and publish. | Promotion runs never inherit writable scratch state, creator credentials, or ad hoc local caches. | First trustworthy promotion-runner slice |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Infra Promotion Operator | Needs a trusted runtime substrate for rebuild/validate publication work. | One clean-room runtime class with explicit inputs and proof. |
| Security Reviewer | Reviews trusted publication posture. | Clear guarantees that promotion is distinct from scratch authoring. |
| Platform Maintainer | Needs Port to stay the runtime owner without absorbing policy or signing ownership. | A clean execution boundary and proof contract. |

## Scope

### In Scope

- [SCOPE-01] One canonical `blessed-closure-promotion-runner` runtime class.
- [SCOPE-02] Declared-input and clean-room execution posture for that class.
- [SCOPE-03] Execution identity and proof surfaces consumed by downstream
  publication tooling.

### Out of Scope

- [SCOPE-04] Cache publication, signing, and rollback policy ownership.
- [SCOPE-05] Spoke admission decisions or creator-facing promotion UX.
- [SCOPE-06] Scratch builder implementation beyond the adjacent runtime-class
  epic.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define a dedicated runtime class for `blessed-closure-promotion-runner` rather than reusing a scratch-builder guest with elevated trust. | GOAL-01, GOAL-02 | must | The trust boundary depends on runtime separation. |
| FR-02 | The promotion runner must accept only declared immutable inputs and trusted publication material required for rebuild or validation. | GOAL-01, GOAL-02 | must | Hidden scratch state or credentials would invalidate the clean-room claim. |
| FR-03 | Port must expose execution identity and proof surfaces for the promotion runner so downstream publication tooling can link evidence to what actually ran. | GOAL-01 | must | Trusted promotion needs runtime proof, not only config intent. |
| FR-04 | The runtime class must work across local and AWS lanes without changing the clean-room contract. | GOAL-01, GOAL-02 | should | Downstream publication tooling needs one runtime story. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Promotion execution must remain distinct from scratch execution in identity, writable state, and proof artifacts. | GOAL-01, GOAL-02 | must | Reusing the scratch lane would erase the trust boundary. |
| NFR-02 | Port must not absorb signing or publication policy ownership while adding this runtime class. | GOAL-01, GOAL-02 | must | The adjacent-system boundary has to stay intact. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Promotion runner launch | Automated tests plus CLI proof | Story-level execution proof and logs |
| Clean-room boundary | Manual review plus negative-path proof | Story-level failure-path evidence |
| Structural integrity | `keel doctor --status` | No board drift after decomposition and execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Downstream `infra` tooling will own manifest, evidence, and rollback state while Port provides execution only. | The epic could overreach into publication substrate ownership. | Reconfirm against the verified `infra` planning mission during decomposition. |
| A single clean-room contract can cover both rebuild and validate postures. | The runtime class might need separate variants or additional inputs. | Validate during voyage planning. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which proof surfaces should be mandatory for downstream evidence linkage? | Epic owner | Open |
| Does local need an equivalent trusted lane or only a contract-compatible development stand-in? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port provides one canonical `blessed-closure-promotion-runner` runtime
  class.
- [ ] The promotion runner preserves a clean-room boundary distinct from
  scratch authoring.
- [ ] Downstream tooling can link promotion evidence to Port execution proof.
<!-- END SUCCESS_CRITERIA -->
