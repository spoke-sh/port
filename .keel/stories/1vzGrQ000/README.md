---
id: 1vzGrQ000
title: Publish PVM Operator Proof Workflow
type: feat
status: in-progress
created_at: 2026-03-08T09:20:08
updated_at: 2026-03-08T09:47:05
scope: 1vz3ck000/1vzGo0000
started_at: 2026-03-08T09:47:05
---

# Publish PVM Operator Proof Workflow

## Summary

Publish the operator-facing PVM workflow in README/docs/help text and back it
with repository-local proof scripts that show the x86_64 keep decision, arm64
research-only boundary, and current host-kit/artifact validation flow.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-04/AC-01] README, `docs/pvm.md`, and CLI help explain the x86_64 PVM keep decision, the required host-kit and artifact-kit prerequisites, and the `aarch64` research-only boundary. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrQ000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: manual, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] Repository-local proof scripts and recorded evidence demonstrate the documented foundation workflow without regressing the existing standard Firecracker operator path. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrQ000/verify-ac-2.sh, proof: ac-2.log -->
