---
id: 1vz5nx000
title: Add Hosted Monitoring And Top
type: feat
status: done
created_at: 2026-03-07T20:31:49
updated_at: 2026-03-08T06:29:11
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-07T21:23:23
submitted_at: 2026-03-08T06:29:06
completed_at: 2026-03-08T06:29:11
---

# Add Hosted Monitoring And Top

## Summary

Add hosted monitoring and `top` surfaces once the hosted runtime and guest
brokerage path exist.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-04/AC-01] Port exposes hosted monitoring and `top` surfaces through the canonical operator model and grounds them in hosted node ownership and runtime state. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nx000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Docs and CLI evidence explain the monitoring boundary relative to runtime, forwarding, secrets, services, sandboxes, and SDK follow-on work. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nx000/verify-ac-2.sh, proof: ac-2.log -->
