---
id: 1vzdMW000
title: Define Cloud Hypervisor Contract And Doctor Checks
type: feat
status: done
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:44
updated_at: 2026-03-09T09:35:46
started_at: 2026-03-09T09:27:06
completed_at: 2026-03-09T09:35:46
---

# Define Cloud Hypervisor Contract And Doctor Checks

## Summary

Define the executable Cloud Hypervisor machine, artifact, and doctor contract so
Port can distinguish the new substrate cleanly from Firecracker and surface the
host requirements before launch.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `port-model`, sample config, and `port doctor` represent Cloud Hypervisor as an executable `standard` substrate with explicit artifact selection and no implicit Firecracker fallback. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model && cargo test -q -p port-runtime resolve_artifact_metadata_reports_missing_selected_cloud_hypervisor_variant_without_fallback && cargo test -q -p port-cli && cargo run -q -p port-cli -- --config examples/port.toml doctor | rg "machine:demo-ch|cloud-hypervisor:demo-ch"', proof: ac-1.log -->
<!-- verify: command, SRS-01:end -->
<!-- verify: command, SRS-01:start, proof: ac-2.log -->
- [x] [SRS-01/AC-02] Unsupported Cloud Hypervisor host, architecture, or protection-mode combinations fail fast with substrate-specific diagnostics. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime doctor_report_surfaces_cloud_hypervisor_platform_and_binary_checks && cargo test -q -p port-runtime doctor_report_fails_fast_for_unsupported_cloud_hypervisor_platform_and_protection_mode', proof: ac-2.log -->
<!-- verify: command, SRS-01:end -->
