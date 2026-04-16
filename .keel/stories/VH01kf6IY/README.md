---
# system-managed
id: VH01kf6IY
status: icebox
created_at: 2026-04-16T16:26:18
updated_at: 2026-04-16T16:26:18
# authored
title: Prove Recovery Ladder End-To-End With Simulated Wedges
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 3
---

# Prove Recovery Ladder End-To-End With Simulated Wedges

## Summary

Close the mission with three deterministic integration tests driving the full recovery ladder against simulated wedges. Tests use `tokio::time::pause` and channel-based event hooks on the runner so convergence does not depend on wall-clock `sleep`. Tier-3 tests use a fake `HostRebootClient` that returns success after observing the reboot call.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] An end-to-end test simulates a guest-side wedge on a local Firecracker machine and asserts tier-1 alone converges it without operator intervention, with the full event stream (`started`, `succeeded`, `recovery_state: "ok"`) captured from the runner event hook. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- tier_1_converges_guest_wedge, proof: ac-1.log -->
- [ ] [SRS-06/AC-01] An end-to-end test simulates a node-side wedge on a single-tenant host; the fake `HostRebootClient` receives the reboot call, the node-agent re-registers, guest heartbeats recover, and `recovery_state` returns to `"ok"` on the affected machine. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- tier_3_converges_node_wedge_on_single_tenant_host, proof: ac-2.log -->
- [ ] [SRS-07/AC-01] An end-to-end test drives the ladder through exhaustion (no convergence at any tier), asserts `recovery_state = "exhausted"` persists across a window rollover, then invokes `port machine unfence` and confirms the ladder re-arms and a subsequent wedge again fires tier-1. <!-- [SRS-07/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- ladder_exhausts_then_unfences_cleanly, proof: ac-3.log -->
- [ ] [SRS-NFR-02/AC-01] None of the new end-to-end tests use wall-clock `sleep` for convergence; a lint or `grep` check in the test file enforces absence of `thread::sleep` / `tokio::time::sleep` outside of explicitly annotated exceptions. <!-- [SRS-NFR-02/AC-01] verify: ! rg -n 'thread::sleep|tokio::time::sleep' crates/port-runtime/tests/recovery_ladder.rs | grep -v 'allow(sleep)', proof: ac-4.log -->
