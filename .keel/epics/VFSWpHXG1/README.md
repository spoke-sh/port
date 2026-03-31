---
# system-managed
id: VFSWpHXG1
created_at: 2026-03-31T08:27:36
# authored
title: Guest VM Outbound Networking
index: 19
mission: VFSRShGlI
---

# Guest VM Outbound Networking

> Firecracker VMs have no network interface — only vsock exists. This blocks CoreDNS resolution, Flux GitRepository cloning, containerd image pulls, and all outbound HTTP/HTTPS workloads, leaving the Flux GitOps loop in spoke-sh/infra stuck at 5/18 components.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 1/1 voyages complete, 3/3 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [TAP Networking and Host NAT for Local Firecracker VMs](voyages/VFSXWpO18/) | done | 3/3 |
<!-- END GENERATED -->
