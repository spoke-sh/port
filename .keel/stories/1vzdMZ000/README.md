---
id: 1vzdMZ000
title: Implement Local Cloud Hypervisor Machine Driver
type: feat
status: done
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:47
updated_at: 2026-03-09T09:53:46
started_at: 2026-03-09T09:36:35
completed_at: 2026-03-09T09:53:46
---

# Implement Local Cloud Hypervisor Machine Driver

## Summary

Implement the local Cloud Hypervisor launch, status, and stop path through
Port's machine-driver seam, including runtime manifest ownership and console
capture.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `port machine launch|status|stop` executes a Cloud Hypervisor machine locally through the canonical driver boundary and records coherent runtime metadata. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime cloud_hypervisor_launch_status_and_stop_write_canonical_runtime_state && cargo test -q -p port-cli --test machine_commands cli_machine_launch_status_and_stop_route_cloud_hypervisor_locally', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Local Cloud Hypervisor preflight failures identify the missing host prerequisite or runtime boundary instead of generic unsupported-host output. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime cloud_hypervisor_launch_surfaces_missing_binary_preflight && cargo test -q -p port-cli --test machine_commands cli_machine_launch_surfaces_missing_cloud_hypervisor_binary', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->
