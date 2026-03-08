# VOYAGE REPORT: PVM Runtime Admission And Placement

## Voyage Metadata
- **ID:** 1vzHPo000
- **Epic:** 1vz3ck000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Model Pvm Node Capability Contract
- **ID:** 1vzHRt000
- **Status:** done

#### Summary
Add one canonical x86_64 PVM capability contract that can be resolved from the
local Firecracker lane and from hosted node inventory without implying
`aarch64` support.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` exposes explicit x86_64 PVM capability state for local and hosted execution, and the sample config serializes that state without widening the planned lane beyond `x86_64`. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRt000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Hosted protocol or SDK contracts can carry the same capability state so hosted placement logic can consume it without inventing a second PVM vocabulary. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRt000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w04a0000: Prefer Per-Ac Verify Annotations Over Shared Proof Blocks**
  - Insight: Shared summary-level verify comments can cause proof metadata to drift or cross-wire between acceptance criteria, while one inline verify annotation per AC stays stable.
  - Suggested Action: For multi-AC stories, use repo-rooted `verify-ac-*.sh` scripts and only the per-AC inline verify comment form before recording evidence.
  - Applies To: `.keel/stories/*/README.md`, `.keel/stories/*/verify-ac-*.sh`
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzHRt000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzHRt000/EVIDENCE/ac-2.log)

### Gate Hosted Pvm Placement
- **ID:** 1vzHRx000
- **Status:** done

#### Summary
Extend the hosted control-plane and node-agent path so PVM machines can only be
placed onto nodes that advertise the required x86_64 PVM capability.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted protocol and runtime behavior expose node PVM readiness and use it when resolving machine placement or denial reasons. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRx000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted `port machine launch|status` proofs reject unplaceable PVM machines without regressing standard hosted machine workflows. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRx000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzHRx000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzHRx000/EVIDENCE/ac-2.log)

### Publish Pvm Admission Workflow
- **ID:** 1vzHRy000
- **Status:** done

#### Summary
Document the canonical local and hosted PVM admission workflow so operators can
discover what Port can prove today, what requires a prepared host kit, and what
remains explicitly unsupported.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README, `docs/pvm.md`, sample config, and CLI help explain local and hosted PVM admission, required host-kit prerequisites, and the explicit `aarch64` boundary. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRy000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Repository-local proof commands or scripts demonstrate both the PVM admission path and the preserved standard Firecracker path with recorded evidence. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRy000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzHRy000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzHRy000/EVIDENCE/ac-2.log)

### Select Local Pvm Runtime Inputs
- **ID:** 1vzHSA000
- **Status:** done

#### Summary
Teach the local Firecracker runtime path to select PVM-specific launch inputs
and fail with host-kit-specific diagnostics instead of treating PVM as a vague
future lane.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port-runtime` resolves the PVM-specific Firecracker binary and launch metadata only when the requested machine selects `protection_mode = "pvm"`, while leaving the standard lane unchanged. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHSA000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Local CLI proofs surface host-kit preflight failures as explicit PVM admission errors rather than falling back to the standard Firecracker lane. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHSA000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w04f0000: Split Lane-Specific Binary Selection Into A Pure Helper**
  - Insight: Pulling lane-specific binary selection into a pure helper makes the protection-mode contract testable without depending on a live Firecracker process or host PATH mutations.
  - Suggested Action: When adding more substrate or protection-mode launch inputs, isolate the selection logic in a pure helper before wiring it into the process-spawn path.
  - Applies To: `crates/port-runtime/src/lib.rs`, launch-path selection helpers
  - Category: code


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzHSA000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzHSA000/EVIDENCE/ac-2.log)


