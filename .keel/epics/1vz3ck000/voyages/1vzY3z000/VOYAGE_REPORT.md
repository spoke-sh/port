# VOYAGE REPORT: PVM Host Kit And Artifact Delivery

## Voyage Metadata
- **ID:** 1vzY3z000
- **Epic:** 1vz3ck000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Pvm Host Kit Package Contract
- **ID:** 1vzY51000
- **Status:** done

#### Summary
Define the canonical `x86_64` Firecracker/PVM host-kit package contract in the
model, runtime, and hosted inventory surfaces so Port can describe prepared PVM
nodes as real package-bearing capacity rather than only "planned" metadata.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port defines a canonical PVM host-kit package contract that binds patched Firecracker identity, prepared host-kernel metadata, and required boot-line expectations for `x86_64` Firecracker/PVM nodes. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model sample_config_derives_hosted_node_pvm_host_kit_package_identity && cargo test -q -p port-runtime doctor_report_includes_hosted_pvm_host_kit_contract_checks', proof: ac-1.log -->
- [x] [SRS-01/AC-02] Unsupported or incomplete PVM host-kit combinations fail fast with explicit diagnostics and no standard-lane fallback in doctor or placement summaries. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model ready_hosted_pvm_lane_requires_host_kit_package_identity && cargo test -q -p port-runtime doctor_report_fails_fast_for_missing_pvm_boot_arg_and_binary', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzY51000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzY51000/EVIDENCE/ac-2.log)

### Add Pvm Artifact Mobility Workflow
- **ID:** 1vzY52000
- **Status:** done

#### Summary
Extend the canonical `port artifacts ...` surface so `x86_64/firecracker/pvm`
kernel and guest-image variants can be built, validated, pushed, and pulled
without implicit reuse of the standard Firecracker lane.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port artifacts build|validate|push|pull` supports the `x86_64/firecracker/pvm` kernel and guest-image variants through the canonical artifact model. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli cli_artifact_build_and_validate_selected_pvm_kernel_variant -- --exact && cargo test -q -p port-cli cli_artifact_build_and_validate_selected_pvm_guest_image_variant -- --exact && cargo test -q -p port-cli cli_artifact_push_and_pull_round_trip_pvm_variant_contract_for_kernel_and_guest_image -- --exact', proof: ac-1.log -->
- [x] [SRS-02/AC-02] PVM artifact mobility remains deterministic and explicit: missing variants fail with the selected variant name and Port does not fall back to standard Firecracker artifacts. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime tests::resolve_artifact_metadata_distinguishes_standard_and_pvm_paths -- --exact && cargo test -q -p port-runtime tests::resolve_guest_image_metadata_distinguishes_standard_and_pvm_paths -- --exact && cargo test -q -p port-runtime tests::resolve_artifact_metadata_reports_missing_selected_pvm_variant_without_fallback -- --exact', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzY52000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzY52000/EVIDENCE/ac-2.log)

### Implement Hosted Pvm Node Preparation
- **ID:** 1vzY6F000
- **Status:** done

#### Summary
Implement the hosted node-preparation/import flow that upgrades a node from
planned PVM capacity to ready PVM capacity when a complete host-kit package is
attached through canonical hosted inventory and node-agent state.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Port can prepare or import a hosted node with a complete PVM host-kit package so hosted inventory records a ready `x86_64` Firecracker/PVM node instead of only planned capacity. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime && cargo test -q -p port-cli --test machine_commands cli_control_plane_prepare_pvm_node_enables_generic_hosted_pvm_launch -- --exact', proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted placement and doctor output distinguish ready PVM nodes from planned or incomplete nodes with node-specific remediation guidance. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime doctor_report_uses_imported_prepared_hosted_pvm_state -- --exact && cargo test -q -p port-cli --test machine_commands cli_control_plane_prepare_pvm_node_enables_generic_hosted_pvm_launch -- --exact', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzY6F000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzY6F000/EVIDENCE/ac-2.log)

### Publish Pvm Host Kit Operator Workflow
- **ID:** 1vzY6J000
- **Status:** done

#### Summary
Publish the canonical operator workflow for PVM host-kit packaging, artifact
mobility, hosted node preparation, and proof so the lane is discoverable
through README, PVM docs, hosted docs, and CLI help.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README, `docs/pvm.md`, `docs/hosted.md`, and CLI help publish the canonical `x86_64` Firecracker/PVM host-kit and artifact workflow, including the explicit `aarch64` research-only boundary. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzY6J000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] The published workflow includes repo-local proof commands for artifact build/validate/push/pull plus hosted node preparation/import so operators can verify the lane without hidden steps. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzY6J000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzY6J000: Use Absolute Verify Scripts For Board Checks**
  - Insight: Absolute-path `bash` verify scripts are more reliable than inline shell expressions or relative script paths under the current `keel` shell execution environment
  - Suggested Action: Prefer `.keel/stories/<id>/verify-ac-*.sh` plus absolute paths in acceptance comments for new stories
  - Applies To: `.keel/stories/*/README.md`, `.keel/stories/*/verify-ac-*.sh`
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzY6J000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzY6J000/EVIDENCE/ac-2.log)


