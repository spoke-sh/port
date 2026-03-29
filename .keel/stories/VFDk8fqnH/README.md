---
# system-managed
id: VFDk8fqnH
status: backlog
created_at: 2026-03-28T19:46:24
updated_at: 2026-03-28T19:50:21
# authored
title: Add Cluster CLI And Config Contract
type: feat
operator-signal:
scope: VFDhlRjOf/VFDk8fdnG
index: 1
---

# Add Cluster CLI And Config Contract

## Summary

Introduce the first named cluster-facing Port surface and local cluster contract
so operators stop assembling the local K3s workflow from raw `machine` and
`guest exec` steps.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Port exposes a named cluster-facing surface for the first local K3s lane and fails fast on unsupported multi-node, hosted, or AWS requests in this slice. <!-- verify: manual, SRS-01 -->
- [ ] [SRS-NFR-01/AC-02] Existing `machine`, `guest`, `service`, and hosted-K3s primitives remain available as underlying implementation substrate without silent regressions. <!-- verify: manual, SRS-NFR-01 -->
