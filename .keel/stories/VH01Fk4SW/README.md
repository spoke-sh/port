---
# system-managed
id: VH01Fk4SW
status: icebox
created_at: 2026-04-16T16:24:20
updated_at: 2026-04-16T16:24:20
# authored
title: Implement Host Reboot Client For AWS And SSH Providers
type: feat
operator-signal:
scope: VGzxMc4G4/VGzxnR97R
index: 2
---

# Implement Host Reboot Client For AWS And SSH Providers

## Summary

Introduce the `HostRebootClient` trait and land its two first implementations. `AwsEc2RebootClient` wraps the AWS SDK's `RebootInstances`, identifying the host via `host.provider_instance_id`. `SshSystemdRestartClient` reuses the existing SSH host credential path to run `systemctl restart port-node-agent`. Both return a structured `RebootOutcome` enum so the recovery runner can distinguish success from partial failures (unreachable, insufficient permissions, reboot completed but re-registration timed out). Extend `port doctor` to validate the relevant provider prerequisite for every host that would be eligible for tier-3.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] `HostRebootClient` trait exists with `AwsEc2RebootClient` and `SshSystemdRestartClient` implementations returning a structured `RebootOutcome`; unit tests cover success, unreachable, and permission-denied paths per implementation using fakes. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- host_reboot_client_aws_and_ssh, proof: ac-1.log -->
- [ ] [SRS-04/AC-02] `port doctor` validates per-host reboot prerequisites (AWS: credentials reachable + `ec2:RebootInstances` action present; SSH: existing host credential check succeeds) and reports actionable failures per host. <!-- [SRS-04/AC-02] verify: cargo test -p port-runtime -- doctor_validates_host_reboot_prerequisites, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-01] If a reboot returns `RebootOutcome::Succeeded` but the node-agent has not yet re-registered, the runner does not re-invoke reboot within a documented cooldown; a test covers the "reboot succeeded, registration pending" window. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- host_reboot_cooldown_on_pending_registration, proof: ac-3.log -->
