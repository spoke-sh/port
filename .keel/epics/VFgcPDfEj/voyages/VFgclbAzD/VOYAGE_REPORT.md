# VOYAGE REPORT: AWS PVM Host Kit Preparation

## Voyage Metadata
- **ID:** VFgclbAzD
- **Epic:** VFgcPDfEj
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Define AWS PVM Prepared Host Contract
- **ID:** VFgcoUMUb
- **Status:** done

#### Summary
Define the implementation-ready AWS hosted PVM prepared-host contract for
`cloud-aws`, keeping the lane explicitly tied to x86_64 AWS Linux, custom
kernel and boot requirements, patched `firecracker-pvm`, and dedicated PVM
artifacts without generic-node substitution.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port publishes an explicit `cloud-aws` x86_64 prepared-host contract that captures the required host kit, `pti=off`, patched `firecracker-pvm`, and PVM artifact prerequisites. <!-- [SRS-01/AC-01] verify: manual, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-02] The contract keeps the scope boundary explicit: x86_64 AWS hosted PVM only, with no arm64 or non-AWS support claim. <!-- [SRS-NFR-01/AC-02] verify: manual, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFgcoUMUb/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFgcoUMUb/EVIDENCE/ac-2.log)
- [ac-4.log](../../../../stories/VFgcoUMUb/EVIDENCE/ac-4.log)

### Implement AWS Node Preparation Workflow
- **ID:** VFgcoUWUa
- **Status:** done

#### Summary
Implement the canonical preparation/import workflow that moves an eligible AWS
node into the hosted PVM-ready state for `cloud-aws` and exposes the resulting
readiness through operator-visible Port surfaces.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port control-plane prepare-pvm-node` prepares or imports AWS hosted PVM readiness for an x86_64 AWS node without manual config overlays or hand-edited imported inventory. <!-- [SRS-02/AC-01] verify: manual, proof: ac-2.log-->
- [x] [SRS-03/AC-02] Doctor, status, or imported-readiness surfaces explain missing or stale AWS host-kit prerequisites with `cloud-aws` guidance and no standard-lane ambiguity. <!-- [SRS-03/AC-02] verify: manual, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFgcoUWUa/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFgcoUWUa/EVIDENCE/ac-2.log)
- [ac-4.log](../../../../stories/VFgcoUWUa/EVIDENCE/ac-4.log)


