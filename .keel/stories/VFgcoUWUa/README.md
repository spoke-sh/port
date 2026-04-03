---
# system-managed
id: VFgcoUWUa
status: done
created_at: 2026-04-02T18:19:12
updated_at: 2026-04-02T18:55:03
# authored
title: Implement AWS Node Preparation Workflow
type: feat
operator-signal:
scope: VFgcPDfEj/VFgclbAzD
index: 2
started_at: 2026-04-02T18:35:08
submitted_at: 2026-04-02T18:54:59
completed_at: 2026-04-02T18:55:03
---

# Implement AWS Node Preparation Workflow

## Summary

Implement the canonical preparation/import workflow that moves an eligible AWS
node into the hosted PVM-ready state for `cloud-aws` and exposes the resulting
readiness through operator-visible Port surfaces.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] `port control-plane prepare-pvm-node` prepares or imports AWS hosted PVM readiness for an x86_64 AWS node without manual config overlays or hand-edited imported inventory. <!-- [SRS-02/AC-01] verify: manual, proof: ac-2.log-->
<!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-03/AC-02] Doctor, status, or imported-readiness surfaces explain missing or stale AWS host-kit prerequisites with `cloud-aws` guidance and no standard-lane ambiguity. <!-- [SRS-03/AC-02] verify: manual, proof: ac-4.log-->
