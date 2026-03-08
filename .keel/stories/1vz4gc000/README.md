---
id: 1vz4gc000
title: Define Hosted Guest Bridge Attach Contract
type: feat
status: backlog
created_at: 2026-03-07T19:20:10
updated_at: 2026-03-07T19:22:36
scope: 1vz4Yn000/1vz4cU000
---

# Define Hosted Guest Bridge Attach Contract

## Summary

Define the first hosted guest bridge attach contract so later hosted `exec`,
`copy`, `pty`, `logs`, and `forward` operations can reuse the current guest
protocol through control-plane and node-agent brokerage.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] Port publishes an implementation-ready hosted guest bridge attach contract that preserves the current guest protocol and names the control-plane and node-agent brokerage path explicitly.
- [ ] [SRS-04/AC-02] README, hosted docs, and CLI help explain how hosted guest operations map onto the same canonical `guest` verbs and what follow-on implementation still remains.
