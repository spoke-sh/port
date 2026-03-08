---
id: 1vzMXy000
title: Implement Hosted Streamed Copy Transport
type: feat
status: backlog
created_at: 2026-03-08T15:24:26
updated_at: 2026-03-08T15:25:58
scope: 1vzMVF000/1vzMVY000
---

# Implement Hosted Streamed Copy Transport

## Summary

Replace the hosted guest-copy bootstrap assumption with real streamed byte
transport through the control plane and node agent.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Hosted `port guest copy` transfers bytes through the
  hosted control-plane and node-agent path without assuming the source or
  destination host paths are directly visible on the node host.
- [ ] [SRS-03/AC-02] Hosted copy success and failure paths surface explicit
  route, auth, and ownership context instead of ambiguous transport errors.
