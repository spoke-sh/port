---
id: 1vzLZY000
title: Publish Macos Avf Operator Workflow
type: feat
status: done
created_at: 2026-03-08T14:22:00
updated_at: 2026-03-08T15:20:34
scope: 1vzJKE000/1vzLYD000
started_at: 2026-03-08T15:09:32
completed_at: 2026-03-08T15:20:34
---

# Publish Macos Avf Operator Workflow

## Summary

Publish the native macOS AVF workflow across the CLI help and docs once the
runtime slices are in place, including proof commands and explicit unsupported
boundaries.

## Acceptance Criteria

<!-- verify: command, SRS-05:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] CLI help, README, `docs/avf.md`, and macOS operator docs describe the native AVF workflow, prerequisites, and unsupported boundaries through the canonical `port` command model. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-05:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-02] Recorded proof demonstrates the AVF workflow contract through the canonical CLI and docs surfaces for a new operator. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-06:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-01] Recorded proof demonstrates the AVF workflow contract while also preserving explicit Linux-lane and unsupported-host boundaries for operators. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-3.sh, proof: ac-3.log -->
