---
id: 1vz2oQ000
title: Model Substrates And Protection Modes
type: feat
status: in-progress
created_at: 2026-03-07T17:20:06
updated_at: 2026-03-07T17:25:37
scope: 1vz2eV000/1vz2ky000
started_at: 2026-03-07T17:25:37
---

# Model Substrates And Protection Modes

## Summary

Extend Port's canonical model, validation, and operator docs so runtime
capability is expressed through substrate, protection mode, architecture, and
artifact compatibility instead of through provider identity alone.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port-model` can represent backend, protection mode, architecture, and artifact-compatibility metadata for machines and artifacts while the existing local Firecracker/KVM sample config still parses and validates.
- [ ] [SRS-06/AC-02] Port publishes canonical substrate terms and a support matrix covering Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, Apple Virtualization Framework, and the explicit arm64 protected-virtualization research lane.
- [ ] [SRS-01/AC-03] Unsupported backend, protection-mode, or architecture combinations fail fast with actionable model or CLI diagnostics instead of silently degrading.
