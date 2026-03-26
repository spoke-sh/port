---
# system-managed
id: VEyjdNRno
status: done
created_at: 2026-03-26T06:10:19
updated_at: 2026-03-26T08:01:54
# authored
title: Implement Hosted External Project Deployment Workflow
type: feat
operator-signal:
scope: VEyjUL2Zr/VEyjdNXnp
index: 3
started_at: 2026-03-26T07:50:38
submitted_at: 2026-03-26T08:01:19
completed_at: 2026-03-26T08:01:54
---

# Implement Hosted External Project Deployment Workflow

## Summary

Implement the canonical hosted workflow that stages one external project
snapshot into hosted compute, runs it through Port, and proves success with a
host-side curl.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] The canonical proof workflow starts the repo-local hosted control plane and node agent, stages one external static-site project snapshot into hosted compute through `port guest copy` plus any minimal setup step, and keeps hosted machine, host-group, and route context explicit. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-02/AC-01] The workflow starts that staged project through `port service apply`, exposes it through `port guest forward`, and a host-side `curl` returns the expected payload from the staged project bytes. <!-- [SRS-02/AC-01] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
- [x] [SRS-NFR-02/AC-01] Existing hosted `guest copy`, `service`, and `guest forward` behavior remains intact outside the new canonical external-project proof path. <!-- [SRS-NFR-02/AC-01] verify: manual, proof: ac-3.log -->
