# Tier-2 Overlay Recreate And Tier-3 Host Recycle - SRS

## Summary

Epic: VGzxMc4G4
Goal: Deliver tier-2 overlay recreate action with graceful skip for non-overlay machines, and tier-3 host recycle gated behind the single-tenant host check and a per-provider host_reboot integration (AWS EC2 reboot, SSH systemctl restart). Default off.

## Scope

### In Scope

- [SCOPE-01] Extend `[clusters.<name>.recovery]` with `tier_2_after_attempts: u32`, `tier_3_after_attempts: u32`, `require_single_tenant_for_tier_3: bool`, and `window_seconds: u64` — all used by this voyage's tier promotion logic.
- [SCOPE-03] Tier-2 guest recreate: drop `runtime/<machine>/overlay` then relaunch, when `recovery_attempts.tier_1` has hit `tier_2_after_attempts` within `window_seconds` and the machine has a configured rootfs overlay. When it has no overlay, emit `tier_2_skipped_no_overlay` and skip directly to tier-3 consideration.
- [SCOPE-05] Per-provider `host_reboot` integration exposed through a single `HostRebootClient` trait with two implementations: AWS EC2 `RebootInstances` for `provider = aws`; SSH `systemctl restart port-node-agent` for `provider = ssh`. Implementations live in `port-runtime::hosted_control_plane::host_reboot`.
- [SCOPE-04] Tier-3 host recycle: when cumulative attempts hit `tier_3_after_attempts` within `window_seconds`, invoke `HostRebootClient.reboot(host)`; then wait for the node-agent to re-register and the guest heartbeats on its placements to come back. Gated on `host.single_tenant_host == true` OR exactly one machine currently placed on the host; otherwise set `recovery_state = "suppressed_multi_tenant"`.
- [SCOPE-07] Tier-2 and tier-3 emit structured events analogous to tier-1's (including `tier_2_skipped_no_overlay` and `tier_3_suppressed_multi_tenant` outcomes).
- [SCOPE-08] Tier-3 serialization: the host-level reboot serializes across every machine on the host; while reboot is in flight, no tier-1/2 action fires on any of the host's machines.

### Out of Scope

- [SCOPE-06] Sticky `recovery_exhausted` state and `port machine unfence` reset — owned by voyage VGzxoN8WF.
- [SCOPE-09] End-to-end integration proof — owned by voyage VGzxoN8WF.
- [SCOPE-12] Providers other than `aws` and `ssh`.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `[clusters.<name>.recovery]` grows `tier_2_after_attempts`, `tier_3_after_attempts`, `require_single_tenant_for_tier_3`, and `window_seconds` with documented defaults; zero or negative values are rejected with an actionable error. | SCOPE-01 | FR-01 | unit |
| SRS-02 | When `recovery_attempts.tier_1` reaches `tier_2_after_attempts` within the `window_seconds` window and the machine has a configured rootfs overlay, the runner drops `runtime/<machine>/overlay` and relaunches; `recovery_attempts.tier_2` and `last_recovery_action` update accordingly. | SCOPE-03 | FR-01 | integration |
| SRS-03 | When a machine with no rootfs overlay reaches the tier-2 promotion condition, the runner emits a `tier_2_skipped_no_overlay` event, does not touch the filesystem, and advances promotion toward tier-3 as if tier-2 had been attempted. | SCOPE-03 | FR-01 | unit |
| SRS-04 | `HostRebootClient` is a single trait with `aws` and `ssh` implementations returning structured success/failure; a doctor check validates the relevant provider integration (credentials for `aws`, SSH reachability for `ssh`). | SCOPE-05 | FR-01 | unit |
| SRS-05 | When cumulative attempts reach `tier_3_after_attempts` within `window_seconds`, the runner invokes `HostRebootClient.reboot(host)` and waits for node-agent re-registration plus guest heartbeat recovery on the host's placements; success transitions `recovery_state` to `"ok"` on every affected machine. | SCOPE-04 | FR-01 | integration |
| SRS-06 | If `require_single_tenant_for_tier_3 = true` and `host.single_tenant_host` is not `true` AND the host has more than one placed machine, tier-3 is suppressed: the runner sets `recovery_state = "suppressed_multi_tenant"` on the wedged machine and emits a `tier_3_suppressed_multi_tenant` event. | SCOPE-04 | FR-01 | integration |
| SRS-07 | While a tier-3 reboot is in flight on a host, no tier-1 or tier-2 action fires on any of the host's other machines; the recovery runner holds a host-level lock for the duration. | SCOPE-08 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Tier-2's overlay drop must be idempotent: re-running against an already-cleared overlay returns success without error. | SCOPE-03 | NFR-01 | unit |
| SRS-NFR-02 | Host reboot must not be re-attempted within a cooldown window if the previous reboot already completed but the node-agent has not yet re-registered; tests cover the "reboot succeeded, registration pending" window. | SCOPE-04 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
