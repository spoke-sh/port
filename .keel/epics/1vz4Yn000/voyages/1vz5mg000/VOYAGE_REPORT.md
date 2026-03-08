# VOYAGE REPORT: Hosted Runtime And Service Expansion

## Voyage Metadata
- **ID:** 1vz5mg000
- **Epic:** 1vz4Yn000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 6/6 stories complete

## Implementation Narrative
### Implement Hosted Control Plane Runtime Path
- **ID:** 1vz5nU000
- **Status:** done

#### Summary
Implement the first authenticated hosted runtime path for canonical `machine
list|status|stop` operations through the control plane and node agent.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Hosted `machine list|status|stop` operations work through the canonical CLI and route through the modeled hosted control-plane and node-agent ownership path. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nU000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Help text, docs, and CLI evidence distinguish hosted runtime behavior from still-planned forwarding, monitoring, secrets, services, sandboxes, and SDK work. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nU000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5nU000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5nU000/EVIDENCE/ac-2.log)

### Implement Hosted Guest Operations Runtime Path
- **ID:** 1vz5nk000
- **Status:** done

#### Summary
Implement the first hosted runtime path for canonical
`guest exec|copy|pty|logs|forward` operations over the existing guest protocol.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Hosted guest operations reuse the canonical `guest` verbs and existing guest protocol frames while routing through control-plane authorization and node-agent guest brokerage. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nk000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Operator docs and CLI evidence explain the hosted guest runtime boundary and leave detached forwarding, Unix-socket forwarding, monitoring, secrets, services, sandboxes, and SDK work as explicit follow-on slices. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nk000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5nk000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5nk000/EVIDENCE/ac-2.log)

### Add Hosted Secrets Services And Sandboxes
- **ID:** 1vz5nl000
- **Status:** done

#### Summary
Add the first hosted secrets, services, and sandboxes surfaces on top of the
hosted runtime, forwarding, and monitoring foundation.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] Port defines and implements coherent hosted secrets, services, and sandboxes surfaces that build on the canonical runtime and guest model rather than bypassing it. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nl000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Operator docs and evidence explain the supported hosted service/sandbox workflows and the remaining SDK or advanced platform work. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nl000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5nl000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5nl000/EVIDENCE/ac-2.log)

### Publish Hosted SDK And API Clients
- **ID:** 1vz5nm000
- **Status:** done

#### Summary
Publish supported hosted SDK and API client surfaces once the hosted runtime and
service verbs stabilize.

#### Acceptance Criteria
- [x] [SRS-06/AC-01] Port publishes a supported SDK and API client surface for hosted machine, guest, and service operations that mirrors the canonical operator model. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nm000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-06/AC-02] README, docs, and examples show the intended SDK/API client entry points and call out any surfaces that remain planned rather than shipped. <!-- [SRS-06/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nm000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5nm000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5nm000/EVIDENCE/ac-2.log)

### Add Hosted Monitoring And Top
- **ID:** 1vz5nx000
- **Status:** done

#### Summary
Add hosted monitoring and `top` surfaces once the hosted runtime and guest
brokerage path exist.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Port exposes hosted monitoring and `top` surfaces through the canonical operator model and grounds them in hosted node ownership and runtime state. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nx000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Docs and CLI evidence explain the monitoring boundary relative to runtime, forwarding, secrets, services, sandboxes, and SDK follow-on work. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nx000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5nx000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5nx000/EVIDENCE/ac-2.log)

### Add Detached And Unix-Socket Forwarding
- **ID:** 1vz5o6000
- **Status:** done

#### Summary
Extend the canonical forwarding surface with detached and Unix-socket modes once
the hosted guest runtime path exists.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `guest forward` supports detached lifecycle management and Unix-socket forwarding without introducing a second forwarding command family. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5o6000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] CLI help, docs, and evidence explain how detached and Unix-socket forwarding relate to the hosted guest runtime path and what remains downstream. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5o6000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz5o6000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz5o6000/EVIDENCE/ac-2.log)


