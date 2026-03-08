---
id: 1vz3kq000
title: Extract Firecracker Driver Boundary
type: feat
status: done
created_at: 2026-03-07T18:20:28
updated_at: 2026-03-07T18:26:31
scope: 1vz3ck000/1vz3j0000
started_at: 2026-03-07T18:22:52
submitted_at: 2026-03-07T18:26:25
completed_at: 2026-03-07T18:26:31
---

# Extract Firecracker Driver Boundary

## Summary

Define and scaffold the first substrate-driver boundary so local Firecracker
runtime ownership becomes one implementation behind shared lifecycle and guest
attach interfaces.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `port-runtime` defines implementation-ready driver seams for launch, inventory/status, stop, and guest attach without hiding Firecracker-specific behavior behind ad hoc branching. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-runtime && rg -n "trait MachineDriver|struct FirecrackerLocalDriver|fn driver_for_machine|fn firecracker_local_launch_machine|fn firecracker_local_list_machines|fn firecracker_local_stop_machine|fn resolve_firecracker_guest_endpoint" crates/port-runtime/src/lib.rs', proof: ac-1.log-->
