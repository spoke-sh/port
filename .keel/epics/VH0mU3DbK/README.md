---
# system-managed
id: VH0mU3DbK
created_at: 2026-04-16T19:31:56
# authored
title: Wire Wedge Detection And Recovery Into Live Control Plane
index: 35
---

# Wire Wedge Detection And Recovery Into Live Control Plane

> Mission VGzwzdKvB shipped wedge detection and a three-tier recovery ladder as library-level functions, but neither the detector tick nor the recovery runner is invoked from the live serve_control_plane process, and the per-machine wedge fields are only on MachineStatus rather than on the HostedK3sMachineTruth entries inside port cluster status --format json. Consumers (notably spoke-sh/infra) cannot see wedged_since on the cluster aggregate they already poll, and Port itself never executes the recovery ladder against a wedged guest at runtime. Close the deferred wiring so the existing internals become an active runtime contract.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/2 voyages complete, 0/3 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Cluster Aggregate Wedge Field Threading](voyages/VH0mjMP8p/) | draft | 0/1 |
| [Live Detector And Recovery Runner Wiring](voyages/VH0mlnCSq/) | draft | 0/2 |
<!-- END GENERATED -->
