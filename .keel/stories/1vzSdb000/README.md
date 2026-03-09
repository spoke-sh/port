---
id: 1vzSdb000
title: Surface Placement State Through Canonical Service Commands
type: feat
status: in-progress
created_at: 2026-03-08T21:54:39
updated_at: 2026-03-08T22:25:11
scope: 1vzSbL000/1vzSc3000
started_at: 2026-03-08T22:25:11
---

# Surface Placement State Through Canonical Service Commands

## Summary

Surface selected node, host group, and placement/runtime detail through the
existing `port service list|status|stop` workflow instead of adding a separate
hosted scheduler surface.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] Hosted `port service list`, `status`, and `stop` surface selected node identity, host-group identity, and placement/runtime state through the canonical service output. <!-- [SRS-03/AC-01] verify: cargo test, proof: ac-1.log -->
<!-- verify: command, SRS-03:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] Placement failures or stale placement records remain operator-visible through status/output instead of collapsing into generic service errors. <!-- [SRS-03/AC-02] verify: cargo test, proof: ac-2.log -->
