# VOYAGE REPORT: Define Stable Endpoint Handoff And Failover Proof

## Voyage Metadata
- **ID:** VGafx2vn4
- **Epic:** VGYFpfmpi
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Model Stable HA Endpoint Handoff In Cluster Output
- **ID:** VGafyUpFb
- **Status:** done

#### Summary
Make the stable HA API endpoint explicit in Port's cluster handoff surfaces so
downstream consumers receive one canonical `api_endpoint` contract instead of a
guest-specific address that drifts during failover.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port cluster up`, `port cluster status`, and `port cluster kubeconfig` hand off the configured `api_endpoint` as the stable cluster address for eligible hosted AWS PVM HA clusters. Verified by `cargo test -q -p port-runtime hosted_k3s_bootstrap_and_join_workflow -- --nocapture` in `EVIDENCE/ac-1.bootstrap.log`, `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-3.ha-eligible.log`, and `cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms -- --nocapture` in `EVIDENCE/ac-4.cli-lifecycle.log`. <!-- verify: command, SRS-01:start:end, proof: ac-1.bootstrap.log, ac-3.ha-eligible.log, ac-4.cli-lifecycle.log -->
- [x] [SRS-02/AC-02] Cluster-facing output reports stable-endpoint HA posture and missing failover prerequisites explicitly. Verified by `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-2.access.log`, `cargo test -q -p port-runtime hosted_k3s_ha_status_reports_spread_satisfied_across_three_hosts -- --nocapture` in `EVIDENCE/ac-3.ha-eligible.log`, and `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture` in `EVIDENCE/ac-5.cli-status.log`. <!-- verify: command, SRS-02:start:end, proof: ac-2.access.log, ac-3.ha-eligible.log, ac-5.cli-status.log -->
- [x] [SRS-NFR-02/AC-03] Port does not claim a stable HA endpoint when the flow still depends on manual downstream rewrites or one control-plane guest address. Verified by `cargo test -q -p port-runtime hosted_k3s_bootstrap_and_join_workflow -- --nocapture` in `EVIDENCE/ac-1.bootstrap.log`, `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract -- --nocapture` in `EVIDENCE/ac-2.access.log`, and the negative-path CLI proofs in `EVIDENCE/ac-4.cli-lifecycle.log` and `EVIDENCE/ac-5.cli-status.log`, all of which assert `stable-endpoint posture: manual-rewrite-required` until hosted AWS PVM real HA is actually satisfied. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-1.bootstrap.log, ac-2.access.log, ac-4.cli-lifecycle.log, ac-5.cli-status.log -->

#### Verified Evidence
- [ac-1.bootstrap.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-1.bootstrap.log)
- [ac-2.access.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-2.access.log)
- [ac-3.ha-eligible.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-3.ha-eligible.log)
- [ac-4.cli-lifecycle.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-4.cli-lifecycle.log)
- [ac-5.cli-status.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-5.cli-status.log)
- [ac-6.fmt.log](../../../../stories/VGafyUpFb/EVIDENCE/ac-6.fmt.log)

### Capture Hosted AWS PVM Failover Proof For The Stable Endpoint
- **ID:** VGafyVDGA
- **Status:** done

#### Summary
Capture one human-reviewable failover proof for the hosted AWS PVM HA endpoint
so Port's first real-HA claim is backed by executable evidence rather than a
documentation promise.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] The canonical proof workflow shows the stable endpoint working before and after one supported control-plane host-loss or guest-replacement scenario on hosted AWS PVM. <!-- verify: command, SRS-03:start:end -->
- [x] [SRS-NFR-01/AC-02] The failover proof is stored as a human-reviewable Port proof artifact rather than as chat-only notes. <!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-02/AC-03] The proof or its paired negative-path evidence makes missing failover prerequisites explicit instead of implying stability that Port cannot yet provide. <!-- verify: command, SRS-NFR-02:start:end -->

#### Verified Evidence
![ac-1.gif](../../../../stories/VGafyVDGA/EVIDENCE/ac-1.gif)
- [ac-1.log](../../../../stories/VGafyVDGA/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGafyVDGA/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGafyVDGA/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGafyVDGA/EVIDENCE/ac-4.log)
- [hosted-k3s-ha-failover-workflow.cast](../../../../stories/VGafyVDGA/EVIDENCE/hosted-k3s-ha-failover-workflow.cast)


