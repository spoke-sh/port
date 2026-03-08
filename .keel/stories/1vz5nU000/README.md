---
id: 1vz5nU000
title: Implement Hosted Control Plane Runtime Path
type: feat
status: done
created_at: 2026-03-07T20:31:20
updated_at: 2026-03-07T21:05:35
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-07T20:37:23
submitted_at: 2026-03-07T21:05:29
completed_at: 2026-03-07T21:05:35
---

# Implement Hosted Control Plane Runtime Path

## Summary

Implement the first authenticated hosted runtime path for canonical `machine
list|status|stop` operations through the control plane and node agent.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-01/AC-01] Hosted `machine list|status|stop` operations work through the canonical CLI and route through the modeled hosted control-plane and node-agent ownership path. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nU000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Help text, docs, and CLI evidence distinguish hosted runtime behavior from still-planned forwarding, monitoring, secrets, services, sandboxes, and SDK work. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nU000/verify-ac-2.sh, proof: ac-2.log -->
