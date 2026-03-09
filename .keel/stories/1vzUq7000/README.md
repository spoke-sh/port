---
id: 1vzUq7000
title: Materialize Imported Fleet Inventory
type: feat
status: in-progress
created_at: 2026-03-09T00:15:43
updated_at: 2026-03-09T00:41:01
scope: 1vzUnI000/1vzUoK000
started_at: 2026-03-09T00:41:01
---

# Materialize Imported Fleet Inventory

## Summary

Materialize an imported fleet inventory contract into the hosted control-plane
state so Port can merge externally supplied node membership and provenance with
configured nodes before routing or inspection occurs.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [x] [SRS-03/AC-01] Port accepts and persists an imported inventory contract that records node membership, provider, provenance, and capability summary under the hosted control-plane state. <!-- [SRS-03/AC-01] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-2.log -->
<!-- verify: command, SRS-03:end, proof: ac-3.log -->
- [x] [SRS-03/AC-02] Imported inventory merges onto canonical configured node identities and reports unknown-node or conflicting imports explicitly instead of silently inventing new runtime-only nodes. <!-- [SRS-03/AC-02] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-2.log -->
<!-- verify: command, SRS-03:start, proof: ac-3.log -->
- [x] [SRS-03/AC-03] Import mismatch and persistence failures include durable import path context and affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-03/AC-03] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-3.log -->
<!-- verify: command, SRS-03:end, proof: ac-3.log -->
