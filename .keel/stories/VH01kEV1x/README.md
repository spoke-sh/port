---
# system-managed
id: VH01kEV1x
status: icebox
created_at: 2026-04-16T16:26:17
updated_at: 2026-04-16T16:26:17
# authored
title: Add Sticky Recovery Exhausted State Across Window Rollovers
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 1
---

# Add Sticky Recovery Exhausted State Across Window Rollovers

## Summary

Introduce `recovery_state = "exhausted"` as the ladder's terminal state. Persist it to `runtime/recovery/<machine>.json` alongside the existing registered-node state so it survives control-plane restarts. Transition into `exhausted` when tier-3 has fired without convergence, or when tier-3 has been suppressed and no further tiers remain. Once set, the recovery runner stops acting on the machine, even as the attempt-counter window rolls over.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] An integration test drives the full ladder without convergence and asserts `recovery_state` transitions to `"exhausted"`, no further tier actions fire, and the state persists to `runtime/recovery/<machine>.json`. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime -- recovery_exhausted_is_terminal, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] With `recovery_state = "exhausted"`, rolling the attempt-counter window (via injectable clock) does not re-arm the ladder; a test asserts `recovery_attempts.*` can reset to zero without triggering any tier-1 action on the exhausted machine. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- recovery_exhausted_survives_window_rollover, proof: ac-2.log -->
- [ ] [SRS-NFR-01/AC-01] A test restarts the control plane while a machine is `exhausted` and asserts the state reloads from disk with no change to `recovery_attempts`, `last_recovery_action`, or the exhaustion flag. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- recovery_exhausted_persists_across_control_plane_restart, proof: ac-3.log -->
