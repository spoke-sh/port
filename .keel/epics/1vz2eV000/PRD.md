# Cloud Substrate And PVM Strategy - Product Requirements

> Expand Port from a local Firecracker MVP into a hosted, substrate-aware
platform with credible cloud cost-control lanes, first-class macOS support, and
the operator/API surfaces needed to approach SlicerVM-level capability.

## Problem Statement

Port's completed MVP deliberately optimized for a narrow local Linux launch
path. That left major product gaps relative to SlicerVM:

- no hosted or remote control plane;
- no machine inventory, status, or stop lifecycle;
- no streamed PTY or log-follow semantics;
- no artifact push, pull, or remote cache story;
- no first-class multi-hypervisor design; and
- no supportable protected-VM lane for cloud cost control.

The user objective has changed. Port now needs a durable strategy for running on
cloud VMs even when nested virtualization is unavailable or too expensive, while
also supporting macOS operators and eventually a hosted Port environment. That
requires substrate-aware planning across model, runtime, CLI, artifacts, and
hosted control surfaces.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Introduce a substrate-aware Port model | Model, docs, and CLI can represent Firecracker KVM, Firecracker PVM, Cloud Hypervisor, and Apple Virtualization Framework lanes without ambiguity | Initial implementation slice shipped |
| GOAL-02 | Establish hosted control-plane foundations | Port has machine inventory and lifecycle surfaces plus a documented API/client split that can span local and hosted environments | Foundation voyage complete |
| GOAL-03 | Define artifact mobility for local and remote operation | Artifact contracts include build, publish, pull, cache, and variant selection semantics | Foundation voyage complete |
| GOAL-04 | Reopen protected virtualization as a first-class lane | Port documents a credible near-term PVM path and begins implementation from the right platform assumptions instead of treating PVM as out of scope | Early PVM voyage in flight |
| GOAL-05 | Make macOS a first-class operator lane | Apple Virtualization Framework is represented as a real Port substrate, not only as documentation telling operators to use Linux elsewhere | AVF design voyage planned |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port locally, remotely, or in a hosted deployment | One coherent CLI and API for machine lifecycle, guest access, and artifact handling |
| Cloud Cost-Conscious Team | Needs microVM isolation on affordable cloud hosts | Execution lanes that do not force bare metal or premium nested-virt instances everywhere |
| Hosted Port Operator | Runs shared Port infrastructure for customers | Durable control-plane ownership, inventory, transport brokering, and policy surfaces |
| macOS Developer | Wants a real local Port experience on Apple hardware | A first-class macOS lane instead of Linux-only workarounds |
| Artifact Producer | Builds and distributes kernels and guest images | Deterministic build plus push, pull, cache, and variant selection flows |

## Scope

### In Scope

- [SCOPE-01] Substrate-aware model and documentation changes across local, hosted, and planned lanes.
- [SCOPE-02] Hosted control-plane foundations, including machine inventory and lifecycle surfaces.
- [SCOPE-03] Artifact mobility and variant-selection contracts for local and remote use.
- [SCOPE-04] Protected-VM planning and implementation slices backed by current upstream and product research.
- [SCOPE-05] First-class Apple Virtualization Framework planning and CLI/model integration.

### Out of Scope

- [SCOPE-06] Shipping full parity with SlicerVM in a single voyage.
- [SCOPE-07] Treating every cloud provider and protected-VM technology as equally mature.
- [SCOPE-08] Hidden substrate-specific operator paths outside the canonical Port surfaces.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must model execution substrates, protection modes, and artifact variants explicitly so operators can understand which lanes are supported, partial, or experimental. | GOAL-01, GOAL-04, GOAL-05 | must | The old provider-only cloud model is too narrow for the new product objective. |
| FR-02 | Port must expose canonical machine inventory and lifecycle surfaces, including list, status, and stop, across local runtimes and future hosted runtimes. | GOAL-02 | must | Operators need durable machine visibility and control before later hosted or scheduler features can make sense. |
| FR-03 | Port must introduce a hosted control-plane contract with a daemon/API/client split that can carry the existing guest protocol and lifecycle operations beyond one local process. | GOAL-02 | must | Hosted and hybrid deployment require a long-lived owner for runtimes and transport brokering. |
| FR-04 | Port must define and begin implementing artifact publish, pull, cache, and variant-selection flows in addition to local build and validate. | GOAL-03 | must | A hosted or remote product cannot depend on host-local artifact paths alone. |
| FR-05 | Port must provide a credible protected-VM lane that matches Slicer's strategic value while staying honest about current architecture and platform limits. | GOAL-04 | must | Cloud cost control is now a primary product objective. |
| FR-06 | Port must treat Apple Virtualization Framework on macOS as a first-class citizen in the model, docs, and CLI evolution. | GOAL-01, GOAL-05 | should | macOS cannot remain a second-class “go use Linux” story. |
| FR-07 | Port's CLI and docs must expand toward Slicer-level discoverability while preserving Port's current command clarity and canonical surface discipline. | GOAL-01, GOAL-02, GOAL-03 | must | New capabilities are not done unless discoverable, learnable, and usable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Unsupported or experimental substrate lanes must fail fast with explicit operator guidance rather than falling back silently. | GOAL-01, GOAL-04, GOAL-05 | must | The product cannot blur proven and speculative execution paths. |
| NFR-02 | Local and hosted control paths must preserve one canonical guest-operation model and avoid divergent operator semantics. | GOAL-02 | must | Port's existing guest protocol coherence is a strength worth preserving. |
| NFR-03 | Artifact selection must remain deterministic across architecture, substrate, and protection-mode variants. | GOAL-03, GOAL-04, GOAL-05 | must | Variant explosion is a major risk once Port supports more than one runtime lane. |
| NFR-04 | The expanded CLI and API surface must remain testable and evidence-friendly through automated tests, command proofs, and documentation. | GOAL-02, GOAL-03 | must | The board needs traceable evidence as the product surface broadens. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove model and runtime behavior through story-level automated tests.
- Record CLI-level evidence for lifecycle, artifact, and guest workflows.
- Use documentation review plus command proofs for discoverability and operator
  guidance.
- Keep `keel doctor`, `cargo test`, and any voyage-specific recordings or
  command transcripts as required evidence.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing Port guest protocol can remain canonical even as local and hosted control paths diverge internally | We may need a more disruptive protocol or client split | Validate during hosted-control design and scaffolding |
| The first practical expansion slice should start with lifecycle/control-plane foundations rather than with a full PVM implementation | The epic could stall on premature platform work | Re-check after the first voyage and first implementation results |
| Apple Virtualization Framework can be represented cleanly as a Port substrate even before full parity exists | The macOS story may need a separate product lane | Validate in the AVF planning voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which protected-VM technology becomes Port's long-term arm64 lane? | Platform | Open |
| How much of the Slicer-style Firecracker PVM stack is portable across clouds? | Platform | Open |
| How quickly should Port add a public API versus focusing on richer local CLI first? | Product/Platform | Open |
| What artifact registry and cache contract best fits hosted Port? | Platform | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [x] The research distinguishes near-term, supportable execution lanes from aspirational ones across Firecracker KVM, Firecracker PVM, Cloud Hypervisor, and Apple Virtualization Framework.
- [x] The research clarifies what is true today about Slicer's PVM lane versus upstream arm64 protected-virtualization work.
- [x] The research yields a concrete recommendation for Port's control-plane, model, and artifact evolution, with at least one immediately plannable voyage.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

The opportunity cost is substantial: broadening Port this way delays polish on
the original narrow local tool. But the user objective has explicitly changed,
and the competitive comparison makes the old scope insufficient. Avoiding the
work would leave Port structurally behind on the exact axes that matter for a
hosted product.

### Dependencies

Port now depends on a sequence of architectural shifts:

- a new substrate-aware model beyond the current provider-only framing;
- a hosted control plane with API/SDK and machine lifecycle ownership;
- richer runtime manifests and inventory/status semantics;
- artifact distribution contracts for local and remote use; and
- dedicated research and prototype slices for protected virtualization and Apple
  Virtualization Framework.

### Alternatives Considered

Alternatives considered:

- Keep PVM scoped out again:
  rejected because cloud cost control is now a primary product objective.
- Treat arm64 as proof that Firecracker PVM is already solved:
  rejected because current public evidence does not support that conclusion.
- Focus only on control-plane ergonomics and ignore substrate work:
  rejected because hosted value depends on having cost-effective execution lanes.
- Focus only on PVM and defer lifecycle/API expansion:
  rejected because Port still needs the control-plane scaffolding that would
  make any new substrate usable as a product.

---

*This PRD was seeded from bearing `1vz2eV000`. See `bearings/1vz2eV000/` for original research.*
