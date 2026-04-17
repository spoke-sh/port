---
# system-managed
id: VH01FzHcw
status: done
created_at: 2026-04-16T16:24:21
updated_at: 2026-04-16T18:31:34
# authored
title: Auto-Clear Tier-3 Escalation When Host Returns
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 3
started_at: 2026-04-16T18:29:59
submitted_at: 2026-04-16T18:31:34
completed_at: 2026-04-16T18:31:34
---

# Auto-Clear Tier-3 Escalation When Host Returns

## Summary

Port signals tier-3 escalation but never acts on the host, so it must instead notice when the host has come back. Extend the existing "heartbeat returned → wedge clears" transition so it also flips `recovery_state` from `awaiting_tier_3_host_recycle` back to `ok`, emits a `tier_3_host_returned` structured event, and resets `recovery_attempts` to zero. Observation signals: node-agent re-registration against the control plane AND a fresh guest heartbeat on the affected machine. This closes the loop without any response path from the consumer back into Port.

Also assert the per-machine boundary of the signal: if two machines on the same host both escalate, each machine transitions independently — Port emits one `tier_3_escalation` per machine and later one `tier_3_host_returned` per machine. Consumer-side deduplication is not Port's concern.

## Acceptance Criteria

<!-- verify: manual, SRS-06:start:end, proof: ac-1.log-->
- [x] [SRS-06/AC-01] The decision function returns `Tier3AutoClear` when `recovery_state = AwaitingTier3HostRecycle` and heartbeats are fresh; `emit_tier_3_host_returned` writes a `Tier3HostReturned` event to the sink. A unit test exercises both layers. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- tier_3_auto_clears_when_heartbeats_return_fresh, proof: ac-1.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-01] Two machines both escalate and both return: Port emits four independent events (two `Tier3Escalation`, two `Tier3HostReturned`) — one per machine per transition. Port does not dedupe per host. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- tier_3_signal_is_per_machine_not_per_host, proof: ac-2.log -->
