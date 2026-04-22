# Harden Hosted Wedge Detection And Runtime Recovery - Product Requirements

## Problem Statement

Prod exposed two gaps in hosted recovery: managed k3s services do not restart on unhealthy healthchecks, and guest wedge classification falls back to machine placement age, producing false positives that make the wedge endpoint unsafe for auto-recovery.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Restore Port's ability to recover hosted K3s guest runtime failures without requiring manual guest intervention. | Hosted K3s managed services restart automatically when their healthchecks fail and tests prove the policy | `hosted_k3s_service_policy` enables unhealthy restarts and supervisor tests stay green |
| GOAL-02 | Eliminate false positive guest wedges caused by machine-age fallback when guest refresh metadata is absent. | Healthy running machines without guest refresh age no longer classify as `guest` wedged in detector and endpoint tests | Hosted wedge tests show no guest wedge without live heartbeat/runtime evidence |
| GOAL-03 | Raise wedge endpoint fidelity so downstream auto-recovery can trust the signal. | `/v1/machines/<name>/wedge` includes concrete recovery evidence describing guest refresh age and managed K3s runtime state | Endpoint serialization/tests cover the new fields and evidence mapping |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Engineer operating hosted Port clusters and triaging recovery events. | A wedge/recovery surface that accurately identifies real guest runtime failures and self-heals the recoverable ones. |
| Auto-Recovery Consumer | Downstream automation that reads Port's wedge endpoint to decide whether to restart or recreate machines. | High-fidelity machine wedge evidence that avoids acting on false positives. |

## Scope

### In Scope

- [SCOPE-01] Hosted K3s managed-service policy changes needed to restart on unhealthy healthchecks.
- [SCOPE-02] Hosted wedge classification changes that stop using machine placement age as a guest-heartbeat surrogate.
- [SCOPE-03] Machine wedge endpoint and detector evidence improvements needed to expose trustworthy recovery inputs.
- [SCOPE-04] Focused tests proving unhealthy restart, false-positive suppression, and endpoint fidelity.

### Out of Scope

- [SCOPE-05] New cloud-provider lifecycle integrations or changes to who owns tier-3 host recycle.
- [SCOPE-06] Rebalancing stateful workloads after a worker loss.
- [SCOPE-07] Broad hosted fleet scheduling or storage resilience work unrelated to the wedge signal itself.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Hosted K3s managed services must restart automatically when their configured healthcheck reports unhealthy. | GOAL-01 | must | Prevents dead-but-still-running guest runtime processes from requiring manual intervention. |
| FR-02 | Guest wedge classification must only use real guest heartbeat age or explicit managed-runtime failure evidence; machine placement age cannot stand in for guest refresh age. | GOAL-02, GOAL-03 | must | Prevents healthy long-lived machines from being misclassified as guest wedges. |
| FR-03 | The machine wedge endpoint must expose the evidence Port used to classify the wedge, including guest refresh age when present and hosted K3s service/runtime status details when available. | GOAL-03 | must | Gives downstream automation and operators a trustworthy basis for recovery decisions. |
| FR-04 | Recovery action selection must not repeatedly escalate while a recovery attempt is already in progress within its settle window. | GOAL-01, GOAL-03 | should | Avoids compounding actions while Port is still waiting to see whether a recovery changed runtime state. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | New wedge evidence must be deterministic and machine-readable so downstream automation can rely on it without heuristic scraping. | GOAL-02, GOAL-03 | must | Keeps the endpoint safe for automated recovery. |
| NFR-02 | The fix must be covered by focused unit/integration tests in the hosted runtime and control-plane layers. | GOAL-01, GOAL-02, GOAL-03 | must | This regression path is subtle and must stay pinned by tests. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Hosted K3s restart policy | Targeted Rust tests for service policy and supervisor behavior | `cargo test hosted_k3s_service_policy` and adjacent guest-agent/service tests |
| Wedge classification | Targeted Rust tests in `hosted_control_plane` | New/updated wedge detector and recovery decision tests |
| Endpoint fidelity | Route serialization/assertion tests for `/v1/machines/<name>/wedge` | Endpoint response tests covering evidence fields |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Hosted K3s service healthchecks are already the right signal for runtime liveness in guest recovery. | If false, unhealthy restart could flap or miss the real failure. | Validate against current service policy and failing production symptom. |
| Downstream automation can consume additional wedge JSON fields without requiring backward-compatibility shims inside Port. | If false, endpoint changes may need a coordinated consumer rollout. | Keep the change additive and validate current callers if needed. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Could enabling unhealthy restarts on hosted K3s server processes hide a deeper host or disk fault? | Epic owner | Mitigated via existing recovery ladder and explicit runtime evidence |
| Do any current consumers depend on the false-positive `guest` wedge behavior? | Epic owner | Low risk; current behavior is already unsafe for auto-recovery |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted K3s unhealthy services restart under the managed-service policy without manual guest intervention.
- [ ] Healthy running hosted machines are not classified as `guest` wedged solely because they have been placed for longer than the guest heartbeat threshold.
- [ ] The machine wedge endpoint returns explicit runtime evidence that explains Port's current wedge classification and recovery posture.
<!-- END SUCCESS_CRITERIA -->
