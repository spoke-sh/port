---
id: 1vz5nl000
title: Add Hosted Secrets Services And Sandboxes
type: feat
status: in-progress
created_at: 2026-03-07T20:31:37
updated_at: 2026-03-08T06:29:28
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-08T06:29:28
---

# Add Hosted Secrets Services And Sandboxes

## Summary

Add the first hosted secrets, services, and sandboxes surfaces on top of the
hosted runtime, forwarding, and monitoring foundation.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-05/AC-01] Port defines and implements coherent hosted secrets, services, and sandboxes surfaces that build on the canonical runtime and guest model rather than bypassing it. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nl000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Operator docs and evidence explain the supported hosted service/sandbox workflows and the remaining SDK or advanced platform work. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nl000/verify-ac-2.sh, proof: ac-2.log -->
