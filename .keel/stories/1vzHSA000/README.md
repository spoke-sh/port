---
id: 1vzHSA000
title: Select Local Pvm Runtime Inputs
type: feat
status: done
created_at: 2026-03-08T09:58:06
updated_at: 2026-03-08T10:15:33
scope: 1vz3ck000/1vzHPo000
started_at: 2026-03-08T10:09:23
submitted_at: 2026-03-08T10:15:26
completed_at: 2026-03-08T10:15:33
---

# Select Local Pvm Runtime Inputs

## Summary

Teach the local Firecracker runtime path to select PVM-specific launch inputs
and fail with host-kit-specific diagnostics instead of treating PVM as a vague
future lane.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `port-runtime` resolves the PVM-specific Firecracker binary and launch metadata only when the requested machine selects `protection_mode = "pvm"`, while leaving the standard lane unchanged. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHSA000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Local CLI proofs surface host-kit preflight failures as explicit PVM admission errors rather than falling back to the standard Firecracker lane. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHSA000/verify-ac-2.sh, proof: ac-2.log -->
