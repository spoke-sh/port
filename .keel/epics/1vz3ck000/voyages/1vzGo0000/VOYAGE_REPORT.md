# VOYAGE REPORT: X86 64 PVM Host Kit Foundation

## Voyage Metadata
- **ID:** 1vzGo0000
- **Epic:** 1vz3ck000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Model X86 64 PVM Host Kit Contract
- **ID:** 1vzGrP000
- **Status:** done

#### Summary
Define the explicit shared-model contract for the `x86_64/firecracker/pvm`
host kit and keep the `aarch64` PVM boundary research-only across model
rendering and example configuration.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` defines an implementation-ready `x86_64/firecracker/pvm` host-kit contract that captures the prepared-host boundary, boot-line expectation (`pti=off`), and patched Firecracker binary requirement. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-model, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Example config and rendered model-facing output keep `x86_64` as the only planned PVM implementation lane and mark `aarch64/firecracker/pvm` as research-only with no silent compatibility claim. <!-- [SRS-01/AC-02] verify: cargo test -q -p port-model && cargo test -q -p port-cli, proof: ac-2.log -->

#### Implementation Insights
- **1w03v0000: Prefer Serializable Contract Fields Over Orphan Helper Types**
  - Insight: A standalone contract type is not enough for operator-facing work. If the contract matters to CLI discovery or future runtime checks, it should live on the serializable model surface so examples, validation, and later stories all reuse the same source of truth.
  - Suggested Action: When adding future substrate or host-kit contracts, first attach them to the config/model structs and round-trip them through the checked-in example before building doctor or runtime logic on top.
  - Applies To: `crates/port-model/src/lib.rs`, `examples/*.toml`, future substrate contracts
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzGrP000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzGrP000/EVIDENCE/ac-2.log)

### Publish PVM Operator Proof Workflow
- **ID:** 1vzGrQ000
- **Status:** done

#### Summary
Publish the operator-facing PVM workflow in README/docs/help text and back it
with repository-local proof scripts that show the x86_64 keep decision, arm64
research-only boundary, and current host-kit/artifact validation flow.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README, `docs/pvm.md`, and CLI help explain the x86_64 PVM keep decision, the required host-kit and artifact-kit prerequisites, and the `aarch64` research-only boundary. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrQ000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Repository-local proof scripts and recorded evidence demonstrate the documented foundation workflow without regressing the existing standard Firecracker operator path. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrQ000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w03x0000: Order Operator Proofs So The Log Tells The Story**
  - Insight: Proof scripts should be ordered so the log explains the workflow clearly from top to bottom. Rebuilding prerequisite artifacts before a diagnostic step can make the evidence materially easier to review without changing the underlying behavior.
  - Suggested Action: When writing workflow proofs, read the first screenful of the resulting log and reorder the commands until that excerpt communicates the intended operator outcome.
  - Applies To: `.keel/stories/*/verify-ac-*.sh`, operator workflow evidence
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzGrQ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzGrQ000/EVIDENCE/ac-2.log)

### Add PVM Doctor Host Kit Checks
- **ID:** 1vzGrd000
- **Status:** done

#### Summary
Extend `port doctor` so the `x86_64/firecracker/pvm` lane reports explicit
host-kit readiness and blocking diagnostics instead of blurring unsupported
hosts into the standard Firecracker path.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port doctor` reports the x86_64 Firecracker/PVM host-kit check with explicit pass/fail diagnostics for platform, architecture, boot-line, and PVM Firecracker binary readiness. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrd000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Unsupported architecture, missing `pti=off`, and missing patched-binary states fail fast with clear messages and no fallback to the standard Firecracker lane. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrd000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w03vv000: Add Probe Seams Before Expanding Host Diagnostics**
  - Insight: A small probe struct is enough to turn environment-dependent diagnostics into a testable seam. That is lower-cost and more maintainable than trying to mock shell commands or `/proc` access ad hoc in each test.
  - Suggested Action: When extending `doctor` with more host or platform checks, first introduce or reuse a single fact-gathering struct and keep the decision logic pure over that struct.
  - Applies To: `crates/port-runtime/src/lib.rs`, future doctor and platform readiness checks
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzGrd000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzGrd000/EVIDENCE/ac-2.log)

### Materialize PVM Artifact Variants
- **ID:** 1vzGrf000
- **Status:** done

#### Summary
Add dedicated `x86_64/firecracker/pvm` kernel and guest-image build/validate
variants so artifact commands can materialize the PVM lane without reusing the
standard Firecracker artifacts.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port artifacts build|validate` supports `x86_64/firecracker/pvm` kernel and guest-image variants through dedicated scripts or contracts, and fails immediately when the PVM variant is missing. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrf000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] PVM artifact selection remains deterministic and separate from the standard Firecracker lane in code, output paths, and validation behavior. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrf000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w03w0000: Infer Artifact Variants From Canonical Output Paths**
  - Insight: Deriving selector intent from the canonical artifact path keeps model selection, cache/store layout, and script behavior aligned. That lets new variants like `x86_64/firecracker/pvm` land without widening the script API.
  - Suggested Action: When adding future artifact selectors, keep the path layout canonical and let the scripts derive selector intent from it before introducing new script arguments or hidden environment variables.
  - Applies To: `scripts/artifacts/*.sh`, `crates/port-runtime/src/lib.rs`, artifact selector evolution
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzGrf000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzGrf000/EVIDENCE/ac-2.log)


