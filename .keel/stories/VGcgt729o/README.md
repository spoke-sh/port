---
# system-managed
id: VGcgt729o
status: done
created_at: 2026-04-12T16:39:10
updated_at: 2026-04-12T17:12:16
# authored
title: Model Hosted Machine And Service Truth In Cluster Status
type: feat
operator-signal:
scope: VGcgU7q58/VGcghuutu
index: 1
started_at: 2026-04-12T16:59:16
completed_at: 2026-04-12T17:12:16
---

# Model Hosted Machine And Service Truth In Cluster Status

## Summary

Model hosted machine identity, placement, managed-service state, and related
runtime truth inside the canonical hosted cluster status payload.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Hosted machine identity, placement, and managed-service truth are present in one canonical status payload. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_contract', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The canonical payload remains machine-readable enough for downstream consumers to adopt without schema forks. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift', SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-02/AC-03] The canonical payload is exposed through the existing cluster status surface instead of a one-off diagnostic command. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms && cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth', SRS-02:start:end, proof: ac-3.log-->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract`, proving the canonical hosted status payload now includes structured machine identity, placement, and managed-service truth.
- AC-02: `EVIDENCE/ac-2.log` records `cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift`, proving the hosted status JSON surface remains machine-readable with stable `machines` and `managed_services` arrays.
- AC-03: `EVIDENCE/ac-3.log` records `cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms` plus `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth`, proving the new truth is exposed through the existing hosted cluster lifecycle and status surfaces rather than a separate diagnostic path.
