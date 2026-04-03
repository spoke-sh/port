---
# system-managed
id: VFhLhfrqk
created_at: 2026-04-02T21:17:30
# authored
title: AWS PVM Host Kit Nix Surface
index: 26
mission: VFhLhfYqn
---

# AWS PVM Host Kit Nix Surface

> Port owns the AWS x86_64 PVM host contract conceptually, but downstream image pipelines still cannot consume that contract as a first-class Nix module or package surface. That forces AMI builders to depend on out-of-band repo-local module paths instead of a Port-owned source of truth for the host kernel, boot args, patched firecracker-pvm, and readiness identity used by prepare-pvm-node.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 1/1 voyages complete, 1/1 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Export And Prove AWS PVM Host Kit Module](voyages/VFhLjViAG/) | done | 1/1 |
<!-- END GENERATED -->
