---
id: VDfF1cZOD
title: Add Attached Volume Lane Guidance
type: feat
status: backlog
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T07:44:09
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 1
---

# Add Attached Volume Lane Guidance

## Summary

Surface attached-volume readiness, backing, and lane-support guidance so
operators can tell whether a machine can attach a declared volume before
launching it.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] `port doctor`, validation, and operator docs keep attached-volume backend, host-path, machine, and ownership detail explicit instead of collapsing the storage contract back into rootfs language. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q doctor_attached_volume_guidance && rg -n \"attached volume|host-file|host path|ownership\" README.md docs CONFIGURATION.md', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-2.log -->
- [ ] [SRS-NFR-01/AC-02] Hosted and SSH-owned machines that declare attached volumes fail fast with explicit machine, lane, and backing guidance instead of silently ignoring or rerouting the request. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q attached_volume_unsupported_lane_guidance', proof: ac-2.log -->
