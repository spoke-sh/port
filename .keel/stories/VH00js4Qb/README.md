---
# system-managed
id: VH00js4Qb
status: done
created_at: 2026-04-16T16:22:17
updated_at: 2026-04-16T17:25:14
# authored
title: Add Recovery Config Block And Attempt Counter Fields
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 1
started_at: 2026-04-16T17:25:13
submitted_at: 2026-04-16T17:25:14
completed_at: 2026-04-16T17:25:14
---

# Add Recovery Config Block And Attempt Counter Fields

## Summary

Introduce the config surface the recovery runner reads from. Add `[clusters.<name>.recovery]` with `enabled: bool` (default `false`) and `settle_seconds: u64`. Extend the per-machine status contract with `recovery_attempts { tier_1, tier_2, tier_3 }`, `last_recovery_action { tier, timestamp_unix_s, outcome }`, and `recovery_state: "ok" | "in_progress" | "disabled"` (the full state enum lands in later voyages). Clusters with no `[recovery]` block behave identically to `enabled = false`.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `port-model` defines `ClusterRecoveryConfig` parsed from `[clusters.<name>.recovery]` with `enabled: bool` and `settle_seconds: u64` (default 60); `MachineStatus` grows `recovery_attempts` (`RecoveryAttemptCounters`), `last_recovery_action` (`Option<RecoveryActionRecord>`), and `recovery_state` (`RecoveryState::Ok|InProgress|Disabled`), all skipped when default. Validation rejects `settle_seconds = 0` with an actionable error. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-01/AC-01] `[recovery]` absent from `ClusterSpec` decodes as `ClusterRecoveryConfig::default()` with `enabled = false`; test confirms the absent-block and explicit-false cases produce identical config state. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->
