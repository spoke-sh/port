---
# system-managed
id: VGYQnafG0
status: done
created_at: 2026-04-11T23:10:10
updated_at: 2026-04-11T23:19:14
# authored
title: Model Machine Runtime Class Contracts For Builder Lanes
type: feat
operator-signal:
scope: VGYFpewpf/VGYQ4zrrX
index: 1
started_at: 2026-04-11T23:12:19
completed_at: 2026-04-11T23:19:14
---

# Model Machine Runtime Class Contracts For Builder Lanes

## Summary

Add the shared runtime-class contract to Port's machine model so
`workspace-scratch-builder` becomes an explicit, validated execution lane with
declared writable roots and bounded trust posture.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `MachineSpec` can serialize and deserialize an explicit runtime-class declaration instead of relying on machine naming or comments. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-1.roundtrip.log`. <!-- verify: command, SRS-01:start:end, proof: ac-1.roundtrip.log -->
- [x] [SRS-02/AC-02] Port defines a canonical `workspace-scratch-builder` runtime-class contract that records its workspace-bound writable-state categories and explicitly untrusted posture. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-2.builder-contract.log`. <!-- verify: command, SRS-02:start:end, proof: ac-2.builder-contract.log -->
- [x] [SRS-04/AC-03] Config validation rejects contradictory builder runtime-class declarations, including missing writable-state metadata or publish-trusted posture on the scratch lane. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class -- --nocapture` in `EVIDENCE/ac-3.validation.log`. <!-- verify: command, SRS-04:start:end, proof: ac-3.validation.log -->
