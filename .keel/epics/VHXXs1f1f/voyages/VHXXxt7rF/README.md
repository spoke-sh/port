---
# system-managed
id: VHXXxt7rF
status: in-progress
epic: VHXXs1f1f
created_at: 2026-04-22T10:01:15
# authored
title: Recover Hosted Placement Truth Without Read-Path Stall
index: 1
updated_at: 2026-04-22T10:05:09
started_at: 2026-04-22T10:05:10
---

# Recover Hosted Placement Truth Without Read-Path Stall

> Return live, partial hosted cluster truth when stored placement drifts or disappears, keep request paths non-blocking, and expose enough status fidelity for rollout and auto-recovery to trust the control plane.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 2/4 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Make Hosted Machine And Service Status Live-First Under Placement Drift](../../../../stories/VHXXzjYOd/README.md) | feat | done |
| [Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background](../../../../stories/VHXXzjuOa/README.md) | feat | done |
| [Split Hosted Cluster Readiness From Kubeconfig Handoff](../../../../stories/VHXXzkVPt/README.md) | feat | backlog |
| [Add Control-Plane Placement Stall Observability And Regression Coverage](../../../../stories/VHXXzkwR0/README.md) | fix | backlog |
<!-- END GENERATED -->
