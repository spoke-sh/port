---
# system-managed
id: VFDk8gRoD
status: done
created_at: 2026-03-28T19:46:24
updated_at: 2026-03-28T22:04:07
# authored
title: Implement Cluster Lifecycle Health And Kubeconfig Surfaces
type: feat
operator-signal:
scope: VFDhlRjOf/VFDk8fdnG
index: 3
started_at: 2026-03-28T21:43:20
submitted_at: 2026-03-28T22:04:05
completed_at: 2026-03-28T22:04:07
---

# Implement Cluster Lifecycle Health And Kubeconfig Surfaces

## Summary

Implement the first cluster lifecycle surface so Port can bring a local K3s
cluster up, report whether it is healthy, return kubeconfig directly, and tear
it down without infra-side choreography.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-03/AC-01] Port provides cluster lifecycle and access behavior for the first local cluster without manual API forwarding or kubeconfig rewriting outside Port. <!-- verify: manual, SRS-03, proof: ac-1.log-->
<!-- verify: manual, SRS-NFR-03:start:end -->
- [x] [SRS-NFR-03/AC-02] Cluster-health output clearly distinguishes Port-owned cluster readiness from later downstream bootstrap or networking work. <!-- verify: manual, SRS-NFR-03, proof: ac-2.log-->
