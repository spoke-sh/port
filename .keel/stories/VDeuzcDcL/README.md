---
id: VDeuzcDcL
title: Introduce SSH Hybrid Route Contract
type: feat
status: backlog
created_at: 2026-03-12T06:21:06
updated_at: 2026-03-12T06:23:42
operator-signal: 
scope: VDcStPolu/VDeuazAgk
index: 4
---

# Introduce SSH Hybrid Route Contract

## Summary

Extend the Port host-connection and route vocabulary so SSH-managed remote
Linux hosts are first-class alongside the existing local and hosted-control-
plane lanes.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The model and config surfaces add an explicit SSH-managed host connection contract without replacing the current `local` and `hosted-control-plane` paths. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q ssh_host_connection_contract', proof: ac-1.log -->
- [ ] [SRS-NFR-02/AC-02] Existing local and hosted route semantics remain covered after the SSH lane is introduced. <!-- [SRS-NFR-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q hybrid_route_regression_local_and_hosted', proof: ac-2.log -->
