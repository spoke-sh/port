---
id: 1vzEVi000
title: Define Hosted HTTP Control Contracts
type: feat
status: backlog
created_at: 2026-03-08T06:49:34
updated_at: 2026-03-08T06:51:22
scope: 1vzETR000/1vzETX000
---

# Define Hosted HTTP Control Contracts

## Summary

Define the shared hosted HTTP route, auth, and payload contracts so the CLI,
SDK, control plane, and node agent all speak one live hosted transport model.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-01/AC-01] Port defines implementation-ready hosted HTTP contracts for canonical machine and guest routes, including auth headers, request bodies, and response envelopes that the CLI, SDK, control plane, and node agent can share. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-model, proof: ac-1.log -->
- [ ] [SRS-01/AC-02] The shared contracts preserve explicit node, host-group, runtime-owner, and future substrate context instead of hard-coding a one-off demo transport. <!-- [SRS-01/AC-02] verify: cargo test -q -p port-model, proof: ac-2.log -->
