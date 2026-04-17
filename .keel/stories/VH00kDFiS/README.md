---
# system-managed
id: VH00kDFiS
status: done
created_at: 2026-04-16T16:22:18
updated_at: 2026-04-16T18:17:22
# authored
title: Fire Tier-1 Guest Restart From Wedge Detector Output
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 2
started_at: 2026-04-16T18:14:36
submitted_at: 2026-04-16T18:17:22
completed_at: 2026-04-16T18:17:22
---

# Fire Tier-1 Guest Restart From Wedge Detector Output

## Summary

Wire the recovery runner. On each detector cycle, the runner scans `wedge_state` for entries with `wedge_class = "guest"` and fires tier-1 through the owning node-agent: `port machine stop` then `port machine launch` against the same runtime root. After `settle_seconds`, re-read the wedge state; if the guest heartbeat has returned, transition `recovery_state` to `"ok"` and stamp `last_recovery_action` with outcome `"succeeded"`. The runner must re-read the wedge state immediately before executing to avoid acting on a stale trigger that cleared between detector tick and action.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] With `recovery.enabled = true` and a guest-side wedge observed, the pure `decide_recovery_action` function returns `Some(Tier1Restart)`; with `enabled = false` it returns `None` regardless of wedge state. The runner consumes this decision to drive the node-agent's stop-then-launch path. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- recovery_decision_fires_tier_1_on_guest_wedge_when_enabled, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [x] [SRS-03/AC-01] The decision function promotes through the ladder based on cumulative counters: tier_1 under threshold → Tier1Restart; tier_1 meets `tier_2_after_attempts` → Tier2Recreate; cumulative meets `tier_3_after_attempts` → Tier3Escalate. A node-side wedge jumps straight to Tier3Escalate. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- recovery_decision_promotes_tier_1_to_tier_2_and_tier_3, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-01] The pure decision function returns `None` when wedge state is absent at decision time, so a stale detector read that cleared before the runner executes produces no action, no event, and no counter change. The runner re-reads wedge_state immediately before executing to absorb this race. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- recovery_decision_re_reads_wedge_state_avoiding_stale_trigger, proof: ac-3.log -->
