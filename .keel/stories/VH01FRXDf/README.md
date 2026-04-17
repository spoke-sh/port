---
# system-managed
id: VH01FRXDf
status: done
created_at: 2026-04-16T16:24:18
updated_at: 2026-04-16T18:26:09
# authored
title: Wire Tier-2 Overlay Recreate With Graceful Skip
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 1
started_at: 2026-04-16T18:25:15
submitted_at: 2026-04-16T18:26:09
completed_at: 2026-04-16T18:26:09
---

# Wire Tier-2 Overlay Recreate With Graceful Skip

## Summary

Extend `[clusters.<name>.recovery]` with `tier_2_after_attempts` and `window_seconds`. Add the tier-1 → tier-2 promotion in the recovery runner: when `recovery_attempts.tier_1` reaches `tier_2_after_attempts` within `window_seconds` and the machine has a configured rootfs overlay, the node-agent removes `runtime/<machine>/overlay` (idempotent) and relaunches. When the machine has no overlay, emit `tier_2_skipped_no_overlay` and advance the promotion counter toward tier-3 as if tier-2 had been attempted.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `ClusterRecoveryConfig` grows `tier_2_after_attempts: u32`, `tier_3_after_attempts: u32`, and `window_seconds: u64` with documented defaults (2, 4, 1800). Zero values fail validation with an actionable error message. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
- [x] [SRS-02/AC-01] The decision function promotes to `Tier2Recreate` once `recovery_attempts.tier_1` reaches `tier_2_after_attempts`; the runner uses `drop_machine_rootfs_overlay` to remove the overlay before relaunching. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- drop_machine_rootfs_overlay_is_idempotent, proof: ac-4.log -->
<!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-03/AC-01] `machine_has_rootfs_overlay(config, machine_name)` returns `false` when no overlay spec is set, driving the runner's `SkippedNoOverlay` path which advances promotion toward tier-3 without touching the filesystem. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- machine_has_rootfs_overlay_checks_machine_spec, proof: ac-3.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-01/AC-01] `drop_machine_rootfs_overlay` is idempotent: running it against an already-cleared overlay path returns `Ok(())` without error. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- drop_machine_rootfs_overlay_is_idempotent, proof: ac-4.log -->
