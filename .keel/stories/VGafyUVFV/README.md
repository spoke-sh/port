---
# system-managed
id: VGafyUVFV
status: backlog
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T08:28:03
# authored
title: Surface Control-Plane Host Spread In Cluster Status
type: feat
operator-signal:
scope: VGYFpfUph/VGafx2cmq
index: 2
---

# Surface Control-Plane Host Spread In Cluster Status

## Summary

Expose execution-host spread and HA satisfaction in cluster-facing output so an
operator can see whether the hosted AWS PVM control plane is truly multi-host
or only shaped like HA on paper.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Hosted placement state or lifecycle reports record which execution host each control-plane machine occupies. <!-- verify: automated, SRS-02:start:end -->
- [ ] [SRS-03/AC-02] `port cluster status` or equivalent cluster-facing output reports whether the current control plane satisfies the real-HA spread contract instead of inferring HA from machine count alone. <!-- verify: automated, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-03] The rendered HA truth remains explicitly scoped to hosted AWS `x86_64` PVM rather than broadening to generic hosted HA language. <!-- verify: manual, SRS-NFR-01:start:end -->
