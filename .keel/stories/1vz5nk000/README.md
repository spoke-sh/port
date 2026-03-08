---
id: 1vz5nk000
title: Implement Hosted Guest Operations Runtime Path
type: feat
status: backlog
created_at: 2026-03-07T20:31:36
updated_at: 2026-03-07T20:35:30
scope: 1vz4Yn000/1vz5mg000
---

# Implement Hosted Guest Operations Runtime Path

## Summary

Implement the first hosted runtime path for canonical
`guest exec|copy|pty|logs|forward` operations over the existing guest protocol.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Hosted guest operations reuse the canonical `guest` verbs and existing guest protocol frames while routing through control-plane authorization and node-agent guest brokerage.
- [ ] [SRS-02/AC-02] Operator docs and CLI evidence explain the hosted guest runtime boundary and leave detached forwarding, Unix-socket forwarding, monitoring, secrets, services, sandboxes, and SDK work as explicit follow-on slices.
