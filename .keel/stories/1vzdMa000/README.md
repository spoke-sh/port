---
id: 1vzdMa000
title: Route Hosted Cloud Hypervisor Lifecycle
type: feat
status: backlog
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:48
updated_at: 2026-03-09T09:25:33
---

# Route Hosted Cloud Hypervisor Lifecycle

## Summary

Route hosted placement, launch, status, stop, and guest attach through
Cloud Hypervisor-capable nodes without any Firecracker-specific assumptions in
the control-plane or node-agent path.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] Hosted control-plane and node-agent flows can place, launch, inspect, and stop a Cloud Hypervisor machine through the canonical machine routes. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli --test machine_commands', proof: ac-1.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-04:start, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Hosted Cloud Hypervisor failures report rejected-node or runtime context instead of silently falling back to Firecracker assumptions. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli -p port-sdk', proof: ac-2.log -->
<!-- verify: command, SRS-04:end -->
