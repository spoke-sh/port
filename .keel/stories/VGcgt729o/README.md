---
# system-managed
id: VGcgt729o
status: backlog
created_at: 2026-04-12T16:39:10
updated_at: 2026-04-12T16:39:43
# authored
title: Model Hosted Machine And Service Truth In Cluster Status
type: feat
operator-signal:
scope: VGcgU7q58/VGcghuutu
index: 1
---

# Model Hosted Machine And Service Truth In Cluster Status

## Summary

Model hosted machine identity, placement, managed-service state, and related
runtime truth inside the canonical hosted cluster status payload.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Hosted machine identity, placement, and managed-service truth are present in one canonical status payload. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-NFR-01/AC-02] The canonical payload remains machine-readable enough for downstream consumers to adopt without schema forks. <!-- verify: automated, SRS-NFR-01:start:end -->
- [ ] [SRS-02/AC-03] The canonical payload is exposed through the existing cluster status surface instead of a one-off diagnostic command. <!-- verify: automated, SRS-02:start:end -->
