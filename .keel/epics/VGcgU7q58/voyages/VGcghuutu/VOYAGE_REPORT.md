# VOYAGE REPORT: Expose Hosted Cluster Status Schema

## Voyage Metadata
- **ID:** VGcghuutu
- **Epic:** VGcgU7q58
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Model Hosted Machine And Service Truth In Cluster Status
- **ID:** VGcgt729o
- **Status:** done

#### Summary
Model hosted machine identity, placement, managed-service state, and related
runtime truth inside the canonical hosted cluster status payload.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Hosted machine identity, placement, and managed-service truth are present in one canonical status payload. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_contract', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The canonical payload remains machine-readable enough for downstream consumers to adopt without schema forks. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift', SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-02/AC-03] The canonical payload is exposed through the existing cluster status surface instead of a one-off diagnostic command. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms && cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth', SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcgt729o/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcgt729o/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGcgt729o/EVIDENCE/ac-3.log)

### Report Legacy Detached Runtime Drift In Cluster Status
- **ID:** VGcgt8hAx
- **Status:** done

#### Summary
Teach the hosted cluster status contract to report legacy detached K3s PID/log
drift explicitly so downstream consumers can reject that runtime shape.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Hosted status reports legacy detached-runtime drift when PID/log artifacts appear outside managed-service ownership. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_reports_legacy_detached_runtime_drift && cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-02] The legacy-drift signal does not create a second contradictory hosted truth path. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_cluster_access_contract && cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth', SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcgt8hAx/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcgt8hAx/EVIDENCE/ac-2.log)

### Document Downstream Hosted Status Contract
- **ID:** VGcgtAKBo
- **Status:** done

#### Summary
Author the downstream hosted status contract and its proof posture so paired
infra work can consume Port truth intentionally instead of by reverse
engineering runtime output.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] The downstream hosted status contract is documented with the fields and semantics that consumers may rely on. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] The proof posture for validating the hosted status contract is documented alongside the contract. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcgtAKBo/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcgtAKBo/EVIDENCE/ac-2.log)


