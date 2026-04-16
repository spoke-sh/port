# Wedge Detection And Guest Heartbeat Surface - Product Requirements

## Problem Statement

Hosted fleet operators cannot tell when a microVM guest is wedged: node-agent refresh age only reports node-side liveness, not guest-agent liveness, so stale or silent guests hide behind a Live node. Without a signal that distinguishes a node-side wedge (node-agent silent) from a guest-side wedge (node-agent healthy, guest-agent silent), Port cannot drive tier-appropriate recovery. Introduce a guest-agent heartbeat and a wedge detector that surfaces wedged_since, wedge_class, recovery_attempts, last_recovery_action, and recovery_state on the hosted cluster status contract, without taking recovery actions yet.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Resolve the problem described above for the primary user. | A measurable outcome is defined for this problem | Target agreed during planning |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | The person or team most affected by the problem above. | A clearer path to the outcome this epic should improve. |

## Scope

### In Scope

- [SCOPE-01] Introduce a `Ping`/`Pong` frame pair in `port-agent-protocol` and a matching handler in `port-guest-agent` so the node-agent can prove a guest-agent read-loop is awake without reusing `Exec`.
- [SCOPE-02] Drive a periodic per-machine guest-agent probe from the node-agent that stamps `guest_agent_last_heartbeat` on every successful pong, independent of in-flight guest operations.
- [SCOPE-03] Extend the hosted cluster status contract to surface `guest_refresh_age_seconds` and the wedge fields `wedged_since`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, `recovery_state` per machine.
- [SCOPE-04] Implement a configurable wedge detector that consumes both `refresh_age_seconds` (node-side) and `guest_refresh_age_seconds` (guest-side) and writes the wedge fields without taking any recovery action.
- [SCOPE-05] Cover the new protocol frames, probe loop, status fields, and detector with unit and integration tests, and keep existing guest-operation suites green.

### Out of Scope

- [SCOPE-06] Any recovery action (tiers 1/2/3, host reboot, unfence reset) — owned by epic VGzxMc4G4.
- [SCOPE-07] Cross-cluster aggregation, alerting, dashboard, or UI surfaces.
- [SCOPE-08] Guest-side kernel watchdog or OS-level liveness probes; this epic owns only the agent-layer heartbeat.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Deliver the primary user workflow for this epic end-to-end. | GOAL-01 | must | Establishes the minimum functional capability needed to achieve the epic goal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01 | must | Keeps operations stable and makes regressions detectable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
