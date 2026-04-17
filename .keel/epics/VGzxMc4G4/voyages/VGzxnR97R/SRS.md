# Tier-2 Overlay Recreate And Tier-3 Escalation Signal - SRS

## Summary

Epic: VGzxMc4G4
Goal: Deliver tier-2 overlay recreate action with graceful skip for non-overlay machines, and a structured tier-3 escalation signal that an external consumer (spoke-sh/infra, operators, systemd watchers) can act on. Port never calls cloud-provider APIs itself.

## Scope

### In Scope

- [SCOPE-01] Extend `[clusters.<name>.recovery]` with `tier_2_after_attempts: u32`, `tier_3_after_attempts: u32`, and `window_seconds: u64` — all used by this voyage's tier promotion logic.
- [SCOPE-03] Tier-2 guest recreate: drop `runtime/<machine>/overlay` then relaunch, when `recovery_attempts.tier_1` has hit `tier_2_after_attempts` within `window_seconds` and the machine has a configured rootfs overlay. When it has no overlay, emit `tier_2_skipped_no_overlay` and advance promotion toward tier-3 as if tier-2 had been attempted.
- [SCOPE-04] Tier-3 escalation signal: when cumulative attempts reach `tier_3_after_attempts` within `window_seconds`, the runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a `tier_3_escalation` structured event carrying machine, host, timestamp, and the last failed tier outcome. Port takes no further action; the event is the handoff point. A CI-level boundary check asserts the recovery code path contains no cloud-provider SDK imports or remote-shell invocations.
- [SCOPE-04] Tier-3 auto-clear: when node-agent re-registration and a fresh guest heartbeat are observed on the affected machine, the runner transitions `recovery_state` back to `ok` and emits a `tier_3_host_returned` event. No response path from the consumer back into Port is required.

### Out of Scope

- [SCOPE-06] Sticky `recovery_exhausted` state and `port machine unfence` reset — owned by voyage VGzxoN8WF.
- [SCOPE-08] End-to-end integration proof — owned by voyage VGzxoN8WF.
- [SCOPE-09] Any cloud-provider API call. AWS `RebootInstances`, GCP/Azure equivalents, and remote-shell reboot commands are out of scope for Port — they belong in the consumer of the escalation signal.
- [SCOPE-10] SSH `systemctl restart port-node-agent` as part of recovery. Remote shell actions belong to the consumer side.
- [SCOPE-14] Single-tenant-host gating inside Port. The signal is per-machine; the consumer decides whether the host-level blast radius is acceptable before acting.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `[clusters.<name>.recovery]` grows `tier_2_after_attempts`, `tier_3_after_attempts`, and `window_seconds` with documented defaults; zero values are rejected with an actionable error. | SCOPE-01 | FR-01 | unit |
| SRS-02 | When `recovery_attempts.tier_1` reaches `tier_2_after_attempts` within the `window_seconds` window and the machine has a configured rootfs overlay, the runner drops `runtime/<machine>/overlay` and relaunches; `recovery_attempts.tier_2` and `last_recovery_action` update accordingly. | SCOPE-03 | FR-01 | integration |
| SRS-03 | When a machine with no rootfs overlay reaches the tier-2 promotion condition, the runner emits a `tier_2_skipped_no_overlay` event, does not touch the filesystem, and advances promotion toward tier-3 as if tier-2 had been attempted. | SCOPE-03 | FR-01 | unit |
| SRS-04 | When cumulative attempts reach `tier_3_after_attempts` within `window_seconds`, the runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a `tier_3_escalation` event with `machine`, `host`, `timestamp_unix_s`, and `last_tier_outcome`. The runner takes no further action against the machine or host. | SCOPE-04 | FR-01 | integration |
| SRS-05 | A static boundary test scans the `port-runtime` recovery code path and asserts it contains no `aws-sdk-*` or cloud-provider HTTP call, and no remote shell invocation (`Command::new("ssh")`, `openssh`, `russh`). This pins the no-cloud-inside-Port rule as a build-time check. | SCOPE-04 | FR-01 | unit |
| SRS-06 | While `recovery_state = "awaiting_tier_3_host_recycle"`, the runner continues observing node-agent and guest heartbeats. When the node-agent re-registers AND a fresh guest heartbeat arrives, the runner transitions `recovery_state` back to `ok`, emits a `tier_3_host_returned` event, and resets `recovery_attempts` to zero. | SCOPE-04 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Tier-2's overlay drop must be idempotent: re-running against an already-cleared overlay returns success without error. | SCOPE-03 | NFR-01 | unit |
| SRS-NFR-02 | The tier-3 escalation signal is per-machine, not per-host. If two machines on the same host both escalate, each machine's `recovery_state` flips independently and the structured events are emitted once per machine — the consumer decides whether the underlying host action should be deduplicated. | SCOPE-04 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
