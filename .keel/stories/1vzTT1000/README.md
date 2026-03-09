---
id: 1vzTT1000
title: Define Registered Node Contract And State
type: feat
status: in-progress
created_at: 2026-03-08T22:47:47
updated_at: 2026-03-08T22:50:56
scope: 1vzTQB000/1vzTR9000
started_at: 2026-03-08T22:50:56
---

# Define Registered Node Contract And State

## Summary

Define the shared registered-node contract and control-plane-owned registration
state so hosted machine placement can reason about live nodes without transient
`--node-binding` startup flags.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Shared model, hosted protocol, and runtime state define registered-node identity, endpoint, token, freshness, and placement-facing fields. <!-- [SRS-01/AC-01] verify: cargo test, proof: ac-1.log -->
<!-- verify: command, SRS-01:end, proof: ac-2.log -->
- [ ] [SRS-01/AC-02] Validation or diagnostics surface missing or invalid registered-node inputs with explicit detail. <!-- [SRS-01/AC-02] verify: cargo test, proof: ac-2.log -->
