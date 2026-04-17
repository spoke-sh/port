---
# system-managed
id: VH01kEV1x
status: icebox
created_at: 2026-04-16T16:26:17
updated_at: 2026-04-16T16:26:17
# authored
title: Persist Recovery State Across Control-Plane Restarts
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 1
---

# Persist Recovery State Across Control-Plane Restarts

## Summary

Make `recovery_state` and `recovery_attempts` durable so a control-plane restart mid-escalation does not silently re-arm the ladder against a machine that is already in `awaiting_tier_3_host_recycle`. Write each machine's record to `runtime/recovery/<machine>.json` alongside the existing registered-node state; load on startup into the in-memory recovery map. Once the machine is in `awaiting_tier_3_host_recycle`, the runner does not attempt further tier actions — it only observes heartbeats for auto-clear.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end -->
- [ ] [SRS-01/AC-01] An integration test seeds `recovery_state = "awaiting_tier_3_host_recycle"` with non-zero `recovery_attempts`, restarts the control plane (fresh process), and asserts the state and counters reload from disk unchanged. The ladder takes no tier-1/2 action against the machine after reload. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime -- recovery_state_persists_across_restart, proof: ac-1.log -->
