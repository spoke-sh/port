---
# system-managed
id: VH00kTTrf
status: backlog
created_at: 2026-04-16T16:22:19
updated_at: 2026-04-16T17:20:32
# authored
title: Serialize Tier-1 Against Human Lifecycle Operations And Emit Events
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxmpqrI
index: 3
---

# Serialize Tier-1 Against Human Lifecycle Operations And Emit Events

## Summary

Two related concerns: do not trample human lifecycle operations, and leave a structured breadcrumb trail for every tier transition. Serialize tier-1 by having the runner `try_lock` the existing per-machine lifecycle lock that `port machine stop/launch/up` already holds; on contention, emit a `skipped_busy` event and wait for the next interval without incrementing the attempt counter. Add a JSON-per-line event sink writing to `runtime/recovery/events.log` (path configurable) with fields `machine`, `tier`, `timestamp_unix_s`, `outcome`, and a monotonic sequence number.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] Every tier-1 attempt emits a JSON event (`{started, succeeded, failed, skipped_busy}`) to the configured sink with `machine`, `tier`, `timestamp_unix_s`, `outcome`, and a monotonic sequence number; a test captures the sink output and asserts the full event stream for a converging recovery. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- tier_1_event_stream_covers_all_outcomes, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] A test holds the per-machine lifecycle lock from a simulated human `port machine stop` and asserts the tier-1 runner `try_lock`s, emits `skipped_busy`, leaves the counter unchanged, and retries successfully on the next settling interval once the lock is released. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- tier_1_skipped_when_human_stop_holds_lock, proof: ac-2.log -->
