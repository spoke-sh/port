---
id: 1vzdMa000
title: Route Hosted Cloud Hypervisor Lifecycle
type: feat
status: in-progress
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:48
updated_at: 2026-03-09T10:06:22
started_at: 2026-03-09T10:06:22
---

# Route Hosted Cloud Hypervisor Lifecycle

## Summary

Route hosted placement, launch, status, stop, and guest attach through
Cloud Hypervisor-capable nodes without any Firecracker-specific assumptions in
the control-plane or node-agent path.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] Hosted control-plane and node-agent flows can place, launch, inspect, and stop a Cloud Hypervisor machine through the canonical machine routes. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_status_stop_route_through_live_control_plane && cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_status_and_stop_round_trip', proof: ac-1.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-04:start, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Hosted Cloud Hypervisor failures report rejected-node or runtime context instead of silently falling back to Firecracker assumptions. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback && cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback', proof: ac-2.log -->
<!-- verify: command, SRS-04:end -->
