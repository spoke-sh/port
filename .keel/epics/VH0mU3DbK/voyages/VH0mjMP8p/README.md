---
# system-managed
id: VH0mjMP8p
status: draft
epic: VH0mU3DbK
created_at: 2026-04-16T19:32:55
# authored
title: Cluster Aggregate Wedge Field Threading
index: 1
---

# Cluster Aggregate Wedge Field Threading

> Thread the per-machine wedge fields onto HostedK3sMachineTruth so consumers polling port cluster status --format json see wedged_since, wedge_class, recovery_attempts, last_recovery_action, recovery_state, and guest_refresh_age_seconds on the cluster aggregate without needing per-machine port machine status calls.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 0/1 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Thread Wedge Fields Onto HostedK3sMachineTruth](../../../../stories/VH0oGGkcz/README.md) | feat | icebox |
<!-- END GENERATED -->
