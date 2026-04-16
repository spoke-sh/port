---
# system-managed
id: VGzxnR97R
status: draft
epic: VGzxMc4G4
created_at: 2026-04-16T16:10:36
# authored
title: Tier-2 Overlay Recreate And Tier-3 Host Recycle
index: 2
---

# Tier-2 Overlay Recreate And Tier-3 Host Recycle

> Deliver tier-2 overlay recreate action with graceful skip for non-overlay machines, and tier-3 host recycle gated behind the single-tenant host check and a per-provider host_reboot integration (AWS EC2 reboot, SSH systemctl restart). Default off.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 0/3 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Wire Tier-2 Overlay Recreate With Graceful Skip](../../../../stories/VH01FRXDf/README.md) | feat | icebox |
| [Implement Host Reboot Client For AWS And SSH Providers](../../../../stories/VH01Fk4SW/README.md) | feat | icebox |
| [Fire Tier-3 Host Recycle Behind Single-Tenant Gate](../../../../stories/VH01FzHcw/README.md) | feat | icebox |
<!-- END GENERATED -->
