---
# system-managed
id: VH00kDFiS
status: backlog
created_at: 2026-04-16T16:22:18
updated_at: 2026-04-16T17:20:32
# authored
title: Fire Tier-1 Guest Restart From Wedge Detector Output
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 2
---

# Fire Tier-1 Guest Restart From Wedge Detector Output

## Summary

Wire the recovery runner. On each detector cycle, the runner scans `wedge_state` for entries with `wedge_class = "guest"` and fires tier-1 through the owning node-agent: `port machine stop` then `port machine launch` against the same runtime root. After `settle_seconds`, re-read the wedge state; if the guest heartbeat has returned, transition `recovery_state` to `"ok"` and stamp `last_recovery_action` with outcome `"succeeded"`. The runner must re-read the wedge state immediately before executing to avoid acting on a stale trigger that cleared between detector tick and action.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] With `recovery.enabled = true`, a guest-side wedge causes the responsible node-agent to execute `port machine stop` followed by `port machine launch` against the same runtime root; an integration test exercises this end-to-end with a fake wedge and a real node-agent. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- tier_1_stop_then_launch_converges_simulated_wedge, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] A converging tier-1 attempt increments `recovery_attempts.tier_1`, stamps `last_recovery_action = { tier: 1, timestamp_unix_s, outcome: "succeeded" }`, and transitions `recovery_state` back to `"ok"` when `wedged_since` clears. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- tier_1_accounting_on_convergence, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-01] A race-guard test: the detector sets `wedged_since`, the runner reads it, then the detector clears it before the runner executes — the runner must re-read and decline to act, emitting no event and no counter change. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- tier_1_skips_when_wedge_clears_before_action, proof: ac-3.log -->
