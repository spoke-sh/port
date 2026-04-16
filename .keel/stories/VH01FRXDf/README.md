---
# system-managed
id: VH01FRXDf
status: icebox
created_at: 2026-04-16T16:24:18
updated_at: 2026-04-16T16:24:18
# authored
title: Wire Tier-2 Overlay Recreate With Graceful Skip
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 1
---

# Wire Tier-2 Overlay Recreate With Graceful Skip

## Summary

Extend `[clusters.<name>.recovery]` with `tier_2_after_attempts` and `window_seconds`. Add the tier-1 → tier-2 promotion in the recovery runner: when `recovery_attempts.tier_1` reaches `tier_2_after_attempts` within `window_seconds` and the machine has a configured rootfs overlay, the node-agent removes `runtime/<machine>/overlay` (idempotent) and relaunches. When the machine has no overlay, emit `tier_2_skipped_no_overlay` and advance the promotion counter toward tier-3 as if tier-2 had been attempted.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `ClusterRecoveryConfig` grows `tier_2_after_attempts: u32` and `window_seconds: u64` with documented defaults; zero or negative values fail validation. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_recovery_tier_2_config, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] An integration test seeds tier-1 attempts up to `tier_2_after_attempts` without convergence; the runner then drops the overlay, relaunches, increments `recovery_attempts.tier_2`, and stamps `last_recovery_action` with `tier: 2`. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- tier_2_overlay_recreate_converges, proof: ac-2.log -->
- [ ] [SRS-03/AC-01] For a machine without `rootfs_overlay`, the promotion path emits `tier_2_skipped_no_overlay`, does not touch the filesystem, and advances the promotion counter as if tier-2 had been attempted. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- tier_2_skipped_no_overlay_advances_to_tier_3, proof: ac-3.log -->
- [ ] [SRS-NFR-01/AC-01] The overlay drop is idempotent — running tier-2 against an already-cleared overlay returns success without error. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- tier_2_overlay_drop_is_idempotent, proof: ac-4.log -->
