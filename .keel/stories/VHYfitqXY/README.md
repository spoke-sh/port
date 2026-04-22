---
# system-managed
id: VHYfitqXY
status: done
created_at: 2026-04-22T14:38:20
updated_at: 2026-04-22T14:46:55
# authored
title: Harden Hosted K3s Service Recovery After Guest Relaunch
type: bug
operator-signal:
started_at: 2026-04-22T14:38:23
completed_at: 2026-04-22T14:46:55
---

# Harden Hosted K3s Service Recovery After Guest Relaunch

## Summary

Harden the hosted K3s recovery path exposed by the `cloud-aws-worker-1`
incident: Port must not leave timed-out health probe subprocesses behind, and
an active hosted service must be replayed automatically when a relaunched guest
returns with only detached runtime records and no live supervisor handle.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `refresh_machine_service_runtime` replays an active managed service when the guest reports a detached hosted runtime with no supervisor handle, so hosted K3s service recovery does not depend on transient launch-time state. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime refresh_machine_service_runtime_replays_active_service_after_detached_status -- --nocapture && cargo test -p port-runtime hosted_k3s_service_status_survives_from_persisted_records_after_launch -- --nocapture', proof: ac-1.log -->
<!-- verify: command, SRS-01:end -->
<!-- verify: command, SRS-NFR-01:start, proof: ac-2.log -->
- [x] [SRS-NFR-01/AC-02] A timed-out managed-service health command reaps its full subprocess tree, so hung `kubectl` or `crictl` descendants cannot pin hosted recovery in a false healthy state. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-guest-agent timed_out_health_command_reaps_the_full_subprocess_tree -- --nocapture && cargo test -p port-guest-agent background_supervisor_can_opt_in_to_restart_after_sustained_health_check_failure -- --nocapture', proof: ac-2.log -->
<!-- verify: command, SRS-NFR-01:end -->
