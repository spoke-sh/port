---
# system-managed
id: VGzxKV9OX
created_at: 2026-04-16T16:08:45
# authored
title: Wedge Detection And Guest Heartbeat Surface
index: 33
mission: VGzwzdKvB
---

# Wedge Detection And Guest Heartbeat Surface

> Hosted fleet operators cannot tell when a microVM guest is wedged: node-agent refresh age only reports node-side liveness, not guest-agent liveness, so stale or silent guests hide behind a Live node. Without a signal that distinguishes a node-side wedge (node-agent silent) from a guest-side wedge (node-agent healthy, guest-agent silent), Port cannot drive tier-appropriate recovery. Introduce a guest-agent heartbeat and a wedge detector that surfaces wedged_since, wedge_class, recovery_attempts, last_recovery_action, and recovery_state on the hosted cluster status contract, without taking recovery actions yet.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/2 voyages complete, 0/6 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Guest-Agent Heartbeat And Age Surface](voyages/VGzxkoGrw/) | planned | 0/3 |
| [Wedge Detector And Cluster Status Fields](voyages/VGzxlScKS/) | draft | 0/3 |
<!-- END GENERATED -->
