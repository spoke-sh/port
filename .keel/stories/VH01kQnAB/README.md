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

Add a `port machine unfence --machine X` command that resets any non-`ok` `recovery_state` (e.g. `awaiting_tier_3_host_recycle`, `in_progress`), zeros `recovery_attempts.*`, and emits a `recovery_unfenced` event without changing any runtime state. Route it through a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. Complement that with a post-launch hook: when an operator-driven `port machine launch` succeeds and a Live guest-agent heartbeat arrives within a documented convergence budget on a machine that was in `awaiting_tier_3_host_recycle`, auto-clear the state and emit `recovery_unfenced_via_launch`. Unsuccessful launches do not clear the state.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-02/AC-01] `port machine unfence --machine X` clears any non-`ok` `recovery_state`, resets `recovery_attempts.tier_1/2/3` to zero, emits `recovery_unfenced`, and performs no runtime changes; on a machine already in `recovery_state = "ok"` the command is a no-op returning success. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -p port-cli -- port_machine_unfence_clears_recovery_state, proof: ac-1.log -->
<!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-03/AC-01] An operator-driven `port machine launch` on a machine in `awaiting_tier_3_host_recycle` that produces a Live guest-agent heartbeat within the documented window auto-clears the state and emits `recovery_unfenced_via_launch`; a launch that never produces a heartbeat leaves the state unchanged. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- launch_auto_clears_awaiting_tier_3_on_heartbeat, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [ ] [SRS-NFR-02/AC-01] `port machine unfence` does not call any cloud-provider API or remote shell; its network surface is the existing control-plane HTTP path only. A static boundary test pins this. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- unfence_has_no_cloud_or_remote_shell_dependencies, proof: ac-3.log -->
