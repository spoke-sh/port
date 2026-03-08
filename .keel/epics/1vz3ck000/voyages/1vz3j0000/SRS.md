# Substrate Drivers And Host Kits - Software Requirements Specification

> Define and sequence the first implementation slices for substrate drivers, hosted node-agent runtime ownership, x86_64 PVM host kits, and AVF execution.

**Epic:** [1vz3ck000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Define the substrate driver boundary that local Firecracker,
  future AVF, and future hosted/node-agent execution will share.
- [SCOPE-02] Define the first hosted inventory and lifecycle contract that can
  sit above local runtime roots and later above node agents.
- [SCOPE-03] Define the x86_64 PVM host-kit and artifact-kit contract, keeping
  arm64 Firecracker/PVM explicitly research-only.
- [SCOPE-04] Define the AVF macOS execution contract, guest transport mapping,
  and operator workflow.
- [SCOPE-05] Keep CLI help, docs, and board evidence aligned with those new
  execution semantics.
- Out of scope: a fully shipped x86_64 PVM runtime, a fully shipped AVF driver,
  a production hosted control plane, and any arm64 Firecracker/PVM claim.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current guest protocol can remain canonical while substrate drivers and hosted node agents change the transport or ownership beneath it. | Assumption | Port may need a second guest API instead of one shared protocol surface. |
| x86_64 PVM requires prepared host components and artifact variants rather than only model flags. | Dependency | Planning would under-scope the real work and create false promises. |
| AVF will require a transport and launch adapter that differs from Firecracker/vsock but can still map to the same CLI verbs. | Assumption | macOS could require a divergent operator model. |
| The current local Firecracker lane must remain functional while the runtime is decomposed. | Constraint | A driver refactor that regresses the shipped lane would be unacceptable. |

## Constraints

- Keep one canonical CLI and one canonical guest protocol.
- Preserve the current local Firecracker lane while introducing new abstractions.
- Fail fast on unsupported substrate, architecture, and protection-mode claims.
- Keep x86_64 PVM in planned implementation scope and arm64 Firecracker/PVM in
  research-only scope.
- Do not allow AVF or hosted control to remain documentation-only placeholders
  after this voyage; they need concrete next stories.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a substrate driver interface that isolates launch, status, stop, and guest-attach behavior from Firecracker-specific runtime code. | SCOPE-01, SCOPE-05 | FR-01 | design review + automated test |
| SRS-02 | Port must define the first hosted inventory and lifecycle contract that preserves current CLI verbs while allowing local or node-agent-backed machine ownership. | SCOPE-01, SCOPE-02, SCOPE-05 | FR-02 | design review + doc review |
| SRS-03 | Port must define the x86_64 PVM host-kit and artifact-kit contract, including host-kernel, VMM, artifact-variant, validation, and explicit operator prerequisites. | SCOPE-03, SCOPE-05 | FR-03 | doc review + contract inspection |
| SRS-04 | Port must define the AVF execution contract for macOS, including launch ownership, guest transport mapping, and operator workflow boundaries. | SCOPE-04, SCOPE-05 | FR-04 | doc review + design review |
| SRS-05 | Port must create implementation-ready stories that sequence substrate-driver extraction, hosted inventory ownership, x86_64 PVM host-kit follow-on work, and AVF execution work. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | FR-05 | board review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported substrate, architecture, and protection-mode combinations must remain explicit in the model, docs, and voyage outputs. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | doc review + inspection |
| SRS-NFR-02 | The voyage must preserve one canonical CLI and one canonical guest protocol across all proposed lanes. | SCOPE-01, SCOPE-02, SCOPE-04, SCOPE-05 | NFR-03 | design review + doc review |
| SRS-NFR-03 | The voyage must yield concrete verification techniques for planning-heavy stories, using tests, CLI proofs, and docs review rather than placeholder acceptance criteria. | SCOPE-05 | NFR-02 | board review + doctor |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
