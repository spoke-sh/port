---
id: VDfF1cZOD
title: Add Attached Volume Lane Guidance
type: feat
status: done
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T08:28:31
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 1
started_at: 2026-03-12T08:18:20
completed_at: 2026-03-12T08:28:31
---

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-2.log -->

# Add Attached Volume Lane Guidance

## Summary

Surface attached-volume readiness, backing, and lane-support guidance so
operators can tell whether a machine can attach a declared volume before
launching it.

## Acceptance Criteria

- [x] [SRS-04/AC-01] `port doctor`, validation, and operator docs keep attached-volume backend, host-path, machine, and ownership detail explicit instead of collapsing the storage contract back into rootfs language. <!-- [SRS-04/AC-01] verify: sh -c 'cd /home/alex/workspace/spoke-sh/port && cargo test -q doctor_attached_volume_guidance && /home/alex/.nix-profile/bin/rg -n "attached volume|host-file|host path|ownership" README.md docs CONFIGURATION.md', proof: ac-1.log -->
- [x] [SRS-NFR-01/AC-02] Hosted and SSH-owned machines that declare attached volumes fail fast with explicit machine, lane, and backing guidance instead of silently ignoring or rerouting the request. <!-- [SRS-NFR-01/AC-02] verify: cargo test -q attached_volume_unsupported_lane_guidance, proof: ac-2.log -->
