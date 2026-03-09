---
id: 1vzSdV000
title: Implement Hosted Service Placement Scheduler
type: feat
status: in-progress
created_at: 2026-03-08T21:54:33
updated_at: 2026-03-08T22:07:52
scope: 1vzSbL000/1vzSc3000
started_at: 2026-03-08T22:07:52
---

# Implement Hosted Service Placement Scheduler

## Summary

Implement the first deterministic hosted scheduler slice so `port service
apply` can choose an eligible node from a target host group and record that
placement decision.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [x] [SRS-02/AC-01] Hosted `port service apply --kind service` and `--kind sandbox` select one eligible prepared node from the requested host group and route execution through that node's existing hosted runtime path. <!-- [SRS-02/AC-01] verify: cargo test, proof: ac-1.log -->
<!-- verify: command, SRS-02:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Scheduler selection is deterministic for equal inventory input and returns explicit admission detail when no node qualifies. <!-- [SRS-02/AC-02] verify: cargo test, proof: ac-2.log -->
