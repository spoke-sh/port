---
# system-managed
id: VGzxkoGrw
status: planned
epic: VGzxKV9OX
created_at: 2026-04-16T16:10:26
# authored
title: Guest-Agent Heartbeat And Age Surface
index: 1
updated_at: 2026-04-16T16:15:21
---

# Guest-Agent Heartbeat And Age Surface

> Introduce a guest-agent heartbeat probe and per-machine guest_refresh_age_seconds so the hosted control plane can tell apart a guest-side wedge (node-agent healthy, guest-agent silent) from a node-side wedge.

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
| [Add Ping Frame And Guest-Agent Heartbeat Wire Contract](../../../../stories/VGzxv3FOx/README.md) | feat | backlog |
| [Drive Periodic Guest Heartbeat Probe From Node-Agent](../../../../stories/VGzyLJtZw/README.md) | feat | backlog |
| [Surface Guest Refresh Age Seconds In Cluster Status](../../../../stories/VGzyLTlgJ/README.md) | feat | backlog |
<!-- END GENERATED -->
