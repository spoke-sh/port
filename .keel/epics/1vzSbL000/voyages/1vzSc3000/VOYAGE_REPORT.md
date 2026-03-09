# VOYAGE REPORT: Host Groups And Service Placement

## Voyage Metadata
- **ID:** 1vzSc3000
- **Epic:** 1vzSbL000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Host Group And Scheduler Contracts
- **ID:** 1vzSd6000
- **Status:** done

#### Summary
Define the shared host-group and scheduler-policy contracts so hosted Port can
target prepared groups of nodes without inventing a second service or hosted
workflow model.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Shared model, sample config, and hosted inventory/runtime structures define host groups, membership, and scheduler policy for hosted services and sandboxes. <!-- [SRS-01/AC-01] verify: cargo test, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Validation, doctor output, or CLI-facing diagnostics surface missing or invalid host-group scheduler inputs with explicit detail. <!-- [SRS-01/AC-02] verify: cargo test, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSd6000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSd6000/EVIDENCE/ac-2.log)

### Publish Multi-Node Hosted Service Workflow
- **ID:** 1vzSdH000
- **Status:** done

#### Summary
Publish the first multi-node hosted service workflow so operators can discover
how to target a host group through `port service` and understand the limits
that still remain after the first scheduler slice.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] CLI help, README, hosted docs, and proof publish the multi-node hosted service workflow through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzSdH000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Operator messaging makes explicit the remaining limits after this slice, including no autoscaling, broader scheduler policy, or fleet management yet. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzSdH000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSdH000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSdH000/EVIDENCE/ac-2.log)

### Implement Hosted Service Placement Scheduler
- **ID:** 1vzSdV000
- **Status:** done

#### Summary
Implement the first deterministic hosted scheduler slice so `port service
apply` can choose an eligible node from a target host group and record that
placement decision.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Hosted `port service apply --kind service` and `--kind sandbox` select one eligible prepared node from the requested host group and route execution through that node's existing hosted runtime path. <!-- [SRS-02/AC-01] verify: cargo test, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Scheduler selection is deterministic for equal inventory input and returns explicit admission detail when no node qualifies. <!-- [SRS-02/AC-02] verify: cargo test, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSdV000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSdV000/EVIDENCE/ac-2.log)

### Surface Placement State Through Canonical Service Commands
- **ID:** 1vzSdb000
- **Status:** done

#### Summary
Surface selected node, host group, and placement/runtime detail through the
existing `port service list|status|stop` workflow instead of adding a separate
hosted scheduler surface.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port service list`, `status`, and `stop` surface selected node identity, host-group identity, and placement/runtime state through the canonical service output. <!-- [SRS-03/AC-01] verify: cargo test, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Placement failures or stale placement records remain operator-visible through status/output instead of collapsing into generic service errors. <!-- [SRS-03/AC-02] verify: cargo test, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSdb000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSdb000/EVIDENCE/ac-2.log)


