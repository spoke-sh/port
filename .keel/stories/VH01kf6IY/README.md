---
# system-managed
id: VH01kf6IY
status: done
created_at: 2026-04-16T16:26:18
updated_at: 2026-04-16T18:38:17
# authored
title: Prove Recovery Ladder End-To-End With Simulated Wedges
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 3
started_at: 2026-04-16T18:35:34
submitted_at: 2026-04-16T18:38:17
completed_at: 2026-04-16T18:38:17
---

# Prove Recovery Ladder End-To-End With Simulated Wedges

## Summary

Close the mission with three deterministic integration tests driving the full recovery ladder against simulated wedges. Tests use `tokio::time::pause` and channel-based event hooks on the runner so convergence does not depend on wall-clock `sleep`. The tier-3 test observes the emitted `tier_3_escalation` event and simulates host return via fresh heartbeats — there is no fake cloud client because Port doesn't have one.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] `ladder_e2e_tier_1_converges_guest_wedge` composes the decision function + event sink + attempt counters and asserts tier-1 alone converges a guest wedge: wedge observed → `Tier1Restart` → heartbeats return → next tick returns `None` (back to Ok). Full event stream (`Started`, `Succeeded`) is captured. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tier_1_converges_guest_wedge, proof: ac-2.log -->
<!-- verify: manual, SRS-05:start:end, proof: ac-3.log-->
- [x] [SRS-05/AC-01] `ladder_e2e_tier_3_escalates_and_auto_clears_on_host_return` drives attempts to `tier_3_after_attempts`, observes `Tier3Escalate` decision + `Tier3Escalation` event emission, then flips `heartbeats_fresh = true` under `AwaitingTier3HostRecycle`, observes `Tier3AutoClear` + `Tier3HostReturned` event. No cloud client used. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tier_3_escalates_and_auto_clears_on_host_return, proof: ac-4.log -->
<!-- verify: manual, SRS-06:start:end -->
- [x] [SRS-06/AC-01] `ladder_e2e_restart_preserves_escalation_then_unfence_rearms` persists a `PersistedRecoveryRecord` in `AwaitingTier3HostRecycle`, reloads it (simulated restart), invokes `clear_recovery_record` + `RecoveryUnfenced` event + re-persist, and confirms the ladder re-arms (state back to `Ok`, counters zero). <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_restart_preserves_escalation_then_unfence_rearms, proof: ac-3.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-01/AC-01] `ladder_e2e_tests_have_no_wall_clock_sleeps` is a static guard that scans the three e2e test bodies for `thread::sleep(`, `tokio::time::sleep(`, or `std::thread::sleep(` calls and fails if any are present. Determinism is a hard-coded property of this test suite. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- ladder_e2e_tests_have_no_wall_clock_sleeps, proof: ac-4.log -->
