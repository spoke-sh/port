---
# system-managed
id: VHXXzkVPt
status: backlog
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T10:05:09
# authored
title: Split Hosted Cluster Readiness From Kubeconfig Handoff
type: feat
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 3
---

# Split Hosted Cluster Readiness From Kubeconfig Handoff

## Summary

Refactor hosted cluster readiness so machine/runtime, API, node visibility, and
kubeconfig handoff are reported as separate gates. That keeps `cluster status`
truthful and bounded even when kubeconfig retrieval is the only failing step.

## Acceptance Criteria

<!-- verify: integration, SRS-05:start -->
- [ ] [SRS-05/AC-01] `cluster status` returns structured readiness detail for machine/runtime, API visibility, node visibility, and kubeconfig availability instead of collapsing those states into one opaque hosted failure. <!-- [SRS-05/AC-01] verify: targeted hosted cluster-status tests -->
<!-- verify: integration, SRS-05:end -->
<!-- verify: integration, SRS-06:start -->
- [ ] [SRS-06/AC-02] `cluster kubeconfig` fails only on the kubeconfig handoff boundary and preserves already-established machine/API readiness detail rather than reusing the generic `cluster status` failure path. <!-- [SRS-06/AC-02] verify: hosted kubeconfig handoff tests -->
<!-- verify: integration, SRS-06:end -->
<!-- verify: unit, SRS-NFR-03:start -->
- [ ] [SRS-NFR-03/AC-03] The richer readiness fidelity remains on the canonical `port cluster status` and `port cluster kubeconfig` surfaces without introducing a second operator workflow. <!-- [SRS-NFR-03/AC-03] verify: CLI/output regression tests -->
<!-- verify: unit, SRS-NFR-03:end -->
