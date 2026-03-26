---
# system-managed
id: VEyjdJhne
status: backlog
created_at: 2026-03-26T06:10:18
updated_at: 2026-03-26T06:14:07
# authored
title: Publish External Project Deployment Contract And Boundaries
type: feat
operator-signal:
scope: VEyjUL2Zr/VEyjdNXnp
index: 1
---

# Publish External Project Deployment Contract And Boundaries

## Summary

Publish the canonical external-project deployment workflow, its prerequisites,
and the boundary between this slice and future app-bundle work.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-04/AC-01] README and operator-facing docs publish the canonical external-project deployment workflow, prerequisites, command path, and proof-review path. <!-- [SRS-04/AC-01] verify: manual, proof: ac-1.log -->
- [ ] [SRS-04/AC-02] Docs keep the boundary explicit that this slice stages and runs one external static-site project snapshot through shipped hosted primitives and does not yet ship an app bundle artifact contract or app bundle service runtime. <!-- [SRS-04/AC-02] verify: manual, proof: ac-2.log -->
