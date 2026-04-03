---
# system-managed
id: VFgcPDfEj
created_at: 2026-04-02T18:17:35
# authored
title: AWS Hosted PVM Preparation And Launch
index: 22
mission: VFgcM1Zpu
---

# AWS Hosted PVM Preparation And Launch

> Port has a generic prepared-node x86_64 Firecracker/PVM proof, but it still lacks a real provider-backed cloud-aws hosted runtime contract on regular AWS VMs. Downstream infrastructure currently falls onto the wrong standard Firecracker/KVM lane because Port does not yet own the AWS-specific host-kit preparation, readiness import, and live cloud-aws PVM proof surface.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 1/2 voyages complete, 3/4 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [AWS PVM Host Kit Preparation](voyages/VFgclbAzD/) | done | 2/2 |
| [Cloud Aws PVM Runtime Proof](voyages/VFgclbQzC/) | in-progress | 1/2 |
<!-- END GENERATED -->
