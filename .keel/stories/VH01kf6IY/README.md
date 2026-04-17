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

Close the mission with three deterministic integration tests driving the full recovery ladder against simulated wedges. Tests use `tokio::time::pause` and channel-based event hooks on the runner so convergence does not depend on wall-clock `sleep`. The tier-3 test observes the emitted `tier_3_escalation` event and simulates host return via fresh heartbeats — there is no fake cloud client because Port doesn't have one.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-04/AC-01] An end-to-end test simulates a guest-side wedge on a local Firecracker machine and asserts tier-1 alone converges it without operator intervention, with the full event stream (`started`, `succeeded`, `recovery_state: "ok"`) captured from the runner event hook. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- tier_1_converges_guest_wedge, proof: ac-1.log -->
<!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-05/AC-01] An end-to-end test drives the ladder until `recovery_state = "awaiting_tier_3_host_recycle"` and a `tier_3_escalation` event is captured; then delivers simulated host return (node-agent re-registration + fresh guest heartbeat) and asserts auto-clear back to `ok` with a `tier_3_host_returned` event. The test uses no fake cloud client — observation only. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- tier_3_escalates_and_auto_clears_on_host_return, proof: ac-2.log -->
<!-- verify: manual, SRS-06:start:end -->
- [ ] [SRS-06/AC-01] An end-to-end test drives the ladder into `awaiting_tier_3_host_recycle`, crashes and restarts the control plane, asserts the state reloads, invokes `port machine unfence` via the CLI, and confirms the ladder re-arms and a subsequent wedge fires tier-1 again. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime --test recovery_ladder -- restart_preserves_escalation_then_unfence_rearms, proof: ac-3.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-01/AC-01] None of the new end-to-end tests use wall-clock `sleep` for convergence; a grep-based guard in the test file enforces absence of `thread::sleep` / `tokio::time::sleep` outside explicit `#[allow]` annotations. <!-- [SRS-NFR-01/AC-01] verify: ! rg -n 'thread::sleep|tokio::time::sleep' crates/port-runtime/tests/recovery_ladder.rs | grep -v 'allow(sleep)', proof: ac-4.log -->
