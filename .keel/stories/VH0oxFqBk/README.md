---
# system-managed
id: VH0oxFqBk
status: icebox
created_at: 2026-04-16T19:41:45
updated_at: 2026-04-16T19:41:45
# authored
title: Spawn Recovery Runner Loop With Tier-1 Through Tier-3
type: feat
operator-signal:
scope: VH0mU3DbK/VH0mlnCSq
index: 2
---

# Spawn Recovery Runner Loop With Tier-1 Through Tier-3

## Summary

Spawn the recovery runner as a second `thread::spawn` background worker from `build_state` inside `serve_control_plane`. Per cluster with `ClusterRecoveryConfig.enabled = true`, the worker reads `wedge_state` populated by the detector worker (sibling story `VH0owEJfH`), evaluates `decide_recovery_action` against the persisted recovery record, executes tier-1 (`stop_machine` + `launch_local_machine`) or tier-2 (`drop_machine_rootfs_overlay` + `launch_local_machine`) against the runtime root, persists the updated counters / latest action via `save_recovery_record`, and emits the corresponding event through `RecoveryEventSink`. Tier-3 emits the `awaiting_tier_3_host_recycle` signal via the existing `emit_tier_3_escalation` helper. Per-machine serialization with human lifecycle ops uses the existing `try_acquire_recovery_lock` RAII guard. Successful launches clear any persisted `AwaitingTier3HostRecycle` record via `clear_recovery_record` so an external host recycle closes the loop.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] With `ClusterRecoveryConfig.enabled = true` and a wedged machine present in `wedge_state`, one recovery tick executes the tier dictated by `decide_recovery_action`, persists the updated record via `save_recovery_record`, and emits a matching event through `RecoveryEventSink`. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime recovery_runner_executes_decided_action, proof: ac-1.log -->
- [ ] [SRS-03/AC-02] If `try_acquire_recovery_lock` returns `None` for a machine, the tick logs the contention, skips that machine, and proceeds to the next; no recovery action runs while the lock is contended. <!-- [SRS-03/AC-02] verify: cargo test -p port-runtime recovery_runner_skips_locked_machine, proof: ac-2.log -->
- [ ] [SRS-04/AC-03] With `ClusterRecoveryConfig.enabled = false`, the recovery worker is a no-op for that cluster: no actions fire, no records persist, no events emit; the detector worker still updates `wedge_state` for the same machines. <!-- [SRS-04/AC-03] verify: cargo test -p port-runtime recovery_runner_disabled_is_noop, proof: ac-3.log -->
- [ ] [SRS-05/AC-04] Successful `launch_local_machine` (via the hosted control-plane launch path) clears any persisted `AwaitingTier3HostRecycle` record for that machine via `clear_recovery_record`. <!-- [SRS-05/AC-04] verify: cargo test -p port-runtime launch_clears_awaiting_tier_3_record, proof: ac-4.log -->
