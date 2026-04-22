---
# system-managed
id: VHXXzkVPt
status: done
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T12:22:25
# authored
title: Split Hosted Cluster Readiness From Kubeconfig Handoff
type: feat
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 3
started_at: 2026-04-22T11:54:19
completed_at: 2026-04-22T12:22:25
---

# Split Hosted Cluster Readiness From Kubeconfig Handoff

## Summary

Refactor hosted cluster readiness so machine/runtime, API, node visibility, and
kubeconfig handoff are reported as separate gates. That keeps `cluster status`
truthful and bounded even when kubeconfig retrieval is the only failing step.

## Acceptance Criteria

<!-- verify: true, SRS-05:start, proof: ac-1.log-->
- [x] [SRS-05/AC-01] `cluster status` returns structured readiness detail for machine/runtime, API visibility, node visibility, and kubeconfig availability instead of collapsing those states into one opaque hosted failure. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime hosted_k3s_cluster_access_ -- --nocapture --test-threads=1, proof: ac-2.log-->
<!-- verify: true, SRS-05:end, proof: ac-3.log-->
<!-- verify: true, SRS-06:start -->
- [x] [SRS-06/AC-02] `cluster kubeconfig` fails only on the kubeconfig handoff boundary and preserves already-established machine/API readiness detail rather than reusing the generic `cluster status` failure path. <!-- [SRS-06/AC-02] verify: cargo test -p port-runtime hosted_k3s_cluster_kubeconfig_ -- --nocapture --test-threads=1, proof: ac-5.log-->
<!-- verify: true, SRS-06:end -->
<!-- verify: true, SRS-NFR-03:start -->
- [x] [SRS-NFR-03/AC-03] The richer readiness fidelity remains on the canonical `port cluster status` and `port cluster kubeconfig` surfaces without introducing a second operator workflow. <!-- [SRS-NFR-03/AC-03] verify: sh -lc 'cargo test -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_kubeconfig_failure_preserves_hosted_readiness_detail -- --nocapture --test-threads=1', proof: ac-8.log-->
<!-- verify: true, SRS-NFR-03:end -->
