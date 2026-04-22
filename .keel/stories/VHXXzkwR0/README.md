---
# system-managed
id: VHXXzkwR0
status: backlog
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T10:05:09
# authored
title: Add Control-Plane Placement Stall Observability And Regression Coverage
type: fix
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 4
---

# Add Control-Plane Placement Stall Observability And Regression Coverage

## Summary

Add explicit control-plane observability and regression coverage for placement
repair, alias canonicalization, timeout isolation, and degraded cluster
readiness so this failure mode becomes diagnosable and stays fixed.

## Acceptance Criteria

<!-- verify: unit, SRS-07:start -->
- [ ] [SRS-07/AC-01] Hosted control-plane logs or counters expose placement repair, alias rewrite, timeout isolation, and degraded readiness events with enough machine/node detail to debug rollout stalls. <!-- [SRS-07/AC-01] verify: targeted observability tests or log assertions -->
<!-- verify: unit, SRS-07:end -->
<!-- verify: unit, SRS-07:start -->
- [ ] [SRS-07/AC-02] Regression coverage proves missing placement fallback, alias repair, machine-list timeout isolation, and degraded cluster-status behavior. <!-- [SRS-07/AC-02] verify: targeted hosted-control-plane and cluster-status test suite -->
<!-- verify: unit, SRS-07:end -->
<!-- verify: unit, SRS-NFR-02:start -->
- [ ] [SRS-NFR-02/AC-03] Regression coverage fails if hosted request paths reintroduce control-plane self-calls or synchronous placement writes on read. <!-- [SRS-NFR-02/AC-03] verify: recursion and write-on-read guard tests -->
<!-- verify: unit, SRS-NFR-02:end -->
