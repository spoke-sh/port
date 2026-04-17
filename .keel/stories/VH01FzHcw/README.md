---
# system-managed
id: VH01FzHcw
status: icebox
created_at: 2026-04-16T16:24:21
updated_at: 2026-04-16T16:24:21
# authored
title: Auto-Clear Tier-3 Escalation When Host Returns
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 3
---

# Auto-Clear Tier-3 Escalation When Host Returns

## Summary

Port signals tier-3 escalation but never acts on the host, so it must instead notice when the host has come back. Extend the existing "heartbeat returned → wedge clears" transition so it also flips `recovery_state` from `awaiting_tier_3_host_recycle` back to `ok`, emits a `tier_3_host_returned` structured event, and resets `recovery_attempts` to zero. Observation signals: node-agent re-registration against the control plane AND a fresh guest heartbeat on the affected machine. This closes the loop without any response path from the consumer back into Port.

Also assert the per-machine boundary of the signal: if two machines on the same host both escalate, each machine transitions independently — Port emits one `tier_3_escalation` per machine and later one `tier_3_host_returned` per machine. Consumer-side deduplication is not Port's concern.

## Acceptance Criteria

<!-- verify: manual, SRS-06:start:end -->
- [ ] [SRS-06/AC-01] With a machine in `recovery_state = "awaiting_tier_3_host_recycle"`, a simulated host return (node-agent re-register + fresh guest heartbeat) transitions `recovery_state` back to `ok`, emits a `tier_3_host_returned` event, and resets `recovery_attempts` to zero. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- tier_3_auto_clears_when_host_returns, proof: ac-1.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [ ] [SRS-NFR-02/AC-01] Two machines on the same host both escalate: Port sets `recovery_state = "awaiting_tier_3_host_recycle"` on each independently, emits one `tier_3_escalation` per machine, and when the host returns emits one `tier_3_host_returned` per machine. Port does not dedupe per host; that is the consumer's concern. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- tier_3_signal_is_per_machine_not_per_host, proof: ac-2.log -->
