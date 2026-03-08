---
id: 1vz2oQ000
title: Model Substrates And Protection Modes
type: feat
status: done
created_at: 2026-03-07T17:20:06
updated_at: 2026-03-07T17:40:24
scope: 1vz2eV000/1vz2ky000
started_at: 2026-03-07T17:25:37
submitted_at: 2026-03-07T17:40:06
completed_at: 2026-03-07T17:40:24
---

# Model Substrates And Protection Modes

## Summary

Extend Port's canonical model, validation, and operator docs so runtime
capability is expressed through substrate, protection mode, architecture, and
artifact compatibility instead of through provider identity alone.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `port-model` can represent backend, protection mode, architecture, and artifact-compatibility metadata for machines and artifacts while the existing local Firecracker/KVM sample config still parses and validates. <!-- [SRS-01/AC-01] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-model, proof: ac-1.log-->
<!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] [SRS-01/AC-02] Port publishes canonical substrate terms and a support matrix covering Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, Apple Virtualization Framework, and the explicit arm64 protected-virtualization research lane. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Execution Lanes|Cloud Hypervisor|Apple Virtualization Framework|research lane|protection_mode" README.md docs/cloud.md docs/operators.md examples/port.toml', proof: ac-2.log-->
<!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->
- [x] [SRS-01/AC-03] Unsupported backend, protection-mode, or architecture combinations fail fast with actionable model or CLI diagnostics instead of silently degrading. <!-- [SRS-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime doctor_report_includes_machine_lane_checks -- --exact && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime launch_rejects_unsupported_pvm_artifact_contract -- --exact', proof: ac-3.log-->
