# VOYAGE REPORT: Execute Hosted Services And Sandboxes

## Voyage Metadata
- **ID:** 1vzRnO000
- **Epic:** 1vz4Yn000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Managed Service Execution Contract
- **ID:** 1vzRov000
- **Status:** done

#### Summary
Define the shared contract for hosted service and sandbox execution so the
runtime, guest agent, CLI, and SDK all target one canonical lifecycle model.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port defines the managed service execution contract, route vocabulary, and runtime-state model without adding hosted-only service verbs or a second runtime surface. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRov000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] The contract keeps one canonical `port service` vocabulary across local and hosted lanes and makes the no-hosted-only-verb boundary explicit before implementation begins. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRov000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzRov000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzRov000/EVIDENCE/ac-2.log)

### Implement Guest-Agent Managed Process Supervisor
- **ID:** 1vzRpC000
- **Status:** done

#### Summary
Extend the guest agent with a managed-process supervisor that can launch,
inspect, and stop service and sandbox commands with stable runtime metadata.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The guest agent can start, list/status, and stop managed service or sandbox processes while preserving the existing guest transport and non-service operations. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpC000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Managed processes capture operator-visible runtime metadata, including live state, exit status, and log paths, while keeping injected secret values out of status responses and logs. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpC000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzRpC000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzRpC000/EVIDENCE/ac-2.log)

### Route Hosted Service Lifecycle Through Live Runtime
- **ID:** 1vzRpF000
- **Status:** done

#### Summary
Turn hosted `port service apply|list|status|stop` into live execution through
the control plane, node agent, and guest runtime instead of desired-state-only
storage.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The hosted runtime materializes stored machine secrets into launched guest processes, persists node-owned runtime state, and reports operator-safe log and exit metadata without surfacing raw secret values. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpF000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted `port service apply`, `list`, `status`, and `stop` execute and report real service or sandbox lifecycle state through the canonical CLI, SDK, and live hosted route instead of only mutating stored desired state. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpF000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzRpF000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzRpF000/EVIDENCE/ac-2.log)

### Publish Hosted Service And Sandbox Workflow
- **ID:** 1vzRpI000
- **Status:** done

#### Summary
Publish the hosted service and sandbox execution workflow so operators can
discover, learn, and verify what now runs live versus what still remains
follow-on work.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] CLI help, README, hosted docs, and SDK docs explain the hosted service and sandbox execution workflow through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpI000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Published proof and operator messaging make the boundary explicit between shipped hosted execution and still-planned work such as restart policy, hardened secret backends, and scheduler policy. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpI000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzRpI000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzRpI000/EVIDENCE/ac-2.log)


