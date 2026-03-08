# Executable Pvm And Avf Lanes - Product Requirements

> Port now has enough control-plane, artifact, and guest-agent foundation to stop
treating executable PVM and AVF lanes as distant futures. The next meaningful
step is to split them into two delivery programs:

- an x86_64 Firecracker/PVM host-kit and hosted-launch program for
  cost-controlled Linux fleets
- a first real Apple Virtualization Framework runtime for macOS operators

That matters because the user objective is broader than the finished board:
Port still lacks a real hosted launch path on prepared Linux nodes and still
lacks a first-class executable macOS substrate.

## Problem Statement

The current board exhausted the first PVM and hosted-control slices, but the
product goal is still incomplete. Port can now:

- model x86_64 PVM host-kit requirements
- gate hosted PVM placement by node readiness
- document AVF as a first-class planned lane

Port still cannot:

- build or ship the x86_64 PVM host kit itself
- launch hosted Linux VMs through node agents on prepared hosts
- run a real AVF-backed `machine launch` and guest workflow on macOS

We need the next executable split, not more placeholder scope.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship the first executable x86_64 Firecracker/PVM lane on prepared Linux nodes | Canonical CLI and hosted-launch proof | Initial prepared-node rollout |
| GOAL-02 | Ship the first executable AVF lane on macOS | Canonical CLI and guest attach proof | Initial local AVF rollout |
| GOAL-03 | Preserve one operator model across substrates | No substrate-specific command family | Ongoing |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Linux Fleet Operator | Owns prepared Linux execution hosts or hosted node pools | Protected-launch workflow with explicit admission and runtime ownership |
| macOS Operator | Wants a native local Port lane on macOS | First-class local lifecycle and guest workflow without Linux indirection |
| Platform Engineer | Maintains shared automation and deployment workflows | Stable CLI/model semantics across substrates |

## Scope

### In Scope

- [SCOPE-01] x86_64 Firecracker/PVM host-kit packaging, validation, and
  prepared-node launch.
- [SCOPE-02] Hosted lifecycle ownership for prepared-node PVM launch on Linux.
- [SCOPE-03] AVF local runtime delivery on macOS through the canonical Port
  command model.

### Out of Scope

- [SCOPE-04] arm64 Firecracker/PVM execution.
- [SCOPE-05] Unrelated hypervisor work outside Firecracker/PVM and AVF.
- [SCOPE-06] Provider-wide hosted rollout beyond prepared-node execution
  slices.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must launch x86_64 Firecracker/PVM workloads on prepared Linux nodes through the canonical model, CLI, and hosted runtime ownership. | GOAL-01, GOAL-03 | must | This is the highest-priority cost-control lane. |
| FR-02 | Port must launch AVF-backed Linux guests on macOS through the canonical model, CLI, and guest protocol. | GOAL-02, GOAL-03 | must | This is the first-class macOS execution lane. |
| FR-03 | Port must keep one discoverable command model across local Linux, hosted Linux, and macOS substrates. | GOAL-03 | must | Substrate delivery should not fragment operator behavior. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for each delivered substrate workflow. | GOAL-01, GOAL-02, GOAL-03 | must | Keeps delivery safe and auditable during rollout. |
| NFR-02 | Preserve explicit unsupported boundaries for arm64 Firecracker/PVM and any unimplemented hosted provider paths. | GOAL-03 | must | Prevents overclaiming while the new lanes land incrementally. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Native arm64 standard virtualization is still the practical near-term arm64 cost-control story | AVF and Linux planning might need adjustment | Re-check public arm64 hosting evidence during AVF voyage |
| Prepared Linux nodes can be modeled without redesigning the hosted control split | Hosted PVM launch scope may expand | Validate during the first PVM voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much host-kit packaging must Port own directly versus consume from external build systems? | Product/Runtime | Open |
| Whether AVF implementation should stay pure Rust or use a helper boundary | Runtime | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [x] The research ends with a concrete keep/split recommendation for x86_64
- [x] The outcome names the next epics or voyages needed to resume execution
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

If Port keeps piling substrate work into one generic backlog, it will delay
both the primary Linux cost-control objective and the macOS first-class
objective. Splitting now costs some planning time but prevents the next voyages
from being incoherent.

### Dependencies

- The x86_64 PVM program depends on Port's existing hosted placement, node
  agent, and artifact seams.
- The AVF program depends on the existing shared machine and guest model, not
  on Linux PVM host-kit delivery.

### Alternatives Considered

Alternatives considered:

- Fold AVF under the Linux PVM program:
  rejected because the host platform, runtime owner, and operator proofs are
  materially different.
- Promote arm64 Firecracker/PVM into immediate execution:
  rejected because the current evidence still supports research-only status.
- Stop at documentation and admission gating:
  rejected because the board is empty but the user objective is not complete.

---

*This PRD was seeded from bearing `1vzJKE000`. See `bearings/1vzJKE000/` for original research.*
