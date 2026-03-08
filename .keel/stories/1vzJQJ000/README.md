---
id: 1vzJQJ000
title: Publish Prepared Pvm Operator Workflow
type: feat
status: done
created_at: 2026-03-08T12:04:19
updated_at: 2026-03-08T14:18:54
scope: 1vzJKE000/1vzJP2000
started_at: 2026-03-08T14:11:38
completed_at: 2026-03-08T14:18:54
---

# Publish Prepared Pvm Operator Workflow

## Summary

Publish the prepared-node PVM workflow across CLI help, README, and PVM docs
once the executable runtime path is live, including proof commands and failure
boundaries.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] CLI help, README, and `docs/pvm.md` describe the prepared-node PVM workflow, prerequisites, and failure boundaries through the canonical `port` command model. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQJ000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] Recorded CLI evidence demonstrates prepared-node PVM launch while also proving the preserved standard Firecracker lane for a new operator. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQJ000/verify-ac-2.sh, proof: ac-2.log -->
