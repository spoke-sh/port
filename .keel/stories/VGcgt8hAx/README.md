---
# system-managed
id: VGcgt8hAx
status: backlog
created_at: 2026-04-12T16:39:10
updated_at: 2026-04-12T16:39:43
# authored
title: Report Legacy Detached Runtime Drift In Cluster Status
type: feat
operator-signal:
scope: VGcgU7q58/VGcghuutu
index: 2
---

# Report Legacy Detached Runtime Drift In Cluster Status

## Summary

Teach the hosted cluster status contract to report legacy detached K3s PID/log
drift explicitly so downstream consumers can reject that runtime shape.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Hosted status reports legacy detached-runtime drift when PID/log artifacts appear outside managed-service ownership. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-NFR-02/AC-02] The legacy-drift signal does not create a second contradictory hosted truth path. <!-- verify: automated, SRS-NFR-02:start:end -->
