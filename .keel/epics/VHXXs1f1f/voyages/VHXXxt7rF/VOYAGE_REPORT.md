# VOYAGE REPORT: Recover Hosted Placement Truth Without Read-Path Stall

## Voyage Metadata
- **ID:** VHXXxt7rF
- **Epic:** VHXXs1f1f
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Make Hosted Machine And Service Status Live-First Under Placement Drift
- **ID:** VHXXzjYOd
- **Status:** done

#### Summary
Replace stored-placement-first request handling for hosted machine and service
status with live-first resolution so the control plane reports runtime truth
instead of `malformed` when node-agent paths are still healthy.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `list_machines` and `machine_status` return live runtime truth when stored placement is missing or stale but a live node-agent route can still resolve the machine. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime control_plane_machine_status_falls_back_when_stored_placement_is_unusable -- --nocapture', proof: ac-2.log -->
- [x] [SRS-02/AC-02] Hosted service status and guest-route resolution prefer live placement or candidate-node resolution before failing on missing stored placement, and return explicit degraded detail when live refresh cannot succeed. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime control_plane_service_status_uses_live_route_without_stored_placement -- --nocapture && cargo test -p port-runtime control_plane_guest_exec_prefers_stored_placement_over_candidate_order -- --nocapture', proof: ac-5.log -->
- [x] [SRS-NFR-01/AC-03] Hosted machine/service status fan-out uses bounded per-machine deadlines so one bad route degrades partial truth instead of wedging the full response. <!-- [SRS-NFR-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture', proof: ac-8.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHXXzjYOd/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHXXzjYOd/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHXXzjYOd/EVIDENCE/ac-3.log)
- [ac-5.log](../../../../stories/VHXXzjYOd/EVIDENCE/ac-5.log)
- [ac-8.log](../../../../stories/VHXXzjYOd/EVIDENCE/ac-8.log)

### Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background
- **ID:** VHXXzjuOa
- **Status:** done

#### Summary
Move hosted placement repair out of synchronous read handlers and into explicit
reconcile hooks so placement persistence becomes deterministic, canonical, and
separate from operator truth surfaces.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted request handlers stop persisting placement state on read paths; placement repair is triggered only by startup, registration, or lifecycle hooks. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime registration_reconcile_ -- --nocapture && cargo test -p port-runtime proxy_bytes_does_not_purge_stale_machine_placement_when_guest_socket_is_missing -- --nocapture', proof: ac-2.log -->
- [x] [SRS-04/AC-02] The placement reconciler canonicalizes legacy node aliases to configured node identities and persists repaired machine placement without requiring a user read path. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture', proof: ac-5.log -->
- [x] [SRS-NFR-02/AC-03] No hosted request handler reintroduces control-plane self-recursion or synchronous write-on-read repair while reconciling placement. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime effective_recovery_wedge_uses_live_node_service_status_without_self_calling_control_plane -- --nocapture && cargo test -p port-runtime proxy_bytes_does_not_purge_stale_machine_placement_when_guest_socket_is_missing -- --nocapture', proof: ac-8.log -->

#### Verified Evidence
- [ac-2.log](../../../../stories/VHXXzjuOa/EVIDENCE/ac-2.log)
- [ac-5.log](../../../../stories/VHXXzjuOa/EVIDENCE/ac-5.log)
- [ac-8.log](../../../../stories/VHXXzjuOa/EVIDENCE/ac-8.log)

### Split Hosted Cluster Readiness From Kubeconfig Handoff
- **ID:** VHXXzkVPt
- **Status:** done

#### Summary
Refactor hosted cluster readiness so machine/runtime, API, node visibility, and
kubeconfig handoff are reported as separate gates. That keeps `cluster status`
truthful and bounded even when kubeconfig retrieval is the only failing step.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] `cluster status` returns structured readiness detail for machine/runtime, API visibility, node visibility, and kubeconfig availability instead of collapsing those states into one opaque hosted failure. <!-- [SRS-05/AC-01] verify: cargo test -p port-runtime hosted_k3s_cluster_access_ -- --nocapture --test-threads=1, proof: ac-2.log-->
- [x] [SRS-06/AC-02] `cluster kubeconfig` fails only on the kubeconfig handoff boundary and preserves already-established machine/API readiness detail rather than reusing the generic `cluster status` failure path. <!-- [SRS-06/AC-02] verify: cargo test -p port-runtime hosted_k3s_cluster_kubeconfig_ -- --nocapture --test-threads=1, proof: ac-5.log-->
- [x] [SRS-NFR-03/AC-03] The richer readiness fidelity remains on the canonical `port cluster status` and `port cluster kubeconfig` surfaces without introducing a second operator workflow. <!-- [SRS-NFR-03/AC-03] verify: sh -lc 'cargo test -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift -- --nocapture --test-threads=1 && cargo test -p port --test machine_commands cli_cluster_kubeconfig_failure_preserves_hosted_readiness_detail -- --nocapture --test-threads=1', proof: ac-8.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHXXzkVPt/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHXXzkVPt/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHXXzkVPt/EVIDENCE/ac-3.log)
- [ac-5.log](../../../../stories/VHXXzkVPt/EVIDENCE/ac-5.log)
- [ac-8.log](../../../../stories/VHXXzkVPt/EVIDENCE/ac-8.log)

### Add Control-Plane Placement Stall Observability And Regression Coverage
- **ID:** VHXXzkwR0
- **Status:** done

#### Summary
Add explicit control-plane observability and regression coverage for placement
repair, alias canonicalization, timeout isolation, and degraded cluster
readiness so this failure mode becomes diagnosable and stays fixed.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] Hosted control-plane logs or counters expose placement repair, alias rewrite, timeout isolation, and degraded readiness events with enough machine/node detail to debug rollout stalls. <!-- [SRS-07/AC-01] verify: sh -lc 'cargo test -p port-runtime registration_reconcile_ -- --nocapture --test-threads=1 && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture --test-threads=1 && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture --test-threads=1', proof: ac-2.log-->
- [x] [SRS-07/AC-02] Regression coverage proves missing placement fallback, alias repair, machine-list timeout isolation, and degraded cluster-status behavior. <!-- [SRS-07/AC-02] verify: sh -lc 'cargo test -p port-runtime control_plane_service_status_uses_live_route_without_stored_placement -- --nocapture --test-threads=1 && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture --test-threads=1 && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture --test-threads=1 && cargo test -p port-runtime hosted_k3s_cluster_access_reports_kubeconfig_handoff_separately -- --nocapture --test-threads=1' -->
- [x] [SRS-NFR-02/AC-03] Regression coverage fails if hosted request paths reintroduce control-plane self-calls or synchronous placement writes on read. <!-- [SRS-NFR-02/AC-03] verify: sh -lc 'cargo test -p port-runtime effective_recovery_wedge_uses_live_node_service_status_without_self_calling_control_plane -- --nocapture --test-threads=1 && cargo test -p port-runtime machine_status_read_does_not_persist_machine_placement_on_read -- --nocapture --test-threads=1' -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHXXzkwR0/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHXXzkwR0/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHXXzkwR0/EVIDENCE/ac-3.log)


