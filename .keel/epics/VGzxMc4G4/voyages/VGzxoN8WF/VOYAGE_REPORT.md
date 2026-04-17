# VOYAGE REPORT: Tier-3 Signal Persistence, Unfence Reset, And End-To-End Proof

## Voyage Metadata
- **ID:** VGzxoN8WF
- **Epic:** VGzxMc4G4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Persist Recovery State Across Control-Plane Restarts
- **ID:** VH01kEV1x
- **Status:** done

#### Summary
Make `recovery_state` and `recovery_attempts` durable so a control-plane restart mid-escalation does not silently re-arm the ladder against a machine that is already in `awaiting_tier_3_host_recycle`. Write each machine's record to `runtime/recovery/<machine>.json` alongside the existing registered-node state; load on startup into the in-memory recovery map. Once the machine is in `awaiting_tier_3_host_recycle`, the runner does not attempt further tier actions — it only observes heartbeats for auto-clear.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `save_recovery_record` + `load_recovery_record` round-trip a `PersistedRecoveryRecord` (state + counters + last action) through `runtime/recovery/<machine>.json` atomically (tempfile + rename). A test seeds `AwaitingTier3HostRecycle` with non-zero counters, persists it, simulates restart via a fresh load, and asserts the record reloads byte-for-byte unchanged. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime -- recovery_record_persists_and_reloads_across_control_plane_restart, proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01kEV1x/EVIDENCE/ac-1.log)

### Add Port Machine Unfence Command And Auto-Clear On Successful Launch
- **ID:** VH01kQnAB
- **Status:** done

#### Summary
Add a `port machine unfence --machine X` command that resets any non-`ok` `recovery_state` (e.g. `awaiting_tier_3_host_recycle`, `in_progress`), zeros `recovery_attempts.*`, and emits a `recovery_unfenced` event without changing any runtime state. Route it through a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. Complement that with a post-launch hook: when an operator-driven `port machine launch` succeeds and a Live guest-agent heartbeat arrives within a documented convergence budget on a machine that was in `awaiting_tier_3_host_recycle`, auto-clear the state and emit `recovery_unfenced_via_launch`. Unsuccessful launches do not clear the state.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `clear_recovery_record(record)` resets `recovery_state` to `Ok` and zeros `recovery_attempts.*`; a `RecoveryUnfenced` event is emitted to the sink. last_recovery_action stays as a breadcrumb so operators can review the transition. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- port_machine_unfence_clears_recovery_state_and_emits_event, proof: ac-2.log -->
- [x] [SRS-03/AC-01] `is_awaiting_tier_3(record)` plus `clear_recovery_record` compose into the post-launch auto-clear hook: a record in `AwaitingTier3HostRecycle` with counters >0 qualifies; clearing leaves the record in `Ok` with zero counters and the predicate returns false. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- launch_auto_clears_awaiting_tier_3_when_record_qualifies, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-01] The Cargo.toml boundary test (from VH01Fk4SW) pins `port-runtime` against any cloud-provider SDK or remote-shell dependency, so `clear_recovery_record` and the post-launch hook cannot accidentally introduce one. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- recovery_code_path_has_no_cloud_or_remote_shell_dependencies, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01kQnAB/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH01kQnAB/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH01kQnAB/EVIDENCE/ac-3.log)

### Prove Recovery Ladder End-To-End With Simulated Wedges
- **ID:** VH01kf6IY
- **Status:** done

#### Summary
Close the mission with three deterministic integration tests driving the full recovery ladder against simulated wedges. Tests use `tokio::time::pause` and channel-based event hooks on the runner so convergence does not depend on wall-clock `sleep`. The tier-3 test observes the emitted `tier_3_escalation` event and simulates host return via fresh heartbeats — there is no fake cloud client because Port doesn't have one.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `ladder_e2e_tier_1_converges_guest_wedge` composes the decision function + event sink + attempt counters and asserts tier-1 alone converges a guest wedge: wedge observed → `Tier1Restart` → heartbeats return → next tick returns `None` (back to Ok). Full event stream (`Started`, `Succeeded`) is captured. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tier_1_converges_guest_wedge, proof: ac-2.log -->
- [x] [SRS-05/AC-01] `ladder_e2e_tier_3_escalates_and_auto_clears_on_host_return` drives attempts to `tier_3_after_attempts`, observes `Tier3Escalate` decision + `Tier3Escalation` event emission, then flips `heartbeats_fresh = true` under `AwaitingTier3HostRecycle`, observes `Tier3AutoClear` + `Tier3HostReturned` event. No cloud client used. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tier_3_escalates_and_auto_clears_on_host_return, proof: ac-4.log -->
- [x] [SRS-06/AC-01] `ladder_e2e_restart_preserves_escalation_then_unfence_rearms` persists a `PersistedRecoveryRecord` in `AwaitingTier3HostRecycle`, reloads it (simulated restart), invokes `clear_recovery_record` + `RecoveryUnfenced` event + re-persist, and confirms the ladder re-arms (state back to `Ok`, counters zero). <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_restart_preserves_escalation_then_unfence_rearms, proof: ac-3.log -->
- [x] [SRS-NFR-01/AC-01] `ladder_e2e_tests_have_no_wall_clock_sleeps` is a static guard that scans the three e2e test bodies for `thread::sleep(`, `tokio::time::sleep(`, or `std::thread::sleep(` calls and fails if any are present. Determinism is a hard-coded property of this test suite. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tests_have_no_wall_clock_sleeps, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01kf6IY/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH01kf6IY/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH01kf6IY/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VH01kf6IY/EVIDENCE/ac-4.log)


