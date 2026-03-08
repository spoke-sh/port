---
id: 1vz2oh000
title: Publish Hosted Node Agent Contract
type: feat
status: backlog
created_at: 2026-03-07T17:20:23
updated_at: 2026-03-07T17:24:27
scope: 1vz2eV000/1vz2ky000
---

# Publish Hosted Node Agent Contract

## Summary

Define the first canonical hosted-Port contract: a node-local agent plus central
control plane that preserve today's guest-operation model while adding remote
lifecycle ownership, transport brokering, and inventory.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Port publishes a canonical hosted-control document describing node-agent responsibilities, control-plane responsibilities, and machine lifecycle ownership for local versus hosted execution.
- [ ] [SRS-03/AC-02] The contract explains how guest `exec`, `copy`, `pty`, `logs`, and `forward` are brokered through the hosted product without replacing the current guest protocol semantics.
- [ ] [SRS-06/AC-03] README and linked docs surface the hosted contract and the current support matrix so operators can distinguish shipped local behavior from planned hosted behavior.
