---
# system-managed
id: VGYQnafG0
status: in-progress
created_at: 2026-04-11T23:10:10
updated_at: 2026-04-11T23:12:19
# authored
title: Model Machine Runtime Class Contracts For Builder Lanes
type: feat
operator-signal:
scope: VGYFpewpf/VGYQ4zrrX
index: 1
started_at: 2026-04-11T23:12:19
---

# Model Machine Runtime Class Contracts For Builder Lanes

## Summary

Add the shared runtime-class contract to Port's machine model so
`workspace-scratch-builder` becomes an explicit, validated execution lane and
`blessed-closure-promotion-runner` is reserved in the same vocabulary for the
adjacent trusted lane.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `MachineSpec` can serialize and deserialize an explicit runtime-class declaration instead of relying on machine naming or comments. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Port defines a canonical `workspace-scratch-builder` runtime-class contract that records its workspace-bound writable-state categories and explicitly untrusted posture. <!-- verify: automated, SRS-02:start:end -->
- [ ] [SRS-04/AC-03] Config validation rejects contradictory builder runtime-class declarations, including missing writable-state metadata or publish-trusted posture on the scratch lane. <!-- verify: automated, SRS-04:start:end -->
