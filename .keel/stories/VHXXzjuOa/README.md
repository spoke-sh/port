---
# system-managed
id: VHXXzjuOa
status: backlog
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T10:05:09
# authored
title: Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background
type: feat
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 2
---

# Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background

## Summary

Move hosted placement repair out of synchronous read handlers and into explicit
reconcile hooks so placement persistence becomes deterministic, canonical, and
separate from operator truth surfaces.

## Acceptance Criteria

<!-- verify: unit, SRS-03:start -->
- [ ] [SRS-03/AC-01] Hosted request handlers stop persisting placement state on read paths; placement repair is triggered only by startup, registration, or lifecycle hooks. <!-- [SRS-03/AC-01] verify: targeted handler and reconcile tests -->
<!-- verify: unit, SRS-03:end -->
<!-- verify: unit, SRS-04:start -->
- [ ] [SRS-04/AC-02] The placement reconciler canonicalizes legacy node aliases to configured node identities and persists repaired machine placement without requiring a user read path. <!-- [SRS-04/AC-02] verify: alias-repair regression tests -->
<!-- verify: unit, SRS-04:end -->
<!-- verify: unit, SRS-NFR-02:start -->
- [ ] [SRS-NFR-02/AC-03] No hosted request handler reintroduces control-plane self-recursion or synchronous write-on-read repair while reconciling placement. <!-- [SRS-NFR-02/AC-03] verify: targeted recursion and write-on-read regression tests -->
<!-- verify: unit, SRS-NFR-02:end -->
