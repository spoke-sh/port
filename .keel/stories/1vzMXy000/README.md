---
id: 1vzMXy000
title: Implement Hosted Streamed Copy Transport
type: feat
status: in-progress
created_at: 2026-03-08T15:24:26
updated_at: 2026-03-08T16:02:17
scope: 1vzMVF000/1vzMVY000
started_at: 2026-03-08T16:02:17
---

# Implement Hosted Streamed Copy Transport

## Summary

Replace the hosted guest-copy bootstrap assumption with real streamed byte
transport through the control plane and node agent.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] Hosted `port guest copy` transfers bytes through the hosted control-plane and node-agent path without assuming the source or destination host paths are directly visible on the node host. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXy000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] Hosted copy success and failure paths surface explicit route, auth, and ownership context instead of ambiguous transport errors. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXy000/verify-ac-2.sh, proof: ac-2.log -->
