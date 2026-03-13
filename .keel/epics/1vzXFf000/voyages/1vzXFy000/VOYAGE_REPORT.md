# VOYAGE REPORT: Enable Hosted Standard Cloud Launch

## Voyage Metadata
- **ID:** 1vzXFy000
- **Epic:** 1vzXFf000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Hosted Standard Placement Contract
- **ID:** 1vzXG2000
- **Status:** done

#### Summary
Define the placement and routing contract for `standard` provider-backed hosted
machines so `cloud-generic`, `cloud-aws`, and `cloud-gcp` resolve onto explicit
registered nodes with actionable rejection detail instead of generic remote
unsupported-host guidance.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Placement summary logic resolves candidate hosted nodes for `cloud-generic`, `cloud-aws`, and `cloud-gcp` while preserving machine, host, provider, and control-plane identity. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model -p port-runtime hosted_standard', proof: ac-2.log -->
- [x] [SRS-01/AC-02] Ineligible, unregistered, or unresolved standard-lane nodes fail with explicit routing context instead of the current generic “run Port on that host directly” provider guidance. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard', proof: ac-2.log -->
- [x] [SRS-01/AC-03] The hosted placement contract serializes candidate-node and selected-node detail so later status or stop routes can follow the same provider-aware placement, satisfying `SRS-NFR-01`. <!-- [SRS-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzXG2000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzXG2000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzXG2000/EVIDENCE/ac-3.log)

### Route Standard Cloud Launch Through Hosted Runtime
- **ID:** 1vzXIF000
- **Status:** done

#### Summary
Route the sample `generic-linux`, `aws`, and `gcp` standard Firecracker lanes
through the live hosted control-plane and node-agent runtime so canonical
`machine launch|status|stop` operate on registered remote nodes instead of
failing fast with provider guidance.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port machine launch --machine cloud-generic`, `cloud-aws`, and `cloud-gcp` route through the hosted control plane and selected node agent, and the selected node owns the runtime root for the launched machine. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_launch && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip', proof: ac-2.log -->
- [x] [SRS-02/AC-02] Hosted standard-lane launch failures surface machine, host, provider, control plane, and selected-node detail without falling back to the local launch path, satisfying `SRS-NFR-01` and `SRS-NFR-02`. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_launch', proof: ac-2.log -->
- [x] [SRS-03/AC-01] `port machine status` and `port machine stop` work for hosted standard-lane machines using stored placement, and their output includes provider plus hosted-node routing detail. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_status_stop && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzXIF000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzXIF000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzXIF000/EVIDENCE/ac-3.log)

### Publish Hosted Standard Cloud Workflow
- **ID:** 1vzXIG000
- **Status:** done

#### Summary
Publish the shipped hosted standard-lane cloud workflow through README, cloud
docs, hosted docs, and CLI help so operators can discover and execute the
`cloud-generic`, `cloud-aws`, and `cloud-gcp` demo lane without stale denial
guidance.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README, `docs/cloud.md`, `docs/hosted.md`, and relevant CLI help publish the hosted standard cloud workflow for the sample `generic-linux`, `aws`, and `gcp` machines. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/verify-hosted-standard-docs.sh', proof: ac-1.log -->
- [x] [SRS-04/AC-02] The published workflow includes executable repo-local proof and removes stale guidance that tells operators to run Port directly on the provider host for these shipped demo lanes. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip && ! grep -nE \"run Port on the AWS Linux host itself|run Port on the GCP Linux host itself|run Port on that Linux host directly\" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzXIG000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzXIG000/EVIDENCE/ac-2.log)


