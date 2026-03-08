# VOYAGE REPORT: Hosted API And Inventory

## Voyage Metadata
- **ID:** 1vz4cU000
- **Epic:** 1vz4Yn000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Define Hosted Auth And API Contract
- **ID:** 1vz4gb000
- **Status:** done

#### Summary
Define the first hosted control-plane endpoint and token-auth contract so Port
can model authenticated hosted targets without inventing a second operator
surface.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port publishes implementation-ready hosted endpoint and token-auth contracts in the shared model, including how hosted API identity maps onto the canonical CLI target surface. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gb000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] README, hosted docs, and CLI help describe the hosted auth contract and clearly distinguish the modeled control-plane path from shipped local behavior. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gb000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4gb000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz4gb000/EVIDENCE/ac-2.log)

### Define Hosted Guest Bridge Attach Contract
- **ID:** 1vz4gc000
- **Status:** done

#### Summary
Define the first hosted guest bridge attach contract so later hosted `exec`,
`copy`, `pty`, `logs`, and `forward` operations can reuse the current guest
protocol through control-plane and node-agent brokerage.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Port publishes an implementation-ready hosted guest bridge attach contract that preserves the current guest protocol and names the control-plane and node-agent brokerage path explicitly. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gc000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] README, hosted docs, and CLI help explain how hosted guest operations map onto the same canonical `guest` verbs and what follow-on implementation still remains. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gc000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4gc000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz4gc000/EVIDENCE/ac-2.log)

### Define Hosted Machine Lifecycle Surface
- **ID:** 1vz4h3000
- **Status:** done

#### Summary
Extend the shared machine lifecycle contract so hosted `machine list|status|stop`
surfaces can be represented explicitly while preserving Port's existing CLI
verbs and reporting model.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Port publishes implementation-ready hosted machine summary, status, and stop contracts that preserve the canonical `machine` verbs and make hosted ownership, routing, and status sources explicit. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4h3000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] CLI help and hosted/operator docs explain the hosted lifecycle surface, including what is modeled versus what is already runnable today. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4h3000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4h3000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz4h3000/EVIDENCE/ac-2.log)

### Define Hosted Node Inventory Model
- **ID:** 1vz4hB000
- **Status:** done

#### Summary
Define the first hosted node and host-group inventory contract so Port can map
placement, ownership, and later scheduling work onto shared machine vocabulary.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Port publishes implementation-ready node and host-group contracts in the shared model, including ownership, placement, and capability fields needed for hosted machine lifecycle routing. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4hB000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Hosted docs explain how nodes and host groups relate to later scheduler, monitoring, and services work without implying those features are already shipped. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4hB000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4hB000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz4hB000/EVIDENCE/ac-2.log)

### Sequence Hosted Follow-On Work
- **ID:** 1vz4ih000
- **Status:** done

#### Summary
Sequence the follow-on hosted-control backlog so monitoring, secrets, services,
sandboxes, detached forwarding, Unix-socket forwarding, and SDK work remain
ordered behind the first hosted auth, inventory, lifecycle, and guest-bridge
foundation.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] The hosted-control board records an implementation-ready follow-on sequence for monitoring, secrets, services, sandboxes, detached forwarding, Unix-socket forwarding, and SDK work after this voyage. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4ih000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] README or hosted-control docs explain that those follow-on capabilities are downstream of the authenticated API, inventory, lifecycle, and guest-bridge foundation rather than already shipped. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4ih000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4ih000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz4ih000/EVIDENCE/ac-2.log)


