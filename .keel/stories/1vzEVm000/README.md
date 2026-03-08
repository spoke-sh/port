---
id: 1vzEVm000
title: Implement Control Plane Serve Path
type: feat
status: in-progress
created_at: 2026-03-08T06:49:38
updated_at: 2026-03-08T06:56:53
scope: 1vzETR000/1vzETX000
started_at: 2026-03-08T06:56:53
---

# Implement Control Plane Serve Path

## Summary

Implement `port control-plane serve` so authenticated clients can execute
hosted machine and guest routes through configured node-agent endpoints.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-01/AC-01] `port control-plane serve` authenticates hosted API requests and serves canonical machine and guest routes by forwarding them to the resolved node agent. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVm000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Control-plane auth, routing, and unavailable-node failures surface explicit control-plane and node context instead of opaque transport errors. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVm000/verify-ac-2.sh, proof: ac-2.log -->
