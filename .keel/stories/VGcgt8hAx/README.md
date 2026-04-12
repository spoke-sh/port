---
# system-managed
id: VGcgt8hAx
status: done
created_at: 2026-04-12T16:39:10
updated_at: 2026-04-12T16:56:41
# authored
title: Report Legacy Detached Runtime Drift In Cluster Status
type: feat
operator-signal:
scope: VGcgU7q58/VGcghuutu
index: 2
started_at: 2026-04-12T16:50:24
completed_at: 2026-04-12T16:56:41
---

# Report Legacy Detached Runtime Drift In Cluster Status

## Summary

Teach the hosted cluster status contract to report legacy detached K3s PID/log
drift explicitly so downstream consumers can reject that runtime shape.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Hosted status reports legacy detached-runtime drift when PID/log artifacts appear outside managed-service ownership. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_reports_legacy_detached_runtime_drift && cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-02] The legacy-drift signal does not create a second contradictory hosted truth path. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_contract && cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth', SRS-NFR-02:start:end, proof: ac-2.log-->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records `cargo test -q -p port-runtime hosted_k3s_cluster_access_reports_legacy_detached_runtime_drift` plus `cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift`, proving the canonical hosted status payload reports detached-runtime drift and exposes the structured signal through `port cluster status --format json`.
- AC-02: `EVIDENCE/ac-2.log` records `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract` plus `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth`, proving the clear-state report preserves the existing hosted truth path while surfacing the new drift field.
