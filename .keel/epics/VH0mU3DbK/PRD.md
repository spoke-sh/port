# Wire Wedge Detection And Recovery Into Live Control Plane - Product Requirements

## Problem Statement

Mission VGzwzdKvB shipped wedge detection and a three-tier recovery ladder as library-level functions, but neither the detector tick nor the recovery runner is invoked from the live serve_control_plane process, and the per-machine wedge fields are only on MachineStatus rather than on the HostedK3sMachineTruth entries inside port cluster status --format json. Consumers (notably spoke-sh/infra) cannot see wedged_since on the cluster aggregate they already poll, and Port itself never executes the recovery ladder against a wedged guest at runtime. Close the deferred wiring so the existing internals become an active runtime contract.

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

- [SCOPE-01] Thread the per-machine wedge fields (`guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, `recovery_state`) onto `HostedK3sMachineTruth` so consumers reading `port cluster status --format json` see them on the cluster aggregate, not only via per-machine `port machine status` calls.
- [SCOPE-02] Spawn the wedge-detector tick loop from `serve_control_plane` so the existing `run_wedge_detector_tick` actually populates `wedge_state` at runtime against live heartbeat ages, instead of only being exercised in unit tests.
- [SCOPE-03] Spawn the recovery runner from `serve_control_plane` so `decide_recovery_action` is invoked against live wedge facts and tier-1 guest restart, tier-2 overlay recreate, and tier-3 awaiting-host-recycle signal fire when `ClusterRecoveryConfig.enabled = true`.
- [SCOPE-04] Wire `ClusterRecoveryConfig` from `PortConfig` per-cluster into the runner; default `enabled = false` so the wiring change is a no-op for clusters that have not opted in.
- [SCOPE-05] Cover the new live wiring with integration tests (cluster-aggregate JSON shape, detector loop populating wedge fields, recovery runner driving a simulated wedge through the ladder) and keep existing suites green.

### Out of Scope

- [SCOPE-06] Adding new wedge fields, new recovery tiers, or new detector classes — this epic ships only the wiring of what already exists.
- [SCOPE-07] Cross-cluster aggregation, alerting, dashboards, or UI surfaces.
- [SCOPE-08] Guest-side kernel watchdog or OS-level liveness; agent-layer signals only.
- [SCOPE-09] Consumer-side integration in `spoke-sh/infra` — that lands in the consumer repo against this epic's contract.

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
