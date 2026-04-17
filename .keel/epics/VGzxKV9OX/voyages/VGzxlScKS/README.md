---
# system-managed
id: VGzxlScKS
status: done
epic: VGzxKV9OX
created_at: 2026-04-16T16:10:28
# authored
title: Wedge Detector And Cluster Status Fields
index: 2
updated_at: 2026-04-16T17:08:26
started_at: 2026-04-16T17:08:30
completed_at: 2026-04-16T17:41:42
---

# Wedge Detector And Cluster Status Fields

> Introduce a configurable wedge detector that consumes both refresh_age_seconds (node-side) and guest_refresh_age_seconds (guest-side) and surfaces wedged_since, wedge_class, recovery_attempts, last_recovery_action, and recovery_state in port cluster status --format json. No recovery actions yet.

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
**Progress:** 3/3 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Add Cluster Detection Config Block With Threshold Defaults](../../../../stories/VH00Brdus/README.md) | feat | done |
| [Implement Control-Plane Wedge Detector Task](../../../../stories/VH00C3h3h/README.md) | feat | done |
| [Surface Wedged Since And Wedge Class In Cluster Status](../../../../stories/VH00CG8GV/README.md) | feat | done |
<!-- END GENERATED -->
