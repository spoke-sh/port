---
# system-managed
id: VFHn1PHka
status: done
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T16:27:13
# authored
title: Prove Flux And Pulumi Operator Install Against Port Kubeconfig
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 3
started_at: 2026-03-29T16:23:41
submitted_at: 2026-03-29T16:27:10
completed_at: 2026-03-29T16:27:13
---

# Prove Flux And Pulumi Operator Install Against Port Kubeconfig

## Summary

Prove that Port's handed-off kubeconfig is GitOps-capable by running Flux and
the Pulumi Kubernetes Operator Helm install directly against the local cluster.

## Acceptance Criteria

- [x] [SRS-04/AC-01] `flux install` succeeds against the kubeconfig returned by `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json`. Verified in `EVIDENCE/ac-1.cluster-kubeconfig.json` and `EVIDENCE/ac-1.flux-install.log`. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-NFR-01/AC-02] `helm upgrade --install pulumi-kubernetes-operator ...` succeeds against the same handed-off kubeconfig, and the proof records the live host-side client commands rather than only Port-local surface checks. Verified in `EVIDENCE/ac-2.helm-install.log`, `EVIDENCE/ac-2.operator-pods.log`, and `EVIDENCE/ac-2.cluster-down.json`. <!-- verify: manual, SRS-NFR-01:start:end -->
