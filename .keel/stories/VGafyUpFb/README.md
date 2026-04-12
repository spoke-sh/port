---
# system-managed
id: VGafyUpFb
status: done
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T09:21:22
# authored
title: Model Stable HA Endpoint Handoff In Cluster Output
type: feat
operator-signal:
scope: VGYFpfmpi/VGafx2vn4
index: 1
started_at: 2026-04-12T09:11:27
completed_at: 2026-04-12T09:21:22
---

# Model Stable HA Endpoint Handoff In Cluster Output

## Summary

Make the stable HA API endpoint explicit in Port's cluster handoff surfaces so
downstream consumers receive one canonical `api_endpoint` contract instead of a
guest-specific address that drifts during failover.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `port cluster up`, `port cluster status`, and `port cluster kubeconfig` hand off the configured `api_endpoint` as the stable cluster address for eligible hosted AWS PVM HA clusters. Verified by `cargo test -q -p port-runtime hosted_k3s_bootstrap_and_join_workflow -- --nocapture` in `EVIDENCE/ac-1.bootstrap.log`, `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-3.ha-eligible.log`, and `cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms -- --nocapture` in `EVIDENCE/ac-4.cli-lifecycle.log`. <!-- verify: command, SRS-01:start:end, proof: ac-1.bootstrap.log, ac-3.ha-eligible.log, ac-4.cli-lifecycle.log -->
- [x] [SRS-02/AC-02] Cluster-facing output reports stable-endpoint HA posture and missing failover prerequisites explicitly. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-2.access.log`, `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-3.ha-eligible.log`, and `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-5.cli-status.log`. <!-- verify: command, SRS-02:start:end, proof: ac-2.access.log, ac-3.ha-eligible.log, ac-5.cli-status.log -->
- [x] [SRS-NFR-02/AC-03] Port does not claim a stable HA endpoint when the flow still depends on manual downstream rewrites or one control-plane guest address. Verified by `cargo test -q -p port-runtime hosted_k3s_bootstrap_and_join_workflow -- --nocapture` in `EVIDENCE/ac-1.bootstrap.log`, `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-2.access.log`, and the negative-path CLI proofs in `EVIDENCE/ac-4.cli-lifecycle.log` and `EVIDENCE/ac-5.cli-status.log`, all of which assert `stable-endpoint posture: manual-rewrite-required` until hosted AWS PVM real HA is actually satisfied. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-1.bootstrap.log, ac-2.access.log, ac-4.cli-lifecycle.log, ac-5.cli-status.log -->
