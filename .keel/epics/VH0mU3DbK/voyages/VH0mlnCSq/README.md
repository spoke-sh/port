---
# system-managed
id: VH0mlnCSq
status: draft
epic: VH0mU3DbK
created_at: 2026-04-16T19:33:04
# authored
title: Live Detector And Recovery Runner Wiring
index: 2
---

# Live Detector And Recovery Runner Wiring

> Spawn the wedge detector tick and recovery runner from serve_control_plane so the existing pure-function library code becomes an active runtime contract: wedge_state populates against live heartbeat ages, decide_recovery_action drives tier-1 guest restart and tier-2 overlay recreate against the runtime root, and tier-3 emits an awaiting-host-recycle signal. Recovery actions stay opt-in via ClusterRecoveryConfig.enabled.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 0/2 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Spawn Wedge Detector Tick Loop From Live Control Plane](../../../../stories/VH0owEJfH/README.md) | feat | icebox |
| [Spawn Recovery Runner Loop With Tier-1 Through Tier-3](../../../../stories/VH0oxFqBk/README.md) | feat | icebox |
<!-- END GENERATED -->
