# VOYAGE REPORT: Foundation And Hosted Cloud Hypervisor Lane

## Voyage Metadata
- **ID:** 1vzdKx000
- **Epic:** 1vzdKB000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Define Cloud Hypervisor Contract And Doctor Checks
- **ID:** 1vzdMW000
- **Status:** done

#### Summary
Define the executable Cloud Hypervisor machine, artifact, and doctor contract so
Port can distinguish the new substrate cleanly from Firecracker and surface the
host requirements before launch.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model`, sample config, and `port doctor` represent Cloud Hypervisor as an executable `standard` substrate with explicit artifact selection and no implicit Firecracker fallback. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model && cargo test -q -p port-runtime resolve_artifact_metadata_reports_missing_selected_cloud_hypervisor_variant_without_fallback && cargo test -q -p port-cli && cargo run -q -p port-cli -- --config examples/port.toml doctor | rg "machine:demo-ch|cloud-hypervisor:demo-ch"', proof: ac-1.log -->
- [x] [SRS-01/AC-02] Unsupported Cloud Hypervisor host, architecture, or protection-mode combinations fail fast with substrate-specific diagnostics. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime doctor_report_surfaces_cloud_hypervisor_platform_and_binary_checks && cargo test -q -p port-runtime doctor_report_fails_fast_for_unsupported_cloud_hypervisor_platform_and_protection_mode', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdMW000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdMW000/EVIDENCE/ac-2.log)

### Implement Local Cloud Hypervisor Machine Driver
- **ID:** 1vzdMZ000
- **Status:** done

#### Summary
Implement the local Cloud Hypervisor launch, status, and stop path through
Port's machine-driver seam, including runtime manifest ownership and console
capture.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port machine launch|status|stop` executes a Cloud Hypervisor machine locally through the canonical driver boundary and records coherent runtime metadata. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime cloud_hypervisor_launch_status_and_stop_write_canonical_runtime_state && cargo test -q -p port-cli --test machine_commands cli_machine_launch_status_and_stop_route_cloud_hypervisor_locally', proof: ac-1.log -->
- [x] [SRS-02/AC-02] Local Cloud Hypervisor preflight failures identify the missing host prerequisite or runtime boundary instead of generic unsupported-host output. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime cloud_hypervisor_launch_surfaces_missing_binary_preflight && cargo test -q -p port-cli --test machine_commands cli_machine_launch_surfaces_missing_cloud_hypervisor_binary', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdMZ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdMZ000/EVIDENCE/ac-2.log)

### Route Hosted Cloud Hypervisor Lifecycle
- **ID:** 1vzdMa000
- **Status:** done

#### Summary
Route hosted placement, launch, status, stop, and guest attach through
Cloud Hypervisor-capable nodes without any Firecracker-specific assumptions in
the control-plane or node-agent path.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Hosted control-plane and node-agent flows can place, launch, inspect, and stop a Cloud Hypervisor machine through the canonical machine routes. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_status_stop_route_through_live_control_plane && cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_status_and_stop_round_trip', proof: ac-1.log -->
- [x] [SRS-04/AC-02] Hosted Cloud Hypervisor failures report rejected-node or runtime context instead of silently falling back to Firecracker assumptions. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback && cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdMa000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdMa000/EVIDENCE/ac-2.log)

### Publish Cloud Hypervisor Operator Workflow
- **ID:** 1vzdMb000
- **Status:** done

#### Summary
Publish the canonical local and hosted Cloud Hypervisor workflow through
README, cloud/operator docs, examples, and CLI help so the substrate is
discoverable and learnable from Port itself.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] README, `docs/cloud.md`, `docs/operators.md`, examples, and CLI help publish one coherent Cloud Hypervisor workflow with explicit local and hosted proof commands. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Cloud Hypervisor|cloud-hypervisor" README.md docs/cloud.md docs/operators.md examples/port.toml crates/port-cli/src/lib.rs', proof: ac-1.log -->
- [x] [SRS-05/AC-02] The published operator surface keeps unsupported boundaries explicit and points at concrete verification evidence for the shipped Cloud Hypervisor lane. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli -- --test-threads=1', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdMb000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdMb000/EVIDENCE/ac-2.log)

### Bridge Cloud Hypervisor Guest Sessions
- **ID:** 1vzdMs000
- **Status:** done

#### Summary
Bridge Cloud Hypervisor guest transport onto Port's shared guest protocol so
guest exec, copy, pty, logs, and forward work without a substrate-specific API.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Cloud Hypervisor machines expose guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical Port guest protocol. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime guest_exec_uses_cloud_hypervisor_vsock_tunnel_when_runtime_socket_is_absent && cargo test -q -p port-runtime hosted_guest_exec_routes_cloud_hypervisor_machine_through_node_runtime_root && cargo test -q -p port-cli --test guest_commands cli_cloud_hypervisor_guest_commands_cover_all_capabilities && cargo test -q -p port-cli --test guest_commands cli_guest_commands_cover_hosted_cloud_hypervisor_runtime', proof: ac-1.log -->
- [x] [SRS-03/AC-02] The Cloud Hypervisor guest path reuses the existing protocol and hosted route families rather than inventing a second substrate-specific guest API. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-sdk && cargo test -q -p port-runtime hosted_guest_exec_routes_cloud_hypervisor_machine_through_node_runtime_root && cargo test -q -p port-cli --test guest_commands cli_guest_commands_cover_hosted_cloud_hypervisor_runtime', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdMs000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdMs000/EVIDENCE/ac-2.log)


