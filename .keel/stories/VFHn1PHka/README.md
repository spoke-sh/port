---
# system-managed
id: VFHn1PHka
status: backlog
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T12:24:38
# authored
title: Prove Flux And Pulumi Operator Install Against Port Kubeconfig
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 3
---

# Prove Flux And Pulumi Operator Install Against Port Kubeconfig

## Summary

Prove that Port's handed-off kubeconfig is GitOps-capable by running Flux and
the Pulumi Kubernetes Operator Helm install directly against the local cluster.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] `flux install` succeeds against the kubeconfig returned by `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json`. <!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-NFR-01/AC-02] `helm upgrade --install pulumi-kubernetes-operator ...` succeeds against the same handed-off kubeconfig, and the proof records the live host-side client commands rather than only Port-local surface checks. <!-- verify: manual, SRS-NFR-01:start:end -->
