---
id: 1vzUq6000
title: Persist Hosted Registration And Freshness
type: feat
status: backlog
created_at: 2026-03-09T00:15:42
updated_at: 2026-03-09T00:20:39
scope: 1vzUnI000/1vzUoK000
---

# Persist Hosted Registration And Freshness

## Summary

Persist hosted node registration and heartbeat freshness under the control-plane
runtime root and refresh that state through the existing hosted node-agent
transport so the fleet view survives restart and stale nodes become explicitly
ineligible.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] The hosted control plane stores and reloads durable node registration records so restart reconstructs the fleet view from runtime-owned state instead of losing node presence. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-1.log -->
<!-- verify: command, SRS-02:start, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] `port node-agent serve` refreshes registration and heartbeat freshness through the existing hosted auth and transport contract without a second token or registration path. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-2.log -->
<!-- verify: command, SRS-01:end, proof: ac-3.log -->
- [ ] [SRS-01/AC-03] Restart recovery and freshness expiry behave deterministically for the same stored registry state and current time inputs, satisfying `SRS-NFR-01`. <!-- [SRS-01/AC-03] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-3.log -->
<!-- verify: command, SRS-02:end, proof: ac-4.log -->
- [ ] [SRS-02/AC-04] Stale-node or durable-registry failures include explicit control-plane path context and affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-02/AC-04] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-4.log -->
