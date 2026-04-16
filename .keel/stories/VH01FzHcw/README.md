---
# system-managed
id: VH01FzHcw
status: icebox
created_at: 2026-04-16T16:24:21
updated_at: 2026-04-16T16:24:21
# authored
title: Fire Tier-3 Host Recycle Behind Single-Tenant Gate
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 3
---

# Fire Tier-3 Host Recycle Behind Single-Tenant Gate

## Summary

Wire tier-3 into the recovery runner. Promote to tier-3 when cumulative attempts reach `tier_3_after_attempts` within `window_seconds`. Before invoking `HostRebootClient.reboot(host)`, check the single-tenant gate: `host.single_tenant_host == true` OR the host has exactly one placed machine. When the gate fails and `require_single_tenant_for_tier_3 = true`, set `recovery_state = "suppressed_multi_tenant"` on the wedged machine and emit `tier_3_suppressed_multi_tenant`. When the gate passes, hold the host-level reboot lock for the duration of reboot + re-registration wait so tier-1/2 runners for other machines on the same host are blocked.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] An integration test seeds cumulative attempts to `tier_3_after_attempts`; the runner invokes `HostRebootClient.reboot`, waits for node-agent re-registration and guest-heartbeat recovery, and returns every affected machine's `recovery_state` to `"ok"`. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- tier_3_host_recycle_converges, proof: ac-1.log -->
- [ ] [SRS-06/AC-01] With `require_single_tenant_for_tier_3 = true` and a host hosting multiple machines, the runner suppresses tier-3: `recovery_state = "suppressed_multi_tenant"`, a `tier_3_suppressed_multi_tenant` event is emitted, and no reboot is attempted. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- tier_3_suppressed_on_multi_tenant_host, proof: ac-2.log -->
- [ ] [SRS-07/AC-01] A host with multiple machines hitting wedges concurrently runs at most one tier-3 reboot at a time; while the reboot lock is held, tier-1/2 actions on the host's other machines return `skipped_busy`. <!-- [SRS-07/AC-01] verify: cargo test -p port-runtime -- tier_3_host_lock_blocks_tier_1_2_on_other_machines, proof: ac-3.log -->
