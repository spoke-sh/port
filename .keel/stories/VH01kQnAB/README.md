---
# system-managed
id: VH01kQnAB
status: icebox
created_at: 2026-04-16T16:26:18
updated_at: 2026-04-16T16:26:18
# authored
title: Add Port Machine Unfence Command And Auto-Clear On Successful Launch
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 2
---

# Add Port Machine Unfence Command And Auto-Clear On Successful Launch

## Summary

Add a `port machine unfence --machine X` command that clears the sticky `recovery_exhausted` flag, resets attempt counters, and emits a `recovery_unfenced` event without changing any runtime state. Route it through a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. Complement that with a post-launch hook: when an operator-driven `port machine launch` succeeds and a Live guest-agent heartbeat arrives within a documented convergence budget, auto-clear `recovery_exhausted` and emit `recovery_unfenced_via_launch`. Unsuccessful launches do not clear the state.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port machine unfence --machine X` clears `recovery_exhausted`, resets `recovery_attempts.tier_1/2/3` to zero, emits `recovery_unfenced`, and produces no runtime side effects; the command fails with an actionable error on a machine that is not exhausted. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -p port-cli -- port_machine_unfence_clears_exhausted, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] An operator-driven `port machine launch` on an exhausted machine that produces a Live guest-agent heartbeat within the documented window auto-clears `recovery_exhausted` and emits `recovery_unfenced_via_launch`; a launch that never produces a heartbeat leaves the state unchanged. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- launch_auto_clears_recovery_exhausted_on_heartbeat, proof: ac-2.log -->
