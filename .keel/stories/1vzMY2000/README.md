---
id: 1vzMY2000
title: Implement Hosted Streamed Forward Transport
type: feat
status: in-progress
created_at: 2026-03-08T15:24:30
updated_at: 2026-03-08T16:19:16
scope: 1vzMVF000/1vzMVY000
started_at: 2026-03-08T16:19:16
---

# Implement Hosted Streamed Forward Transport

## Summary

Move hosted guest forwarding onto node-owned streamed transport so the hosted
path no longer depends on repo-local listener lifecycle.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] Hosted `port guest forward` uses a real hosted transport path owned by the control plane and node agent while preserving the canonical command family. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMY2000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Hosted forward does not silently fall back to the repo-local listener lifecycle once the hosted machine resolves to a streamed transport owner. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMY2000/verify-ac-2.sh, proof: ac-2.log -->
