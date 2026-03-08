---
id: 1vzLZm000
title: Wire Avf Guest Transport And Console Capture
type: feat
status: in-progress
created_at: 2026-03-08T14:22:14
updated_at: 2026-03-08T14:52:50
scope: 1vzJKE000/1vzLYD000
started_at: 2026-03-08T14:52:50
---

# Wire Avf Guest Transport And Console Capture

## Summary

Map the shared guest protocol onto the AVF transport and serial-console
surfaces so the canonical `guest` verbs and machine log inspection work for
AVF-backed machines.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] AVF-targeted machines expose `guest exec|copy|pty|logs|forward` through the canonical CLI and shared guest protocol via an AVF transport adapter. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZm000/verify-ac-1.sh, proof: ac-2.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] AVF boot and console output land in canonical runtime log surfaces that `machine status` and operator inspection can reference. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZm000/verify-ac-2.sh, proof: ac-2.log -->
