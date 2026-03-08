# Hosted Control Plane And Operator Surface - Product Requirements

> Turn Port's hosted story into a real product surface by shipping the first
> authenticated control-plane foundation for node-aware inventory, lifecycle
> control, and guest bridge attachment.

## Problem Statement

Port now has strong local primitives and clear hosted/substrate contracts, but
the user objective remains incomplete because hosted Port is still design-only.
Compared with the requested Slicer-class capability set, Port still lacks:

- authenticated hosted control
- remote machine inventory, status, and stop
- node and host-group vocabulary
- a real control-plane API surface
- a hosted guest bridge for canonical guest operations

Without those foundations, later features such as monitoring, secrets,
services, sandboxes, SDKs, and detached forwarding remain fragmented or
premature.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship the first authenticated hosted-control surface | Hosted lifecycle and inventory contracts are implemented and operator-visible | First voyage complete |
| GOAL-02 | Preserve one canonical Port operator model | `machine` and `guest` verbs remain stable across local and hosted designs | First voyage complete |
| GOAL-03 | Sequence the remaining hosted features on a stable foundation | Follow-on voyages for monitoring, secrets, services, sandboxes, and SDK work are unblocked by the first voyage | Epic planned coherently |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port across local and hosted nodes | One coherent CLI/API for inventory and lifecycle control |
| Hosted Port Builder | Implements Port's control plane and node-agent product | Stable contracts for auth, node ownership, and guest brokerage |
| Application Operator | Eventually consumes monitoring, secrets, and services | Hosted product surfaces that build on predictable lifecycle behavior |

## Scope

### In Scope

- [SCOPE-01] Token-based hosted API identity for the first control-plane slice.
- [SCOPE-02] Node and host-group vocabulary.
- [SCOPE-03] Hosted machine inventory and lifecycle read/write contracts.
- [SCOPE-04] A guest bridge attachment primitive that preserves the current
  guest protocol.
- [SCOPE-05] CLI, help text, and docs alignment for hosted targets.

### Out of Scope

- [SCOPE-06] Full SDK packaging.
- [SCOPE-07] Secrets.
- [SCOPE-08] Services and sandboxes.
- [SCOPE-09] Monitoring or `top`.
- [SCOPE-10] Detached forwards and Unix-socket forwarding.

Those remain required, but they are downstream of this epic's first voyage.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define an authenticated hosted API contract for the first control-plane slice. | GOAL-01, GOAL-02 | must | Hosted lifecycle work is not credible without auth and stable API identity. |
| FR-02 | Port must define node and host-group inventory contracts that can back hosted machine placement and lifecycle ownership. | GOAL-01, GOAL-03 | must | Hosted lifecycle control depends on explicit ownership and placement vocabulary. |
| FR-03 | Port must expose hosted machine inventory, status, and stop surfaces without inventing a second CLI model. | GOAL-01, GOAL-02 | must | The existing `machine` vocabulary is Port's strongest product asset. |
| FR-04 | Port must define the first hosted guest bridge primitive that preserves the current guest protocol for later `exec`, `copy`, `pty`, `logs`, and `forward` work. | GOAL-01, GOAL-02, GOAL-03 | must | Hosted guest operations depend on a stable bridge primitive, not ad hoc per-command transport logic. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Hosted design work must keep one canonical CLI and one canonical guest protocol. | GOAL-02 | must | Prevents local and hosted Port from diverging into separate products. |
| NFR-02 | Hosted docs, help text, and board artifacts must distinguish shipped behavior from planned follow-on work. | GOAL-01, GOAL-03 | must | Avoids overpromising while the hosted foundation is still landing. |
| NFR-03 | The first voyage must leave a coherent ordered implementation set for monitoring, secrets, services, sandboxes, detached forwarding, and SDK work. | GOAL-03 | should | Ensures this epic becomes a foundation rather than another isolated design document. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use story-level verification to prove model/runtime contracts, CLI/help text,
  and docs alignment.
- Prefer Rust tests plus CLI-level evidence for contract-bearing stories.
- Use board review to confirm that downstream hosted-control work is sequenced
  coherently after the first voyage.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Token-based auth is sufficient for the first hosted-control slice | Auth design may need broader identity work earlier | Validate during the first voyage stories |
| One hosted guest bridge primitive can preserve the current guest protocol | Hosted guest operations may fragment into per-command transports | Validate in the guest-bridge contract story |
| Node and host-group concepts are enough for the first hosted placement model | Additional scheduler concepts may be needed earlier | Reassess during follow-on voyages |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should SDK packaging land in the same epic or immediately after the API stabilizes? | Product/Architecture | Open |
| Should monitoring be part of the first hosted voyage or the next voyage after lifecycle control? | Product/Architecture | Open |
| Can the first hosted auth surface remain simple without blocking future multi-user evolution? | Architecture | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted auth, API identity, node vocabulary, and hosted lifecycle contracts are explicitly planned and decomposed into execution stories.
- [ ] The first voyage leaves the board with ready implementation work instead of another empty queue.
- [ ] Hosted CLI and docs remain coherent with the existing local/lifecycle vocabulary.
<!-- END SUCCESS_CRITERIA -->
