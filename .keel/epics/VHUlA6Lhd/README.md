---
# system-managed
id: VHUlA6Lhd
created_at: 2026-04-21T22:34:51
# authored
title: Harden Hosted Wedge Detection And Runtime Recovery
index: 36
mission: VHUlA5uhc
---

# Harden Hosted Wedge Detection And Runtime Recovery

> Prod exposed two gaps in hosted recovery: managed k3s services do not restart on unhealthy healthchecks, and guest wedge classification falls back to machine placement age, producing false positives that make the wedge endpoint unsafe for auto-recovery.

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
| [Hosted Guest Recovery Fidelity](voyages/VHUlRjuw5/) | done | 1/1 |
<!-- END GENERATED -->
