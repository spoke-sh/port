---
# system-managed
id: VGzxMc4G4
created_at: 2026-04-16T16:08:53
# authored
title: Hosted Fleet Recovery Ladder And Host Recycle
index: 34
mission: VGzwzdKvB
---

# Hosted Fleet Recovery Ladder And Host Recycle

> Once Port can detect a wedged microVM, it still has no automatic recovery path — operators must SSH to the host and intervene manually, which is slow and scales poorly. Port owns the runtime and the process graph that lifecycle actions touch, so recovery must live inside Port. Deliver a per-cluster opt-in recovery ladder (tier-1 guest restart, tier-2 overlay recreate, tier-3 host recycle with a single-tenant gate), a per-provider host_reboot integration reused by tier-3, serialization against in-flight human lifecycle operations, a sticky recovery_exhausted terminal state, and an explicit port machine unfence reset path. Keep enabled=false by default so production has to opt in per cluster.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 3/3 voyages complete, 10/10 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Tier-1 Guest Restart And Attempt Accounting](voyages/VGzxmpqrI/) | done | 3/3 |
| [Tier-2 Overlay Recreate And Tier-3 Escalation Signal](voyages/VGzxnR97R/) | done | 3/3 |
| [Tier-3 Signal Persistence, Unfence Reset, And End-To-End Proof](voyages/VGzxoN8WF/) | done | 3/3 |
<!-- END GENERATED -->
