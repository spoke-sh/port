---
# system-managed
id: VH01kQnAB
status: done
created_at: 2026-04-16T16:26:18
updated_at: 2026-04-16T18:35:21
# authored
title: Add Port Machine Unfence Command And Auto-Clear On Successful Launch
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxoN8WF
index: 2
started_at: 2026-04-16T18:33:40
submitted_at: 2026-04-16T18:35:20
completed_at: 2026-04-16T18:35:21
---

# Add Port Machine Unfence Command And Auto-Clear On Successful Launch

## Summary

Add a `port machine unfence --machine X` command that resets any non-`ok` `recovery_state` (e.g. `awaiting_tier_3_host_recycle`, `in_progress`), zeros `recovery_attempts.*`, and emits a `recovery_unfenced` event without changing any runtime state. Route it through a new `POST /v1/machines/{machine}/recovery:unfence` endpoint. Complement that with a post-launch hook: when an operator-driven `port machine launch` succeeds and a Live guest-agent heartbeat arrives within a documented convergence budget on a machine that was in `awaiting_tier_3_host_recycle`, auto-clear the state and emit `recovery_unfenced_via_launch`. Unsuccessful launches do not clear the state.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] `clear_recovery_record(record)` resets `recovery_state` to `Ok` and zeros `recovery_attempts.*`; a `RecoveryUnfenced` event is emitted to the sink. last_recovery_action stays as a breadcrumb so operators can review the transition. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- port_machine_unfence_clears_recovery_state_and_emits_event, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [x] [SRS-03/AC-01] `is_awaiting_tier_3(record)` plus `clear_recovery_record` compose into the post-launch auto-clear hook: a record in `AwaitingTier3HostRecycle` with counters >0 qualifies; clearing leaves the record in `Ok` with zero counters and the predicate returns false. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- launch_auto_clears_awaiting_tier_3_when_record_qualifies, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-01] The Cargo.toml boundary test (from VH01Fk4SW) pins `port-runtime` against any cloud-provider SDK or remote-shell dependency, so `clear_recovery_record` and the post-launch hook cannot accidentally introduce one. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- recovery_code_path_has_no_cloud_or_remote_shell_dependencies, proof: ac-3.log -->
