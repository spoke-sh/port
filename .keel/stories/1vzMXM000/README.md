---
id: 1vzMXM000
title: Publish Streamed Guest Workflow Surface
type: feat
status: done
created_at: 2026-03-08T15:23:48
updated_at: 2026-03-08T19:19:40
scope: 1vzMVF000/1vzMVY000
started_at: 2026-03-08T19:13:47
completed_at: 2026-03-08T19:19:40
---

# Publish Streamed Guest Workflow Surface

## Summary

Publish the streamed guest-session and hosted-transfer workflow across the CLI,
README, hosted docs, and SDK docs with recorded proof.

## Acceptance Criteria

<!-- verify: command, SRS-05:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] CLI help, README, `docs/hosted.md`, and `docs/sdk.md` describe the streamed guest-session and hosted-transfer workflow plus its explicit boundaries through the canonical Port command model. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-05:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-02] Recorded proof demonstrates the streamed PTY, log-follow, hosted copy, and hosted forward workflow through the canonical CLI and docs surfaces for a new operator. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-06:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-03] Recorded proof demonstrates that the streamed transport rollout preserves the existing Firecracker standard, hosted PVM, and AVF workflows. <!-- [SRS-06/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-3.sh, proof: ac-3.log -->
