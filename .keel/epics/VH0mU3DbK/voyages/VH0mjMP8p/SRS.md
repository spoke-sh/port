# Cluster Aggregate Wedge Field Threading - SRS

## Summary

Epic: VH0mU3DbK
Goal: Thread the per-machine wedge fields onto HostedK3sMachineTruth so consumers polling port cluster status --format json see wedged_since, wedge_class, recovery_attempts, last_recovery_action, recovery_state, and guest_refresh_age_seconds on the cluster aggregate without needing per-machine port machine status calls.

## Scope

### In Scope

- [SCOPE-01] Thread the six per-machine wedge fields (`guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, `recovery_state`) onto `HostedK3sMachineTruth` with `#[serde(default, skip_serializing_if = ...)]` so older payloads still decode against newer consumers and absent fields stay out of the wire format. Populate the new fields per machine inside the cluster-status build path by reusing the live `machine_status` route (mirroring how `hosted_k3s_managed_service_truth` already calls `list_machine_services` per machine), so the running control-plane's `wedge_state` map remains the single source of truth. Render the same fields in the human-readable `print_cluster_status_report` so the text output mirrors the JSON shape.
- [SCOPE-05] Cover the new fields with serde round-trip and population tests (baseline of all defaults, and populated case with a wedged machine surfacing `wedged_since_unix_s = Some(_)`), and add a render test that the text output emits `(none)` lines when fields are absent. Keep existing cluster-status suites green.

### Out of Scope

- [SCOPE-02] Spawning the wedge detector loop — owned by voyage VH0mlnCSq.
- [SCOPE-03] Spawning the recovery runner — owned by voyage VH0mlnCSq.
- [SCOPE-04] Wiring `ClusterRecoveryConfig` opt-in — owned by voyage VH0mlnCSq.
- [SCOPE-06] Adding new wedge or recovery field definitions — out of epic scope.
- [SCOPE-07] Cross-cluster aggregation, alerting, dashboards, or UI surfaces — out of epic scope.
- [SCOPE-08] Guest-side kernel watchdog or OS-level liveness — out of epic scope.
- [SCOPE-09] Consumer-side adoption in `spoke-sh/infra` — out of epic scope.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `HostedK3sMachineTruth` carries `guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` with serde defaults so older payloads decode unchanged and absent fields stay off the wire. | SCOPE-01 | FR-01 | unit |
| SRS-02 | The cluster-status build path populates the new fields per machine from the live `machine_status` route, sharing the same `wedge_state` and recovery records that back `MachineStatus`; a wedged machine surfaces `wedged_since_unix_s = Some(_)` on the cluster aggregate. | SCOPE-01 | FR-01 | integration |
| SRS-03 | `print_cluster_status_report` renders the new fields as additional lines per machine in the text output, with `(none)` for absent values, mirroring the existing `print_machine_status` rendering. | SCOPE-01 | FR-01 | unit |
| SRS-04 | The new fields and their population are covered by serde round-trip, population, and render tests; existing cluster-status suites continue to pass. | SCOPE-05 | FR-01 | unit |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The per-machine `machine_status` calls used to populate the new fields are best-effort: if a call errors, the new fields stay at their serde defaults and the rest of the truth row builds unchanged, mirroring the existing `Unreachable` fallback in `hosted_k3s_managed_service_truth`. | SCOPE-01 | NFR-01 | unit |
| SRS-NFR-02 | Existing consumers on prior pins must decode the new payload without error; new fields default to absent on the wire and to `RecoveryState::Ok` / empty counters on decode. | SCOPE-01 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
