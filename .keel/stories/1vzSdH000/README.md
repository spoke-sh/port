---
id: 1vzSdH000
title: Publish Multi-Node Hosted Service Workflow
type: feat
status: in-progress
created_at: 2026-03-08T21:54:19
updated_at: 2026-03-08T22:37:56
scope: 1vzSbL000/1vzSc3000
started_at: 2026-03-08T22:37:56
---

# Publish Multi-Node Hosted Service Workflow

## Summary

Publish the first multi-node hosted service workflow so operators can discover
how to target a host group through `port service` and understand the limits
that still remain after the first scheduler slice.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] CLI help, README, hosted docs, and proof publish the multi-node hosted service workflow through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzSdH000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Operator messaging makes explicit the remaining limits after this slice, including no autoscaling, broader scheduler policy, or fleet management yet. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzSdH000/verify-ac-2.sh, proof: ac-2.log -->
