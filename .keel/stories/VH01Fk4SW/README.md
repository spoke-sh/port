---
# system-managed
id: VH01Fk4SW
status: icebox
created_at: 2026-04-16T16:24:20
updated_at: 2026-04-16T16:24:20
# authored
title: Emit Tier-3 Escalation Signal With Structured Event
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 2
---

# Emit Tier-3 Escalation Signal With Structured Event

## Summary

When tier-1 and tier-2 both exhaust without convergence, Port does not attempt to reboot the host. Instead, the recovery runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a structured `tier_3_escalation` event carrying the machine name, the host name, a unix timestamp, and the last failed tier outcome. That event is the handoff point: an external consumer (spoke-sh/infra, an operator on call, a systemd watcher tailing the event log) reads the signal and decides whether and how to recycle the host.

A companion boundary test scans the `port-runtime` recovery code path for cloud-provider SDK imports and remote-shell invocations and fails the build if any appear — the machine-checkable form of the "no cloud logic inside Port" rule.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-04/AC-01] When cumulative attempts reach `tier_3_after_attempts` within `window_seconds`, the recovery runner sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine, emits a `tier_3_escalation` event with `machine`, `host`, `timestamp_unix_s`, and `last_tier_outcome`, and takes no further host- or machine-level action. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- tier_3_escalation_emits_signal_and_stops_acting, proof: ac-1.log -->
<!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-05/AC-01] A static boundary test scans the recovery code path in `port-runtime` and asserts that no `aws-sdk-*` or cloud-provider HTTP call appears, and no remote shell invocation (`Command::new("ssh")`, `openssh`, `russh`) is introduced. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime -- recovery_code_path_has_no_cloud_or_ssh_dependencies, proof: ac-2.log -->
