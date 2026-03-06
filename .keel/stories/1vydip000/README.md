---
id: 1vydip000
title: Deliver Guest Agent Capabilities
type: feat
status: icebox
created_at: 2026-03-06T14:32:39
updated_at: 2026-03-06T14:32:50
scope: 1vydg7000/1vydgL000
---

# Deliver Guest Agent Capabilities

## Summary

Implement the guest agent transport and expose `exec`, `copy`, `pty`, `logs`,
and `forward` through the Port CLI and shared protocol.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] The guest agent protocol supports request/response flows for `exec`, `copy`, `pty`, `logs`, and `forward`.
- [ ] [SRS-04/AC-02] The canonical CLI exposes `port guest exec`, `port guest copy`, `port guest pty`, `port guest logs`, and `port guest forward`.
- [ ] [SRS-04/AC-03] Automated tests cover protocol framing and at least one happy-path behavior for each guest capability.
