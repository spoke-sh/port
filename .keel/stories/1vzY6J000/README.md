---
id: 1vzY6J000
title: Publish Pvm Host Kit Operator Workflow
type: feat
status: in-progress
created_at: 2026-03-09T03:44:39
updated_at: 2026-03-09T09:06:08
scope: 1vz3ck000/1vzY3z000
started_at: 2026-03-09T09:06:08
---

# Publish Pvm Host Kit Operator Workflow

## Summary

Publish the canonical operator workflow for PVM host-kit packaging, artifact
mobility, hosted node preparation, and proof so the lane is discoverable
through README, PVM docs, hosted docs, and CLI help.

## Acceptance Criteria

<!-- verify: command, SRS-04:start -->
- [ ] [SRS-04/AC-01] README, `docs/pvm.md`, `docs/hosted.md`, and CLI help publish the canonical `x86_64` Firecracker/PVM host-kit and artifact workflow, including the explicit `aarch64` research-only boundary. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzY6J000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-04:start -->
- [ ] [SRS-04/AC-02] The published workflow includes repo-local proof commands for artifact build/validate/push/pull plus hosted node preparation/import so operators can verify the lane without hidden steps. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzY6J000/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-04:end -->
