# VOYAGE REPORT: Define Real-HA Control Plane Placement Truth

## Voyage Metadata
- **ID:** VGafx2cmq
- **Epic:** VGYFpfUph
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Require Honest Real-HA Topology Admission For Hosted AWS PVM
- **ID:** VGafyU6FW
- **Status:** done

#### Summary
Define the admission boundary for real HA on hosted AWS PVM so Port only treats
clusters as HA-capable when the current topology and scheduler contract can
actually spread the control plane across distinct execution hosts.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Hosted AWS PVM K3s configs that try to claim real HA without at least three control-plane machines and `control_plane_scheduler = "spread"` are rejected or classified as non-HA. Verified by `cargo test -q -p port-model hosted_k3s -- --nocapture` in `EVIDENCE/ac-1.model.log`, which covers the new topology posture classification for two- and three-control-plane clusters. <!-- verify: command, SRS-01:start:end, proof: ac-1.model.log -->
- [x] [SRS-01/AC-02] Hosted admission fails with explicit host-group and candidate-node detail when distinct execution hosts are unavailable for the requested control-plane spread. Verified by `cargo test -q -p port-runtime hosted_k3s_spread_scheduler -- --nocapture` in `EVIDENCE/ac-2.runtime.log`. <!-- verify: command, SRS-01:start:end, proof: ac-2.runtime.log -->
- [x] [SRS-NFR-02/AC-03] Port does not silently reuse an occupied execution host and still present the cluster as HA. Verified by `cargo test -q -p port-runtime hosted_k3s_spread_scheduler -- --nocapture` in `EVIDENCE/ac-2.runtime.log`, which exercises the occupied-host rejection path for spread scheduling. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.runtime.log -->

#### Verified Evidence
- [ac-1.model.log](../../../../stories/VGafyU6FW/EVIDENCE/ac-1.model.log)
- [ac-2.runtime.log](../../../../stories/VGafyU6FW/EVIDENCE/ac-2.runtime.log)
- [ac-3.fmt.log](../../../../stories/VGafyU6FW/EVIDENCE/ac-3.fmt.log)

### Surface Control-Plane Host Spread In Cluster Status
- **ID:** VGafyUVFV
- **Status:** done

#### Summary
Expose execution-host spread and HA satisfaction in cluster-facing output so an
operator can see whether the hosted AWS PVM control plane is truly multi-host
or only shaped like HA on paper.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Hosted placement state or lifecycle reports record which execution host each control-plane machine occupies. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-1.runtime.log`, which asserts the runtime report captures control-plane placement entries, and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, which renders the placement lines in `port cluster status`. <!-- verify: command, SRS-02:start:end, proof: ac-1.runtime.log, ac-3.cli.log -->
- [x] [SRS-03/AC-02] `port cluster status` or equivalent cluster-facing output reports whether the current control plane satisfies the real-HA spread contract instead of inferring HA from machine count alone. Verified by `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-2.runtime.log` and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, which prints `real-ha status: non-ha-topology` for the two-control-plane hosted cluster. <!-- verify: command, SRS-03:start:end, proof: ac-2.runtime.log, ac-3.cli.log -->
- [x] [SRS-NFR-01/AC-03] The rendered HA truth remains explicitly scoped to hosted AWS `x86_64` PVM rather than broadening to generic hosted HA language. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-1.runtime.log` and by `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-3.cli.log`, both of which assert the detail text stays pinned to `Hosted AWS x86_64 PVM real-HA status ...`. <!-- verify: command, SRS-NFR-01:start:end, proof: ac-1.runtime.log, ac-3.cli.log -->

#### Verified Evidence
- [ac-1.runtime.log](../../../../stories/VGafyUVFV/EVIDENCE/ac-1.runtime.log)
- [ac-2.runtime.log](../../../../stories/VGafyUVFV/EVIDENCE/ac-2.runtime.log)
- [ac-3.cli.log](../../../../stories/VGafyUVFV/EVIDENCE/ac-3.cli.log)
- [ac-4.fmt.log](../../../../stories/VGafyUVFV/EVIDENCE/ac-4.fmt.log)


