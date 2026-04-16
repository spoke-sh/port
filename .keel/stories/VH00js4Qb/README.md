---
# system-managed
id: VH00js4Qb
status: icebox
created_at: 2026-04-16T16:22:17
updated_at: 2026-04-16T16:22:17
# authored
title: Add Recovery Config Block And Attempt Counter Fields
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 1
---

# Add Recovery Config Block And Attempt Counter Fields

## Summary

Introduce the config surface the recovery runner reads from. Add `[clusters.<name>.recovery]` with `enabled: bool` (default `false`) and `settle_seconds: u64`. Extend the per-machine status contract with `recovery_attempts { tier_1, tier_2, tier_3 }`, `last_recovery_action { tier, timestamp_unix_s, outcome }`, and `recovery_state: "ok" | "in_progress" | "disabled"` (the full state enum lands in later voyages). Clusters with no `[recovery]` block behave identically to `enabled = false`.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port-model` defines `ClusterRecoveryConfig` parsed from `[clusters.<name>.recovery]` with `enabled: bool` and `settle_seconds: u64`; the per-machine status struct grows `recovery_attempts`, `last_recovery_action`, and `recovery_state` (skipped when default); `port cluster status --format json` exposes the fields and a doctor check validates config values. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -p port-runtime -- cluster_recovery_config_and_status_fields, proof: ac-1.log -->
- [ ] [SRS-NFR-01/AC-01] A cluster with no `[recovery]` block behaves exactly as `enabled = false`; an integration test starts a control plane without the block, seeds a guest-side wedge via the detector, and asserts no tier-1 action fires and `recovery_state` stays `"disabled"`. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- recovery_disabled_by_default_for_missing_block, proof: ac-2.log -->
