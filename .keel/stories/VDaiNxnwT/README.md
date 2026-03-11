---
id: VDaiNxnwT
title: Add Mission Verification Surface And Modular Just Workflows
type: feat
status: done
created_at: 2026-03-11T13:05:56
updated_at: 2026-03-11T13:23:07
scope: VDaiFfFPe/VDaiL5HDr
index: 2
started_at: 2026-03-11T13:09:32
completed_at: 2026-03-11T13:23:07
---

# Add Mission Verification Surface And Modular Just Workflows

## Summary

Add a single `just mission` entrypoint backed by the current Keel mission
surfaces, split `just` into logical modules, and make the default help output
show only the workflows maintainers actually use.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `just mission` runs the canonical verification path and ends with a compact mission report that shows mission status, child progress, next step, and a visual throughput or progress plot. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNxnwT/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] The root `just` surface is reorganized into logical modules so the default help focuses on common workflows and demo recipes are no longer listed by default. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNxnwT/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-3.log -->
- [x] [SRS-NFR-01/AC-03] The mission report derives its status from board truth and does not rely on hand-maintained summary text. <!-- [SRS-NFR-01/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNxnwT/verify-ac-3.sh, proof: ac-3.log -->
