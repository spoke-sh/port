---
# system-managed
id: VGafyUVFV
status: done
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T09:07:39
# authored
title: Surface Control-Plane Host Spread In Cluster Status
type: feat
operator-signal:
scope: VGYFpfUph/VGafx2cmq
index: 2
started_at: 2026-04-12T08:54:08
completed_at: 2026-04-12T09:07:39
---

# Surface Control-Plane Host Spread In Cluster Status

## Summary

Expose execution-host spread and HA satisfaction in cluster-facing output so an
operator can see whether the hosted AWS PVM control plane is truly multi-host
or only shaped like HA on paper.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Hosted placement state or lifecycle reports record which execution host each control-plane machine occupies. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-1.runtime.log`, which asserts the runtime report captures control-plane placement entries, and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, which renders the placement lines in `port cluster status`. <!-- verify: command, SRS-02:start:end, proof: ac-1.runtime.log, ac-3.cli.log -->
- [x] [SRS-03/AC-02] `port cluster status` or equivalent cluster-facing output reports whether the current control plane satisfies the real-HA spread contract instead of inferring HA from machine count alone. Verified by `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-2.runtime.log` and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, which prints `real-ha status: non-ha-topology` for the two-control-plane hosted cluster. <!-- verify: command, SRS-03:start:end, proof: ac-2.runtime.log, ac-3.cli.log -->
- [x] [SRS-NFR-01/AC-03] The rendered HA truth remains explicitly scoped to hosted AWS `x86_64` PVM rather than broadening to generic hosted HA language. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-1.runtime.log` and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, both of which assert the detail text stays pinned to `Hosted AWS x86_64 PVM real-HA status ...`. <!-- verify: command, SRS-NFR-01:start:end, proof: ac-1.runtime.log, ac-3.cli.log -->
