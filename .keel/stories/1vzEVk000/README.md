---
id: 1vzEVk000
title: Implement Node Agent Serve Path
type: feat
status: done
created_at: 2026-03-08T06:49:36
updated_at: 2026-03-08T07:12:54
scope: 1vzETR000/1vzETX000
started_at: 2026-03-08T07:07:03
submitted_at: 2026-03-08T07:12:51
completed_at: 2026-03-08T07:12:54
---

# Implement Node Agent Serve Path

## Summary

Implement `port node-agent serve` so one configured node owns a runtime root and
serves authenticated machine plus guest operations for the hosted control plane.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-02/AC-01] `port node-agent serve` runs an authenticated endpoint that serves machine inspection and guest operation routes by reusing Port's existing runtime-root and guest transport logic. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVk000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Node-agent failures surface machine, node, runtime-root, and guest-socket context clearly enough for operator debugging. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVk000/verify-ac-2.sh, proof: ac-2.log -->
