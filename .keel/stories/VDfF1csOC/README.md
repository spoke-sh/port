---
id: VDfF1csOC
title: Implement Local Attached Volume Launch Path
type: feat
status: done
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T08:17:01
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 2
started_at: 2026-03-12T07:57:39
completed_at: 2026-03-12T08:17:01
---

<!-- verify: command, SRS-02:start:end, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-3.log -->

# Implement Local Attached Volume Launch Path

## Summary

Implement the first attached-volume runtime slice by routing one declared
non-root block volume through the direct local machine lifecycle path and
projecting explicit attachment context in output and runtime state.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Direct local `machine launch`, `status`, and `stop` attach one declared non-root volume through the supported local Firecracker launcher path and keep the attachment visible in runtime state. <!-- [SRS-02/AC-01] verify: cargo test -q -p port-cli cli_machine_launch_status_and_stop_with_attached_volume, proof: ac-1.log -->
- [x] [SRS-03/AC-02] CLI success and failure surfaces keep backend, host path, machine, and ownership context explicit for machines with attached volumes. <!-- [SRS-03/AC-02] verify: cargo test -q -p port-cli cli_attached_volume_route_context, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-03] Existing attachment-free local machine workflows remain green after the attached-volume runtime path lands. <!-- [SRS-NFR-02/AC-03] verify: cargo test -q -p port-cli cli_machine_launch_status_and_stop_round_trip, proof: ac-3.log -->
