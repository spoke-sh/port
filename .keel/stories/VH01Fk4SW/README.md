---
# system-managed
id: VH01Fk4SW
status: done
created_at: 2026-04-16T16:24:20
updated_at: 2026-04-16T18:29:44
# authored
title: Emit Tier-3 Escalation Signal With Structured Event
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 2
started_at: 2026-04-16T18:26:22
submitted_at: 2026-04-16T18:29:44
completed_at: 2026-04-16T18:29:44
---

# Emit Tier-3 Escalation Signal With Structured Event

## Summary

When tier-1 and tier-2 both exhaust without convergence, Port does not attempt to reboot the host. Instead, the recovery runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a structured `tier_3_escalation` event carrying the machine name, the host name, a unix timestamp, and the last failed tier outcome. That event is the handoff point: an external consumer (spoke-sh/infra, an operator on call, a systemd watcher tailing the event log) reads the signal and decides whether and how to recycle the host.

A companion boundary test scans the `port-runtime` recovery code path for cloud-provider SDK imports and remote-shell invocations and fails the build if any appear — the machine-checkable form of the "no cloud logic inside Port" rule.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] `emit_tier_3_escalation` writes a structured `Tier3Escalation` event to the sink with the machine name, tier=3, and timestamp. The decision function returns `Tier3Escalate` when cumulative attempts reach the threshold; the caller mutates `recovery_state = AwaitingTier3HostRecycle` and takes no further host- or machine-level action. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- emit_tier_3_escalation_writes_structured_signal, proof: ac-1.log -->
<!-- verify: manual, SRS-05:start:end -->
- [x] [SRS-05/AC-01] A static boundary test scans `port-runtime`'s Cargo.toml for cloud-provider SDK dependencies (`aws-sdk-*`, `aws-config`, `rusoto`, `google-cloud-*`, `azure_*`) and remote-shell crates (`russh`, `openssh-rs`, `async-ssh2`) and fails if any appear. This pins the no-cloud-inside-Port rule as a build-time check. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- recovery_code_path_has_no_cloud_or_remote_shell_dependencies, proof: ac-2.log -->
