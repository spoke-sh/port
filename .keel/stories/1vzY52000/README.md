---
id: 1vzY52000
title: Add Pvm Artifact Mobility Workflow
type: feat
status: done
created_at: 2026-03-09T03:43:20
updated_at: 2026-03-09T08:25:38
scope: 1vz3ck000/1vzY3z000
started_at: 2026-03-09T03:53:11
completed_at: 2026-03-09T08:25:38
---

# Add Pvm Artifact Mobility Workflow

## Summary

Extend the canonical `port artifacts ...` surface so `x86_64/firecracker/pvm`
kernel and guest-image variants can be built, validated, pushed, and pulled
without implicit reuse of the standard Firecracker lane.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log-->
- [x] [SRS-02/AC-01] `port artifacts build|validate|push|pull` supports the `x86_64/firecracker/pvm` kernel and guest-image variants through the canonical artifact model. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli cli_artifact_build_and_validate_selected_pvm_kernel_variant -- --exact && cargo test -q -p port-cli cli_artifact_build_and_validate_selected_pvm_guest_image_variant -- --exact && cargo test -q -p port-cli cli_artifact_push_and_pull_round_trip_pvm_variant_contract_for_kernel_and_guest_image -- --exact', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start, proof: ac-2.log-->
- [x] [SRS-02/AC-02] PVM artifact mobility remains deterministic and explicit: missing variants fail with the selected variant name and Port does not fall back to standard Firecracker artifacts. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime tests::resolve_artifact_metadata_distinguishes_standard_and_pvm_paths -- --exact && cargo test -q -p port-runtime tests::resolve_guest_image_metadata_distinguishes_standard_and_pvm_paths -- --exact && cargo test -q -p port-runtime tests::resolve_artifact_metadata_reports_missing_selected_pvm_variant_without_fallback -- --exact', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->
