---
# system-managed
id: VEyjdN0nf
status: backlog
created_at: 2026-03-26T06:10:19
updated_at: 2026-03-26T06:14:07
# authored
title: Wire Repo-Level Mission Surface To External Project Deployment Proof
type: feat
operator-signal:
scope: VEyjUL2Zr/VEyjdNXnp
index: 2
---

# Wire Repo-Level Mission Surface To External Project Deployment Proof

## Summary

Wire the repo-level proof surface to the new external-project deployment
workflow so maintainers can review the runnable path and artifact from one
place.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-03/AC-01] The current repo-level proof entrypoint surfaces the canonical external-project deployment workflow, including the runnable proof path and the recorded artifact, as the primary operator-facing evidence for this slice. <!-- [SRS-03/AC-01] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-01/AC-01] A renderer-backed human-reviewable artifact is generated from the canonical external-project deployment workflow and linked through mission evidence. <!-- [SRS-NFR-01/AC-01] verify: manual, proof: ac-3.log -->
