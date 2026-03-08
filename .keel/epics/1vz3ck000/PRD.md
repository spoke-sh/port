# PVM And Multi-Substrate Execution - Product Requirements

> Turn Port's current "modeled but mostly local Firecracker" posture into a
productized execution architecture that can support x86_64 PVM host kits,
first-class AVF on macOS, and hosted node-agent lifecycle ownership without
forking the CLI or guest protocol.

## Problem Statement

The current Port board is empty again, but the user's objective is not. Port is
still materially behind Slicer in three areas that are tightly coupled but not
identical:

- cloud cost control when nested virtualization or `/dev/kvm` is unavailable,
- first-class non-Firecracker substrates such as Apple Virtualization
  Framework, and
- a hosted daemon/control-plane architecture that can preserve today's CLI and
  guest protocol while moving lifecycle ownership away from the local process.

The research for this epic is complete. The product requirement now is to turn
those findings into implementable slices:

- keep x86_64 Firecracker/PVM as a strategic lane,
- keep AVF as a first-class macOS lane,
- keep arm64 Firecracker/PVM research-only until stronger evidence exists, and
- restructure Port so local Firecracker ownership is one driver, not the only
  runtime architecture.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make Port substrate-aware at the runtime seam | Firecracker-specific runtime logic is isolated behind a driver boundary | First substrate driver voyage complete |
| GOAL-02 | Establish a productizable hosted lifecycle model | Local and hosted lifecycle ownership share one CLI model and one guest protocol | Hosted node-agent and inventory contract planned and partially scaffolded |
| GOAL-03 | Create a credible x86_64 PVM lane | Host-kit, artifact-kit, and validation requirements are explicit and sequenceable | First PVM host-kit voyage planned with executable follow-on stories |
| GOAL-04 | Keep macOS first-class through AVF | AVF has an explicit implementation plan, operator contract, and guest transport mapping | First AVF voyage planned and documented |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Engineer | Builds and operates execution capacity across local and hosted environments | One coherent CLI and lifecycle model across substrates |
| Hosted Product Runtime Team | Owns control plane and node-agent behavior | A runtime boundary that can move ownership away from the local CLI process |
| macOS Operator | Wants a first-class local Port lane on Apple hardware | A real AVF-backed workflow, not Linux-only documentation |
| Cost-Conscious Cloud Operator | Needs microVM isolation without assuming nested virtualization everywhere | A credible x86_64 PVM path and explicit arm64 boundaries |

## Scope

### In Scope

- [SCOPE-01] substrate driver boundaries in runtime, model, and CLI
- [SCOPE-02] hosted node-agent lifecycle and inventory foundations
- [SCOPE-03] x86_64 PVM host-kit and artifact-kit planning plus early scaffolding
- [SCOPE-04] AVF driver and operator-lane planning
- [SCOPE-05] CLI/doc evolution needed to keep local and hosted semantics coherent

### Out of Scope

- [SCOPE-06] a fully shipped x86_64 PVM runtime in one epic
- [SCOPE-07] an arm64 Firecracker/PVM implementation claim
- [SCOPE-08] production auth, scheduler, or host-group rollout
- [SCOPE-09] unrelated refactors that do not advance the execution architecture

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define a substrate driver boundary for machine lifecycle and guest attachment so Firecracker local execution becomes one driver rather than the only runtime architecture. | GOAL-01, GOAL-02 | must | This is the minimum architectural shift required for AVF, hosted control, and future hypervisors. |
| FR-02 | Port must define and partially scaffold hosted node-agent lifecycle ownership and inventory/state contracts on top of the substrate driver boundary. | GOAL-02 | must | Hosted Port cannot be built on local runtime roots alone. |
| FR-03 | Port must define the x86_64 PVM host-kit and artifact-kit contract, including host kernel, VMM, artifact variants, validation, and explicit operator boundaries. | GOAL-03 | must | PVM is strategically important, but only if treated as a real host preparation lane. |
| FR-04 | Port must define the AVF macOS driver lane and map the canonical guest and lifecycle semantics onto AVF-specific transport and operator workflows. | GOAL-04 | must | macOS remains a first-class requirement and cannot stay a Linux-only footnote. |
| FR-05 | Port's CLI and docs must stay canonical across local, hosted, and future substrate-specific lanes. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The product requirement is one operator surface, not several substrate-specific tools. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Unsupported substrate, architecture, and protection-mode combinations must fail fast with explicit diagnostics and no silent fallback. | GOAL-01, GOAL-03, GOAL-04 | must | Execution boundaries are central to trust in the product. |
| NFR-02 | Host-kit and artifact-kit contracts must stay deterministic and reproducible across local, hosted, and future execution lanes. | GOAL-02, GOAL-03 | must | PVM and hosted operation depend on prepared, validated host and artifact inputs. |
| NFR-03 | The epic must preserve one canonical CLI model and one canonical guest protocol across all planned lanes. | GOAL-01, GOAL-02, GOAL-04 | must | Port should not fragment into substrate-specific user experiences. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use Rust tests for model/runtime boundaries and driver-facing behavior.
- Use CLI proofs and help-text inspection for operator-surface changes.
- Use docs review and artifact/host-kit contract inspection for planning-heavy
  slices.
- Use recorded evidence and reflections for every story and voyage.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing guest protocol remains the right canonical guest-operation surface across local, hosted, Firecracker, and AVF lanes. | A second guest API would create a migration tax. | Test and inspect guest-operation mapping in each voyage. |
| x86_64 PVM remains strategically important even if it requires custom host components. | PVM planning effort could outweigh operator value. | Validate host-kit scope early and keep arm64 claims out of shipping promises. |
| AVF can preserve the current CLI semantics with a substrate-specific transport adapter. | macOS could require a divergent command model. | Plan AVF transport and lifecycle mapping explicitly before implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How quickly can Firecracker-local runtime logic be extracted without destabilizing the shipped Linux lane? | Runtime | Open |
| Does x86_64 PVM require Port-owned kernel/VMM packaging from day one? | Runtime / Artifacts | Open |
| What is the cleanest guest transport mapping for AVF and hosted node agents? | Runtime | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The board contains active voyages and stories that sequence substrate drivers, hosted lifecycle ownership, x86_64 PVM host kits, and AVF work instead of leaving them as modeled future scope.
- [ ] Local Firecracker runtime logic is being refactored behind a substrate-aware boundary without regressing the current Linux lane.
- [ ] Port's docs and CLI can explain x86_64 PVM keep, arm64 PVM research-only scope, and AVF keep with implementation-backed evidence.
<!-- END SUCCESS_CRITERIA -->


---

*This PRD was seeded from bearing `1vz3ck000`. See `bearings/1vz3ck000/` for original research.*
