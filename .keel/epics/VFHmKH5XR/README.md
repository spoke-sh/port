---
# system-managed
id: VFHmKH5XR
created_at: 2026-03-29T12:20:10
# authored
title: Replace Demo Local Cluster Stub With Real K3s Runtime
index: 18
mission: VFHmKGaXQ
---

# Replace Demo Local Cluster Stub With Real K3s Runtime

> Port's current local cluster handoff is good enough for cluster up, kubeconfig, and kubectl get nodes, but it is still a demo API rather than a GitOps-capable single-node K3s control plane. Downstream infra can prove cluster handoff readiness, yet flux install, helm install, broad api-resources discovery, and unchanged infra bootstrap/health still fail or remain out of scope because the local lane is a stub runtime.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/1 voyages complete, 1/4 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Replace Demo API With GitOps-Capable Local K3s Runtime](voyages/VFHmctWC5/) | in-progress | 1/4 |
<!-- END GENERATED -->
