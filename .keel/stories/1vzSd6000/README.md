---
id: 1vzSd6000
title: Define Host Group And Scheduler Contracts
type: feat
status: backlog
created_at: 2026-03-08T21:54:08
updated_at: 2026-03-08T21:54:08
scope: 1vzSbL000/1vzSc3000
---

# Define Host Group And Scheduler Contracts

## Summary

Define the shared host-group and scheduler-policy contracts so hosted Port can
target prepared groups of nodes without inventing a second service or hosted
workflow model.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Shared model, sample config, and hosted inventory/runtime structures define host groups, membership, and scheduler policy for hosted services and sandboxes. <!-- [SRS-01/AC-01] verify: cargo test, proof: ac-1.log -->
<!-- verify: command, SRS-01:end, proof: ac-2.log -->
- [ ] [SRS-01/AC-02] Validation, doctor output, or CLI-facing diagnostics surface missing or invalid host-group scheduler inputs with explicit detail. <!-- [SRS-01/AC-02] verify: cargo test, proof: ac-2.log -->
