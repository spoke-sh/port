---
id: VDeuzX5cO
title: Add SSH Remote Doctor Guidance
type: feat
status: backlog
created_at: 2026-03-12T06:21:06
updated_at: 2026-03-12T06:23:42
operator-signal: 
scope: VDcStPolu/VDeuazAgk
index: 1
---

# Add SSH Remote Doctor Guidance

## Summary

Teach `port doctor` and adjacent CLI guidance to distinguish SSH remote-host
readiness, auth material, and bootstrap expectations from the existing local
and hosted lanes.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `port doctor` surfaces SSH remote-host prerequisites, auth material, and bootstrap requirements separately from local-host and hosted-control-plane checks. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q doctor_ssh_remote_guidance', proof: ac-1.log -->
- [ ] [SRS-NFR-01/AC-02] Misconfigured SSH targets fail with explicit route, host, provider, and ownership guidance rather than vague remote errors or local fallback. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q doctor_ssh_remote_failure_guidance', proof: ac-2.log -->
