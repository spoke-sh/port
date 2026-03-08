---
id: 1vz5o6000
title: Add Detached And Unix-Socket Forwarding
type: feat
status: done
created_at: 2026-03-07T20:31:58
updated_at: 2026-03-07T21:23:00
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-07T21:14:04
submitted_at: 2026-03-07T21:22:54
completed_at: 2026-03-07T21:23:00
---

# Add Detached And Unix-Socket Forwarding

## Summary

Extend the canonical forwarding surface with detached and Unix-socket modes once
the hosted guest runtime path exists.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-03/AC-01] `guest forward` supports detached lifecycle management and Unix-socket forwarding without introducing a second forwarding command family. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5o6000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] CLI help, docs, and evidence explain how detached and Unix-socket forwarding relate to the hosted guest runtime path and what remains downstream. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5o6000/verify-ac-2.sh, proof: ac-2.log -->
