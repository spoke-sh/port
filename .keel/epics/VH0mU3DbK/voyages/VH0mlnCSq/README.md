---
# system-managed
id: VH0mlnCSq
status: done
epic: VH0mU3DbK
created_at: 2026-04-16T19:33:04
# authored
title: Live Detector And Recovery Runner Wiring
index: 2
updated_at: 2026-04-16T20:37:29
started_at: 2026-04-16T20:37:35
completed_at: 2026-04-16T23:46:38
---

# Live Detector And Recovery Runner Wiring

> Spawn the wedge detector tick and recovery runner from serve_control_plane so the existing pure-function library code becomes an active runtime contract: wedge_state populates against live heartbeat ages, decide_recovery_action drives tier-1 guest restart and tier-2 overlay recreate against the runtime root, and tier-3 emits an awaiting-host-recycle signal. Recovery actions stay opt-in via ClusterRecoveryConfig.enabled.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
| [VOYAGE_REPORT.md](VOYAGE_REPORT.md) | Narrative summary of implementation and evidence |
| [COMPLIANCE_REPORT.md](COMPLIANCE_REPORT.md) | Traceability matrix and verification proof |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 2/2 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Spawn Wedge Detector Tick Loop From Live Control Plane](../../../../stories/VH0owEJfH/README.md) | feat | done |
| [Spawn Recovery Runner Loop With Tier-1 Through Tier-3](../../../../stories/VH0oxFqBk/README.md) | feat | done |
<!-- END GENERATED -->
