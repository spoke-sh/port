# VOYAGE REPORT: Tier-2 Overlay Recreate And Tier-3 Escalation Signal

## Voyage Metadata
- **ID:** VGzxnR97R
- **Epic:** VGzxMc4G4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Wire Tier-2 Overlay Recreate With Graceful Skip
- **ID:** VH01FRXDf
- **Status:** done

#### Summary
Extend `[clusters.<name>.recovery]` with `tier_2_after_attempts` and `window_seconds`. Add the tier-1 → tier-2 promotion in the recovery runner: when `recovery_attempts.tier_1` reaches `tier_2_after_attempts` within `window_seconds` and the machine has a configured rootfs overlay, the node-agent removes `runtime/<machine>/overlay` (idempotent) and relaunches. When the machine has no overlay, emit `tier_2_skipped_no_overlay` and advance the promotion counter toward tier-3 as if tier-2 had been attempted.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `ClusterRecoveryConfig` grows `tier_2_after_attempts: u32`, `tier_3_after_attempts: u32`, and `window_seconds: u64` with documented defaults (2, 4, 1800). Zero values fail validation with an actionable error message. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_recovery, proof: ac-2.log -->
- [x] [SRS-02/AC-01] The decision function promotes to `Tier2Recreate` once `recovery_attempts.tier_1` reaches `tier_2_after_attempts`; the runner uses `drop_machine_rootfs_overlay` to remove the overlay before relaunching. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- drop_machine_rootfs_overlay_is_idempotent, proof: ac-4.log -->
- [x] [SRS-03/AC-01] `machine_has_rootfs_overlay(config, machine_name)` returns `false` when no overlay spec is set, driving the runner's `SkippedNoOverlay` path which advances promotion toward tier-3 without touching the filesystem. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- machine_has_rootfs_overlay_checks_machine_spec, proof: ac-3.log -->
- [x] [SRS-NFR-01/AC-01] `drop_machine_rootfs_overlay` is idempotent: running it against an already-cleared overlay path returns `Ok(())` without error. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- drop_machine_rootfs_overlay_is_idempotent, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01FRXDf/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH01FRXDf/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH01FRXDf/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VH01FRXDf/EVIDENCE/ac-4.log)

### Emit Tier-3 Escalation Signal With Structured Event
- **ID:** VH01Fk4SW
- **Status:** done

#### Summary
When tier-1 and tier-2 both exhaust without convergence, Port does not attempt to reboot the host. Instead, the recovery runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a structured `tier_3_escalation` event carrying the machine name, the host name, a unix timestamp, and the last failed tier outcome. That event is the handoff point: an external consumer (spoke-sh/infra, an operator on call, a systemd watcher tailing the event log) reads the signal and decides whether and how to recycle the host.

A companion boundary test scans the `port-runtime` recovery code path for cloud-provider SDK imports and remote-shell invocations and fails the build if any appear — the machine-checkable form of the "no cloud logic inside Port" rule.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `emit_tier_3_escalation` writes a structured `Tier3Escalation` event to the sink with the machine name, tier=3, and timestamp. The decision function returns `Tier3Escalate` when cumulative attempts reach the threshold; the caller mutates `recovery_state = AwaitingTier3HostRecycle` and takes no further host- or machine-level action. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- emit_tier_3_escalation_writes_structured_signal, proof: ac-1.log -->
- [x] [SRS-05/AC-01] A static boundary test scans `port-runtime`'s Cargo.toml for cloud-provider SDK dependencies (`aws-sdk-*`, `aws-config`, `rusoto`, `google-cloud-*`, `azure_*`) and remote-shell crates (`russh`, `openssh-rs`, `async-ssh2`) and fails if any appear. This pins the no-cloud-inside-Port rule as a build-time check. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- recovery_code_path_has_no_cloud_or_remote_shell_dependencies, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01Fk4SW/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH01Fk4SW/EVIDENCE/ac-2.log)

### Auto-Clear Tier-3 Escalation When Host Returns
- **ID:** VH01FzHcw
- **Status:** done

#### Summary
Port signals tier-3 escalation but never acts on the host, so it must instead notice when the host has come back. Extend the existing "heartbeat returned → wedge clears" transition so it also flips `recovery_state` from `awaiting_tier_3_host_recycle` back to `ok`, emits a `tier_3_host_returned` structured event, and resets `recovery_attempts` to zero. Observation signals: node-agent re-registration against the control plane AND a fresh guest heartbeat on the affected machine. This closes the loop without any response path from the consumer back into Port.

Also assert the per-machine boundary of the signal: if two machines on the same host both escalate, each machine transitions independently — Port emits one `tier_3_escalation` per machine and later one `tier_3_host_returned` per machine. Consumer-side deduplication is not Port's concern.

#### Acceptance Criteria
- [x] [SRS-06/AC-01] The decision function returns `Tier3AutoClear` when `recovery_state = AwaitingTier3HostRecycle` and heartbeats are fresh; `emit_tier_3_host_returned` writes a `Tier3HostReturned` event to the sink. A unit test exercises both layers. <!-- [SRS-06/AC-01] verify: cargo test -p port-runtime -- tier_3_auto_clears_when_heartbeats_return_fresh, proof: ac-1.log -->
- [x] [SRS-NFR-02/AC-01] Two machines both escalate and both return: Port emits four independent events (two `Tier3Escalation`, two `Tier3HostReturned`) — one per machine per transition. Port does not dedupe per host. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- tier_3_signal_is_per_machine_not_per_host, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH01FzHcw/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH01FzHcw/EVIDENCE/ac-2.log)


