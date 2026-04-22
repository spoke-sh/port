---
# system-managed
id: VHXXzkwR0
status: done
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T13:07:13
# authored
title: Add Control-Plane Placement Stall Observability And Regression Coverage
type: fix
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 4
started_at: 2026-04-22T12:56:59
completed_at: 2026-04-22T13:07:13
---

# Add Control-Plane Placement Stall Observability And Regression Coverage

## Summary

Add explicit control-plane observability and regression coverage for placement
repair, alias canonicalization, timeout isolation, and degraded cluster
readiness so this failure mode becomes diagnosable and stays fixed.

## Acceptance Criteria

<!-- verify: true, SRS-07:start, proof: ac-1.log-->
- [x] [SRS-07/AC-01] Hosted control-plane logs or counters expose placement repair, alias rewrite, timeout isolation, and degraded readiness events with enough machine/node detail to debug rollout stalls. <!-- [SRS-07/AC-01] verify: sh -lc 'cargo test -p port-runtime registration_reconcile_ -- --nocapture --test-threads=1 && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture --test-threads=1 && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture --test-threads=1', proof: ac-2.log-->
<!-- verify: true, SRS-07:end, proof: ac-3.log-->
<!-- verify: true, SRS-07:start -->
- [x] [SRS-07/AC-02] Regression coverage proves missing placement fallback, alias repair, machine-list timeout isolation, and degraded cluster-status behavior. <!-- [SRS-07/AC-02] verify: sh -lc 'cargo test -p port-runtime control_plane_service_status_uses_live_route_without_stored_placement -- --nocapture --test-threads=1 && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture --test-threads=1 && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture --test-threads=1 && cargo test -p port-runtime hosted_k3s_cluster_access_reports_kubeconfig_handoff_separately -- --nocapture --test-threads=1' -->
<!-- verify: true, SRS-07:end -->
<!-- verify: true, SRS-NFR-02:start -->
- [x] [SRS-NFR-02/AC-03] Regression coverage fails if hosted request paths reintroduce control-plane self-calls or synchronous placement writes on read. <!-- [SRS-NFR-02/AC-03] verify: sh -lc 'cargo test -p port-runtime effective_recovery_wedge_uses_live_node_service_status_without_self_calling_control_plane -- --nocapture --test-threads=1 && cargo test -p port-runtime machine_status_read_does_not_persist_machine_placement_on_read -- --nocapture --test-threads=1' -->
<!-- verify: true, SRS-NFR-02:end -->
