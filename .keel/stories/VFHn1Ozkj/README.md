---
# system-managed
id: VFHn1Ozkj
status: done
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T16:23:22
# authored
title: Harden Kubeconfig Handoff And Kubernetes Discovery
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 2
started_at: 2026-03-29T16:18:40
submitted_at: 2026-03-29T16:23:15
completed_at: 2026-03-29T16:23:22
---

# Harden Kubeconfig Handoff And Kubernetes Discovery

## Summary

Harden the handed-off kubeconfig and API reachability so normal Kubernetes
clients can use the Port-owned local cluster directly and discover the
resources needed for GitOps bootstrap.

## Acceptance Criteria

- [x] [SRS-02/AC-01] `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` returns a kubeconfig that works with normal Kubernetes clients without downstream rewriting. Verified in `EVIDENCE/ac-1.cluster-status.json`, `EVIDENCE/ac-2.cluster-kubeconfig.json`, and `EVIDENCE/ac-2.kubectl.log`. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-02] `kubectl api-resources -o name` against the handed-off kubeconfig includes at least `deployments.apps`, `namespaces`, `serviceaccounts`, `secrets`, `configmaps`, and `customresourcedefinitions.apiextensions.k8s.io`. Verified in `EVIDENCE/ac-2.api-resources.log`. <!-- verify: manual, SRS-03:start:end -->
