---
id: 1vzTTI000
title: Implement Node Agent Registration Refresh
type: feat
status: in-progress
created_at: 2026-03-08T22:48:04
updated_at: 2026-03-08T23:03:11
scope: 1vzTQB000/1vzTR9000
started_at: 2026-03-08T23:03:11
---

# Implement Node Agent Registration Refresh

## Summary

Let `port node-agent serve` register one configured node against the hosted
control plane and refresh that registration while the node agent remains live.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] A running node agent registers its node against the hosted control plane and refreshes freshness state while it is serving. <!-- [SRS-02/AC-01] verify: cargo test -q -p port-runtime node_agent_registers_and_refreshes_against_control_plane, proof: ac-2.log -->
<!-- verify: command, SRS-02:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Registration failures such as unreachable control planes, auth mismatches, or stale registration are surfaced explicitly in hosted runtime proof output. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-runtime node_agent_surfaces_explicit_registration_failures, proof: ac-2.log -->
