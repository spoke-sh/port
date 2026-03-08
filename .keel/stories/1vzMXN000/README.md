---
id: 1vzMXN000
title: Define Streamed Guest Session Contract
type: feat
status: backlog
created_at: 2026-03-08T15:23:49
updated_at: 2026-03-08T15:25:58
scope: 1vzMVF000/1vzMVY000
---

# Define Streamed Guest Session Contract

## Summary

Define the shared protocol, hosted attach contract, and SDK surface for
streamed PTY, log-follow, copy, and forward so the implementation stories can
land on one canonical guest-control contract.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The shared guest protocol and hosted route contract define
  streamed session lifecycle semantics for attach, payload, EOF, exit, and
  failure without introducing a second guest command family.
- [ ] [SRS-01/AC-02] The contract makes stream ownership and termination
  behavior explicit enough for CLI, runtime, node-agent, and SDK callers to
  implement deterministic cleanup.
