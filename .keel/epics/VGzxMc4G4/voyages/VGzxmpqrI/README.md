---
# system-managed
id: VGzxmpqrI
status: in-progress
epic: VGzxMc4G4
created_at: 2026-04-16T16:10:33
# authored
title: Tier-1 Guest Restart And Attempt Accounting
index: 1
updated_at: 2026-04-16T17:20:32
started_at: 2026-04-16T17:20:37
---

# Tier-1 Guest Restart And Attempt Accounting

> Deliver tier-1 guest restart action and attempt accounting: when guest-side wedge trigger fires on an opted-in cluster, serialize against human lifecycle ops, stop-then-launch the machine, increment recovery_attempts.tier_1, and emit structured events. Default off.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 1/3 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Add Recovery Config Block And Attempt Counter Fields](../../../../stories/VH00js4Qb/README.md) | feat | done |
| [Fire Tier-1 Guest Restart From Wedge Detector Output](../../../../stories/VH00kDFiS/README.md) | feat | backlog |
| [Serialize Tier-1 Against Human Lifecycle Operations And Emit Events](../../../../stories/VH00kTTrf/README.md) | feat | backlog |
<!-- END GENERATED -->
