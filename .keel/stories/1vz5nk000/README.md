---
id: 1vz5nk000
title: Implement Hosted Guest Operations Runtime Path
type: feat
status: done
created_at: 2026-03-07T20:31:36
updated_at: 2026-03-07T21:13:27
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-07T21:07:09
submitted_at: 2026-03-07T21:13:22
completed_at: 2026-03-07T21:13:27
---

# Implement Hosted Guest Operations Runtime Path

## Summary

Implement the first hosted runtime path for canonical
`guest exec|copy|pty|logs|forward` operations over the existing guest protocol.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-02/AC-01] Hosted guest operations reuse the canonical `guest` verbs and existing guest protocol frames while routing through control-plane authorization and node-agent guest brokerage. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nk000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Operator docs and CLI evidence explain the hosted guest runtime boundary and leave detached forwarding, Unix-socket forwarding, monitoring, secrets, services, sandboxes, and SDK work as explicit follow-on slices. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nk000/verify-ac-2.sh, proof: ac-2.log -->
