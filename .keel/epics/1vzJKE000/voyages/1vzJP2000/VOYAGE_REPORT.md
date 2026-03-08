# VOYAGE REPORT: Prepared Linux Pvm Runtime

## Voyage Metadata
- **ID:** 1vzJP2000
- **Epic:** 1vzJKE000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Publish Prepared Pvm Operator Workflow
- **ID:** 1vzJQJ000
- **Status:** done

#### Summary
Publish the prepared-node PVM workflow across CLI help, README, and PVM docs
once the executable runtime path is live, including proof commands and failure
boundaries.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] CLI help, README, and `docs/pvm.md` describe the prepared-node PVM workflow, prerequisites, and failure boundaries through the canonical `port` command model. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQJ000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Recorded CLI evidence demonstrates prepared-node PVM launch while also proving the preserved standard Firecracker lane for a new operator. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQJ000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJQJ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJQJ000/EVIDENCE/ac-2.log)

### Define Prepared Pvm Host Kit Contract
- **ID:** 1vzJQg000
- **Status:** done

#### Summary
Define the canonical prepared-node PVM host-kit contract so Port can tell the
difference between a merely admission-ready node and a node that can actually
launch x86_64 Firecracker/PVM workloads.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port model, doctor, and runtime preflight define the prepared-node x86_64 PVM host-kit inputs explicitly, including patched Firecracker binary selection and required host prerequisites. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQg000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Missing or malformed prepared-node PVM host-kit state fails with explicit host-kit detail instead of generic runtime launch errors. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQg000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzJQg000: Share PVM Host-Kit Contracts Across Local And Hosted Lanes**
  - Insight: Modeling only hosted PVM state (`planned` or `ready`) is not enough; the hosted node inventory must carry the same host-kit contract shape as the local lane so doctor, placement, and later node-agent launch can reuse one source of truth.
  - Suggested Action: Add new PVM launch or placement work against the shared `PvmHostKit` contract first, then build runtime behavior on top of it.
  - Applies To: `crates/port-model`, `crates/port-runtime`, hosted inventory and doctor surfaces
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJQg000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJQg000/EVIDENCE/ac-2.log)

### Route Hosted Launch Through Prepared Pvm Nodes
- **ID:** 1vzJQh000
- **Status:** done

#### Summary
Replace hosted PVM provider guidance with a live control-plane and node-agent
launch path once a machine has been admitted onto a prepared x86_64 PVM node.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port machine launch` routes admission-ready PVM machines through the live control-plane and prepared-node node-agent path instead of stopping at provider guidance. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQh000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted PVM launch failures surface explicit placement or host-kit causes without regressing existing hosted standard-machine workflows. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQh000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJQh000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJQh000/EVIDENCE/ac-2.log)

### Implement Node Agent Pvm Launch Path
- **ID:** 1vzJSi000
- **Status:** done

#### Summary
Extend the node-agent runtime so prepared Linux hosts can actually launch
x86_64 Firecracker/PVM machines while keeping Port's canonical runtime
manifests and guest attach behavior intact.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The node agent launches x86_64 PVM machines on prepared Linux hosts using the prepared host-kit contract, canonical artifact selection, and canonical runtime metadata layout. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJSi000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Automated and CLI proof keep the standard Firecracker lane executable while the prepared-node PVM launch path lands. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJSi000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzJSi000: Localize Hosted Node Launch Down To One Machine And Host**
  - Insight: Rewriting only the host connection to `Local` is not enough. The localized config must be narrowed to the target machine and host, and hosted-only inventory must be removed, or config validation will fail on stale hosted references before the launch path runs.
  - Suggested Action: Keep node-agent launch localization as a deliberate scope-reduction step before calling shared local runtime helpers.
  - Applies To: `crates/port-runtime/src/hosted_control_plane.rs`, hosted-to-local runtime adaptation paths
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJSi000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJSi000/EVIDENCE/ac-2.log)


