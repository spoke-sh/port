---
# system-managed
id: VFDk8gRoD
status: backlog
created_at: 2026-03-28T19:46:24
updated_at: 2026-03-28T19:50:21
# authored
title: Implement Cluster Lifecycle Health And Kubeconfig Surfaces
type: feat
operator-signal:
scope: VFDhlRjOf/VFDk8fdnG
index: 3
---

# Implement Cluster Lifecycle Health And Kubeconfig Surfaces

## Summary

Implement the first cluster lifecycle surface so Port can bring a local K3s
cluster up, report whether it is healthy, return kubeconfig directly, and tear
it down without infra-side choreography.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Port provides cluster lifecycle and access behavior for the first local cluster without manual API forwarding or kubeconfig rewriting outside Port. <!-- verify: manual, SRS-03 -->
- [ ] [SRS-NFR-03/AC-02] Cluster-health output clearly distinguishes Port-owned cluster readiness from later downstream bootstrap or networking work. <!-- verify: manual, SRS-NFR-03 -->
