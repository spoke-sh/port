# VOYAGE REPORT: Tier-1 Guest Restart And Attempt Accounting

## Voyage Metadata
- **ID:** VGzxmpqrI
- **Epic:** VGzxMc4G4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Recovery Config Block And Attempt Counter Fields
- **ID:** VH00js4Qb
- **Status:** done

#### Summary
Introduce the config surface the recovery runner reads from. Add `[clusters.<name>.recovery]` with `enabled: bool` (default `false`) and `settle_seconds: u64`. Extend the per-machine status contract with `recovery_attempts { tier_1, tier_2, tier_3 }`, `last_recovery_action { tier, timestamp_unix_s, outcome }`, and `recovery_state: "ok" | "in_progress" | "disabled"` (the full state enum lands in later voyages). Clusters with no `[recovery]` block behave identically to `enabled = false`.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` defines `ClusterRecoveryConfig` parsed from `[clusters.<name>.recovery]` with `enabled: bool` and `settle_seconds: u64` (default 60); `MachineStatus` grows `recovery_attempts` (`RecoveryAttemptCounters`), `last_recovery_action` (`Option<RecoveryActionRecord>`), and `recovery_state` (`RecoveryState::Ok|InProgress|Disabled`), all skipped when default. Validation rejects `settle_seconds = 0` with an actionable error. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->
- [x] [SRS-NFR-01/AC-01] `[recovery]` absent from `ClusterSpec` decodes as `ClusterRecoveryConfig::default()` with `enabled = false`; test confirms the absent-block and explicit-false cases produce identical config state. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00js4Qb/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH00js4Qb/EVIDENCE/ac-2.log)

### Fire Tier-1 Guest Restart From Wedge Detector Output
- **ID:** VH00kDFiS
- **Status:** done

#### Summary
Wire the recovery runner. On each detector cycle, the runner scans `wedge_state` for entries with `wedge_class = "guest"` and fires tier-1 through the owning node-agent: `port machine stop` then `port machine launch` against the same runtime root. After `settle_seconds`, re-read the wedge state; if the guest heartbeat has returned, transition `recovery_state` to `"ok"` and stamp `last_recovery_action` with outcome `"succeeded"`. The runner must re-read the wedge state immediately before executing to avoid acting on a stale trigger that cleared between detector tick and action.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] With `recovery.enabled = true` and a guest-side wedge observed, the pure `decide_recovery_action` function returns `Some(Tier1Restart)`; with `enabled = false` it returns `None` regardless of wedge state. The runner consumes this decision to drive the node-agent's stop-then-launch path. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- recovery_decision_fires_tier_1_on_guest_wedge_when_enabled, proof: ac-2.log -->
- [x] [SRS-03/AC-01] The decision function promotes through the ladder based on cumulative counters: tier_1 under threshold → Tier1Restart; tier_1 meets `tier_2_after_attempts` → Tier2Recreate; cumulative meets `tier_3_after_attempts` → Tier3Escalate. A node-side wedge jumps straight to Tier3Escalate. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- recovery_decision_promotes_tier_1_to_tier_2_and_tier_3, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-01] The pure decision function returns `None` when wedge state is absent at decision time, so a stale detector read that cleared before the runner executes produces no action, no event, and no counter change. The runner re-reads wedge_state immediately before executing to absorb this race. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- recovery_decision_re_reads_wedge_state_avoiding_stale_trigger, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00kDFiS/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH00kDFiS/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH00kDFiS/EVIDENCE/ac-3.log)

### Serialize Tier-1 Against Human Lifecycle Operations And Emit Events
- **ID:** VH00kTTrf
- **Status:** done

#### Summary
Two related concerns: do not trample human lifecycle operations, and leave a structured breadcrumb trail for every tier transition. Serialize tier-1 by having the runner `try_lock` the existing per-machine lifecycle lock that `port machine stop/launch/up` already holds; on contention, emit a `skipped_busy` event and wait for the next interval without incrementing the attempt counter. Add a JSON-per-line event sink writing to `runtime/recovery/events.log` (path configurable) with fields `machine`, `tier`, `timestamp_unix_s`, `outcome`, and a monotonic sequence number.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `RecoveryEventSink` appends JSON-per-line events to a configured path with `machine`, `tier`, `outcome` (Started, Succeeded, Failed, SkippedBusy, SkippedNoOverlay, Tier3Escalation, Tier3HostReturned, RecoveryUnfenced), `timestamp_unix_s`, and a monotonic `sequence` number. A test emits multiple events and parses them back from disk. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- recovery_event_sink_emits_json_lines_with_monotonic_sequence, proof: ac-2.log -->
- [x] [SRS-05/AC-01] `try_acquire_recovery_lock` returns an RAII `RecoveryLockGuard` when the per-machine lock is free and `None` when contended; dropping the guard releases the lock. A test exercises contention, independence across machines, and post-drop re-acquisition — the runner maps `None` to a `SkippedBusy` event. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- recovery_lock_is_try_acquire_and_releases_on_guard_drop, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00kTTrf/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH00kTTrf/EVIDENCE/ac-2.log)


