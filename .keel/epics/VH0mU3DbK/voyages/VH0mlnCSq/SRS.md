# Live Detector And Recovery Runner Wiring - SRS

## Summary

Epic: VH0mU3DbK
Goal: Spawn the wedge detector tick and recovery runner from serve_control_plane so the existing pure-function library code becomes an active runtime contract: wedge_state populates against live heartbeat ages, decide_recovery_action drives tier-1 guest restart and tier-2 overlay recreate against the runtime root, and tier-3 emits an awaiting-host-recycle signal. Recovery actions stay opt-in via ClusterRecoveryConfig.enabled.

## Scope

### In Scope

- [SCOPE-02] Spawn a periodic wedge-detector background task from `serve_control_plane` (mirroring `spawn_guest_heartbeat_probe_loop`) that reads node-agent heartbeat ages and guest heartbeat ages from `ControlPlaneStateInner`, calls `run_wedge_detector_tick` against them with the configured `ClusterDetectionConfig`, and updates `wedge_state` accordingly. Detector observation is unconditional (does not require recovery to be enabled) so consumers always see fresh wedge facts.
- [SCOPE-03] Spawn a periodic recovery-action background task from `serve_control_plane` that, for each cluster with `ClusterRecoveryConfig.enabled = true`, evaluates `decide_recovery_action` against current `wedge_state` and persisted recovery records, executes tier-1 (`port machine stop` then `launch`) and tier-2 (drop overlay then relaunch) against the runtime root, persists updated `RecoveryAttemptCounters` / `last_recovery_action`, emits the tier-3 escalation/return signal via the existing `RecoveryEventSink`, and serializes per-machine work against in-flight human lifecycle operations using `try_acquire_recovery_lock`. On successful machine launch, clear any persisted `AwaitingTier3HostRecycle` recovery record via `clear_recovery_record` so an external host-recycle signal closes the loop.
- [SCOPE-04] Wire `ClusterRecoveryConfig` from `PortConfig` per-cluster into the runner; default `enabled = false` so the wiring is a no-op for clusters that have not opted in. The detector loop runs regardless.
- [SCOPE-05] Cover the new wiring with integration tests: detector populates `wedged_since_unix_s` after the configured trigger age; recovery runner drives a simulated wedge through tier-1 → ok when `enabled = true`; recovery runner is a no-op when `enabled = false`; lock contention causes the tick to skip and retry; existing `serve_control_plane` tests stay green.

### Out of Scope

- [SCOPE-01] Threading wedge fields onto `HostedK3sMachineTruth` — owned by voyage VH0mjMP8p.
- [SCOPE-06] Adding new wedge or recovery field definitions, new ladder tiers, or new detector classes — out of epic scope.
- [SCOPE-07] Cross-cluster aggregation, alerting, dashboard, or UI surfaces — out of epic scope.
- [SCOPE-08] Guest-side kernel watchdog or OS-level liveness — out of epic scope.
- [SCOPE-09] Consumer-side wiring in `spoke-sh/infra` — out of epic scope.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `serve_control_plane` spawns a periodic detector task that calls `run_wedge_detector_tick` against current heartbeat ages and updates `wedge_state`; the detector runs unconditionally and produces wedge facts for any cluster regardless of `ClusterRecoveryConfig.enabled`. | SCOPE-02 | FR-01 | integration |
| SRS-02 | `serve_control_plane` spawns a periodic recovery-action task that, when `ClusterRecoveryConfig.enabled = true` for a cluster, evaluates `decide_recovery_action` against `wedge_state` and the persisted recovery record, executes tier-1 / tier-2 / tier-3 signal as the decision dictates, persists updated counters and the latest action record, and emits the corresponding `RecoveryEventSink` event. | SCOPE-03 | FR-01 | integration |
| SRS-03 | Recovery-action execution acquires the per-machine lock via `try_acquire_recovery_lock`; if contended, the tick logs the contention, skips that machine, and reevaluates next interval. No tier-1/2 action ever runs concurrently with another recovery action or human-driven `port machine stop/launch` on the same machine. | SCOPE-03 | FR-01 | unit |
| SRS-04 | When `ClusterRecoveryConfig.enabled = false` for a cluster, the recovery-action task is a no-op for that cluster: no actions fire, no records persist, no events emit. The detector still updates `wedge_state` for that cluster's machines. | SCOPE-04 | FR-01 | integration |
| SRS-05 | Successful `launch_local_machine` clears any persisted `AwaitingTier3HostRecycle` record for that machine via `clear_recovery_record`, so an external host recycle that ends with a fresh launch closes the recovery loop and resets the ladder state. | SCOPE-03 | FR-01 | unit |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Detector and recovery tasks must not block the control-plane HTTP serving loop; both run as separate `thread::spawn` workers (mirroring `spawn_guest_heartbeat_probe_loop`) with their own runtimes if async work is needed. | SCOPE-02 | NFR-01 | unit |
| SRS-NFR-02 | A panic inside the detector or recovery worker must not take down `serve_control_plane`; each tick is wrapped so per-tick errors are logged and the loop continues. | SCOPE-02 | NFR-01 | unit |
| SRS-NFR-03 | The tick interval and detector-trigger thresholds are configurable via existing `ClusterDetectionConfig` / `ClusterRecoveryConfig` validation paths so tests can run with low values without hard-coded sleeps. | SCOPE-05 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
