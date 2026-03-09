---
id: 1vzTTJ000
title: Route Hosted Machine Launch Through Registered Nodes
type: feat
status: backlog
created_at: 2026-03-08T22:48:05
updated_at: 2026-03-08T22:50:12
scope: 1vzTQB000/1vzTR9000
---

# Route Hosted Machine Launch Through Registered Nodes

## Summary

Route canonical hosted `port machine launch` through registered nodes so the
control plane chooses one eligible live node and records the placement it used.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] Hosted `port machine launch` selects one eligible registered node and executes the existing node-owned launch path through that node. <!-- [SRS-03/AC-01] verify: cargo test, proof: ac-1.log -->
<!-- verify: command, SRS-03:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] Placement remains deterministic for the same registered-node input and rejects stale or ineligible nodes with explicit detail. <!-- [SRS-03/AC-02] verify: cargo test, proof: ac-2.log -->
