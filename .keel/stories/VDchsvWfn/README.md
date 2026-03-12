---
id: VDchsvWfn
title: Implement Canonical CLI Package Workflow
type: feat
status: done
created_at: 2026-03-11T21:16:29
updated_at: 2026-03-11T21:40:48
scope: VDcT0vaPb/VDchK6xzs
index: 1
started_at: 2026-03-11T21:31:15
completed_at: 2026-03-11T21:40:48
---

# Implement Canonical CLI Package Workflow

## Summary

Add the first canonical package build workflow for `port`, including a stable
artifact format, deterministic staging layout, and explicit failure guidance
for unsupported targets or missing prerequisites.

## Acceptance Criteria

<!-- verify: command, SRS-02:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] The repo provides a canonical package workflow that emits one versioned install artifact per supported target with explicit target and included-file reporting. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_workflow', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-2.log -->
- [x] [SRS-NFR-01/AC-02] Package names, staging layout, and included files are deterministic across repeated runs for the same supported target. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_determinism', proof: ac-2.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-3.log -->
- [x] [SRS-NFR-02/AC-03] Unsupported targets and missing packaging prerequisites fail fast with explicit guidance and no fallback to a source-only workflow. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_failure', proof: ac-3.log -->
