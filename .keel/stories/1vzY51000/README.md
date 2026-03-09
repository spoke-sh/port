---
id: 1vzY51000
title: Define Pvm Host Kit Package Contract
type: feat
status: in-progress
created_at: 2026-03-09T03:43:19
updated_at: 2026-03-09T03:46:35
scope: 1vz3ck000/1vzY3z000
started_at: 2026-03-09T03:46:35
---

# Define Pvm Host Kit Package Contract

## Summary

Define the canonical `x86_64` Firecracker/PVM host-kit package contract in the
model, runtime, and hosted inventory surfaces so Port can describe prepared PVM
nodes as real package-bearing capacity rather than only "planned" metadata.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [x] [SRS-01/AC-01] Port defines a canonical PVM host-kit package contract that binds patched Firecracker identity, prepared host-kernel metadata, and required boot-line expectations for `x86_64` Firecracker/PVM nodes. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model sample_config_derives_hosted_node_pvm_host_kit_package_identity && cargo test -q -p port-runtime doctor_report_includes_hosted_pvm_host_kit_contract_checks', proof: ac-1.log -->
<!-- verify: command, SRS-01:end -->
<!-- verify: command, SRS-01:start -->
- [x] [SRS-01/AC-02] Unsupported or incomplete PVM host-kit combinations fail fast with explicit diagnostics and no standard-lane fallback in doctor or placement summaries. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model ready_hosted_pvm_lane_requires_host_kit_package_identity && cargo test -q -p port-runtime doctor_report_fails_fast_for_missing_pvm_boot_arg_and_binary', proof: ac-2.log -->
<!-- verify: command, SRS-01:end -->
