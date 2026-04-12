---
# system-managed
id: VGYQnbJG2
status: backlog
created_at: 2026-04-11T23:10:11
updated_at: 2026-04-11T23:11:37
# authored
title: Define Promotion Runner Runtime Class Guard Rails
type: feat
operator-signal:
scope: VGYFpf9pg/VGYQ50GrI
index: 1
---

# Define Promotion Runner Runtime Class Guard Rails

## Summary

Define the promotion-runner runtime class as a clean-room lane whose identity,
declared-input posture, and validation rules remain distinct from
`workspace-scratch-builder`.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Port models `blessed-closure-promotion-runner` as a dedicated runtime class rather than as a trust flag on `workspace-scratch-builder`. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] The promotion-runner contract declares clean-room and immutable-input posture explicitly. <!-- verify: automated, SRS-02:start:end -->
- [ ] [SRS-03/AC-03] Port machine-facing proof surfaces carry promotion-runner identity and posture so downstream publication tooling can link runtime evidence to what ran. <!-- verify: automated, SRS-03:start:end -->
- [ ] [SRS-04/AC-03] Config validation rejects promotion-runner declarations that try to reuse scratch writable state or creator credentials. <!-- verify: automated, SRS-04:start:end -->
- [ ] [SRS-NFR-03/AC-04] Verification includes at least one negative-path proof for a collapsed scratch/promotion trust boundary. <!-- verify: automated, SRS-NFR-03:start:end -->
