---
id: VDi3O7KjN
title: Implement Hosted HTTP App Proof Workflow
type: feat
status: done
created_at: 2026-03-12T19:13:16
updated_at: 2026-03-12T19:33:22
operator-signal: 
scope: VDi2y6gch/VDi3LHFpb
index: 3
started_at: 2026-03-12T19:21:47
submitted_at: 2026-03-12T19:33:04
completed_at: 2026-03-12T19:33:22
---

# Implement Hosted HTTP App Proof Workflow

## Summary

Implement the canonical hosted workflow that launches one minimal HTTP
application through Port, exposes it through Port, and proves success with a
host-side curl.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-01/AC-01] The canonical proof workflow starts the repo-local hosted control plane and node agent, applies one minimal hosted HTTP service through `port service apply`, and keeps hosted machine, host-group, and route context explicit. <!-- [SRS-01/AC-01] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-02/AC-01] The workflow exposes that hosted HTTP service through `port guest forward`, and a host-side `curl` returns the expected application payload. <!-- [SRS-02/AC-01] verify: manual, proof: ac-4.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-01] Existing hosted service and hosted guest-forward behavior remains intact outside the new canonical app-hosting proof path. <!-- [SRS-NFR-02/AC-01] verify: manual, proof: ac-6.log -->
