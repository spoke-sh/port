---
id: 1vz5nU000
title: Implement Hosted Control Plane Runtime Path
type: feat
status: backlog
created_at: 2026-03-07T20:31:20
updated_at: 2026-03-07T20:35:30
scope: 1vz4Yn000/1vz5mg000
---

# Implement Hosted Control Plane Runtime Path

## Summary

Implement the first authenticated hosted runtime path for canonical `machine
list|status|stop` operations through the control plane and node agent.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Hosted `machine list|status|stop` operations work through the canonical CLI and route through the modeled hosted control-plane and node-agent ownership path.
- [ ] [SRS-01/AC-02] Help text, docs, and CLI evidence distinguish hosted runtime behavior from still-planned forwarding, monitoring, secrets, services, sandboxes, and SDK work.
