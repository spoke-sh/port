# AWS PVM Production Documentation Contract - Product Requirements

## Problem Statement

Port's documentation mentions hosted AWS and the PVM lane across README, foundational docs, focused guides, and the public docs site, but the production narrative is fragmented, duplicative, and still mixed with repo-proof language instead of one clear operator contract.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Converge Port's foundational documentation on one obvious AWS deployment narrative. | Root docs point readers to one canonical AWS/PVM path instead of repeating fragmented or conflicting guidance. | One coherent root doc map |
| GOAL-02 | Make AWS x86_64 hosted Firecracker/PVM the clearest production-oriented cloud narrative Port publishes today. | The docs distinguish the AWS PVM production lane from the standard hosted demo lane and from repo-local proof mechanics. | One explicit AWS PVM operator contract |
| GOAL-03 | Keep shipped boundaries honest while improving fidelity. | The docs remain explicit about missing provider automation, arm64 limits, and non-AWS boundaries. | No accidental over-claims |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | The engineer evaluating whether Port can carry a real AWS production deployment. | One trustworthy path that explains prerequisites, lifecycle, and boundaries for the AWS PVM lane. |
| Product Maintainer | The maintainer keeping Port's root docs, focused guides, and public docs aligned. | Fewer overlapping narratives and a clearer canonical place to update the AWS story. |

## Scope

### In Scope

- [SCOPE-01] Root documentation updates in `README.md`, `ARCHITECTURE.md`, and `CONFIGURATION.md`.
- [SCOPE-02] Focused product guides for hosted, cloud, and PVM behavior.
- [SCOPE-03] Public Docusaurus AWS narrative updates when needed to match the root contract.

### Out of Scope

- [SCOPE-04] Runtime, scheduler, placement, or protocol implementation changes.
- [SCOPE-05] New AWS automation for provisioning, IAM, networking, or downstream GitOps workflows.
- [SCOPE-06] Expanding the production claim to GCP, Azure, or arm64 PVM.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Root docs must direct readers to one canonical AWS production narrative instead of scattering the contract across multiple overlapping summaries. | GOAL-01 | must | Operators should not need to reverse-engineer the deployment story from duplicated docs. |
| FR-02 | The docs must explain the AWS x86_64 hosted Firecracker/PVM lane as the strongest production-oriented Port narrative, including host kit, artifact kit, `prepare-pvm-node`, and canonical machine lifecycle verbs. | GOAL-02 | must | This is the clearest current production story and should read like one contract. |
| FR-03 | The docs must distinguish the AWS PVM lane from the hosted standard lane and from repo-local proof harnesses without hiding those surfaces. | GOAL-01, GOAL-02 | must | Readers need clarity about what is production-oriented versus what is merely proof-backed or demo-only. |
| FR-04 | Failure boundaries must stay explicit, including missing host kit, missing PVM artifacts, arm64 research-only status, and no silent inheritance to GCP or Azure. | GOAL-03 | must | Higher-fidelity docs that overstate scope would be worse than the current fragmented state. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The documentation set must become easier to navigate by reducing repeated AWS/PVM explanations and strengthening cross-links to a canonical path. | GOAL-01 | must | Simplification is part of the requested outcome, not just more text. |
| NFR-02 | Public and root docs must stay aligned on the same AWS production posture. | GOAL-02, GOAL-03 | must | A split between repo docs and public docs would preserve the current fidelity problem. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Root documentation | Manual review of the updated foundational docs | Story evidence citing the converged AWS/PVM sections |
| Focused guides | Manual review of hosted, cloud, and PVM docs | Story evidence showing one canonical AWS production path |
| Public narrative alignment | Manual review of the Docusaurus AWS docs | Story evidence showing the same AWS PVM posture in public docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| AWS x86_64 hosted Firecracker/PVM is the clearest production-oriented Port narrative today. | The docs would optimize the wrong cloud path. | Keep the mission yield rule explicit if product direction changes. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Public docs may still underweight the AWS PVM lane compared with the older hosted standard narrative. | Epic owner | Active risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Foundational docs point to one clear AWS production narrative.
- [ ] AWS x86_64 hosted PVM reads like one canonical Port contract rather than scattered proof notes.
- [ ] Boundaries for standard hosted, non-AWS providers, and arm64 remain explicit and honest.
<!-- END SUCCESS_CRITERIA -->
