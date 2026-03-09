---
id: 1vzTSY000
title: Publish Registered Hosted Machine Workflow
type: feat
status: backlog
created_at: 2026-03-08T22:47:18
updated_at: 2026-03-08T22:50:12
scope: 1vzTQB000/1vzTR9000
---

# Publish Registered Hosted Machine Workflow

## Summary

Publish the repository-local registered-node hosted machine workflow so
operators can discover how nodes register and how hosted machine placement now
works through the canonical machine surface.

## Acceptance Criteria

<!-- verify: command, SRS-05:start, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] CLI help, README, hosted docs, and proof publish the registered-node hosted machine workflow through canonical `port machine` and `port node-agent serve` surfaces. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTSY000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-05:end, proof: ac-2.log -->
- [ ] [SRS-05/AC-02] Operator messaging makes explicit the remaining limits after this slice, including no autoscaling, broader fleet policy, or external inventory yet. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTSY000/verify-ac-2.sh, proof: ac-2.log -->
