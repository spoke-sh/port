---
id: 1vz4h3000
title: Define Hosted Machine Lifecycle Surface
type: feat
status: backlog
created_at: 2026-03-07T19:20:37
updated_at: 2026-03-07T19:22:36
scope: 1vz4Yn000/1vz4cU000
---

# Define Hosted Machine Lifecycle Surface

## Summary

Extend the shared machine lifecycle contract so hosted `machine list|status|stop`
surfaces can be represented explicitly while preserving Port's existing CLI
verbs and reporting model.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Port publishes implementation-ready hosted machine summary, status, and stop contracts that preserve the canonical `machine` verbs and make hosted ownership, routing, and status sources explicit.
- [ ] [SRS-03/AC-02] CLI help and hosted/operator docs explain the hosted lifecycle surface, including what is modeled versus what is already runnable today.
