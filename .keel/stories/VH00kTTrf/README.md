---
# system-managed
id: VH00kTTrf
status: done
created_at: 2026-04-16T16:22:19
updated_at: 2026-04-16T18:21:28
# authored
title: Serialize Tier-1 Against Human Lifecycle Operations And Emit Events
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 3
started_at: 2026-04-16T18:17:39
submitted_at: 2026-04-16T18:21:28
completed_at: 2026-04-16T18:21:28
---

# Serialize Tier-1 Against Human Lifecycle Operations And Emit Events

## Summary

Two related concerns: do not trample human lifecycle operations, and leave a structured breadcrumb trail for every tier transition. Serialize tier-1 by having the runner `try_lock` the existing per-machine lifecycle lock that `port machine stop/launch/up` already holds; on contention, emit a `skipped_busy` event and wait for the next interval without incrementing the attempt counter. Add a JSON-per-line event sink writing to `runtime/recovery/events.log` (path configurable) with fields `machine`, `tier`, `timestamp_unix_s`, `outcome`, and a monotonic sequence number.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] `RecoveryEventSink` appends JSON-per-line events to a configured path with `machine`, `tier`, `outcome` (Started, Succeeded, Failed, SkippedBusy, SkippedNoOverlay, Tier3Escalation, Tier3HostReturned, RecoveryUnfenced), `timestamp_unix_s`, and a monotonic `sequence` number. A test emits multiple events and parses them back from disk. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- recovery_event_sink_emits_json_lines_with_monotonic_sequence, proof: ac-2.log -->
<!-- verify: manual, SRS-05:start:end -->
- [x] [SRS-05/AC-01] `try_acquire_recovery_lock` returns an RAII `RecoveryLockGuard` when the per-machine lock is free and `None` when contended; dropping the guard releases the lock. A test exercises contention, independence across machines, and post-drop re-acquisition — the runner maps `None` to a `SkippedBusy` event. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- recovery_lock_is_try_acquire_and_releases_on_guard_drop, proof: ac-2.log -->
