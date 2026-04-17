---
# system-managed
id: VH01kEV1x
status: done
created_at: 2026-04-16T16:26:17
updated_at: 2026-04-16T18:33:24
# authored
title: Persist Recovery State Across Control-Plane Restarts
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 1
started_at: 2026-04-16T18:31:47
submitted_at: 2026-04-16T18:33:24
completed_at: 2026-04-16T18:33:24
---

# Persist Recovery State Across Control-Plane Restarts

## Summary

Make `recovery_state` and `recovery_attempts` durable so a control-plane restart mid-escalation does not silently re-arm the ladder against a machine that is already in `awaiting_tier_3_host_recycle`. Write each machine's record to `runtime/recovery/<machine>.json` alongside the existing registered-node state; load on startup into the in-memory recovery map. Once the machine is in `awaiting_tier_3_host_recycle`, the runner does not attempt further tier actions — it only observes heartbeats for auto-clear.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `save_recovery_record` + `load_recovery_record` round-trip a `PersistedRecoveryRecord` (state + counters + last action) through `runtime/recovery/<machine>.json` atomically (tempfile + rename). A test seeds `AwaitingTier3HostRecycle` with non-zero counters, persists it, simulates restart via a fresh load, and asserts the record reloads byte-for-byte unchanged. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime -- recovery_record_persists_and_reloads_across_control_plane_restart, proof: ac-1.log -->
