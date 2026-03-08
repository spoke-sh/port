# Cloud Substrate And PVM Strategy - Product Requirements

## Problem Statement

Port's completed MVP proved a narrow local Linux Firecracker workflow. That is
no longer enough for the product objective. Port now needs to become a
substrate-aware orchestration system that can support:

- local Linux operation on Firecracker with KVM
- hosted and remote execution through a real control plane
- cost-sensitive cloud execution with a deliberate PVM lane
- richer operator surfaces closer to SlicerVM
- artifact mobility beyond the local filesystem
- first-class Apple Virtualization Framework support on macOS

The core challenge is architectural, not cosmetic. Port's current runtime,
model, and CLI are tightly aligned to one local Firecracker path. To expand
without fragmenting the product, Port needs one canonical control model, one
canonical guest-operation model, and explicit backend lanes for each supported
substrate.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Introduce a substrate-aware Port architecture. | Port can model backend, protection mode, architecture, and artifact variants without treating cloud provider identity as the only runtime axis. | Shared model, design docs, and initial implementation stories shipped |
| GOAL-02 | Establish a hosted-control foundation. | Port has a concrete node-agent and control-plane design, plus first machine lifecycle surfaces that make local and hosted operation converge instead of diverge. | Machine inventory/status/stop shipped locally; hosted contract published |
| GOAL-03 | Expand Port's operator and artifact surfaces. | Operators can discover richer lifecycle commands and artifact mobility semantics through the canonical CLI and docs. | New CLI/model/docs slices landed with evidence |
| GOAL-04 | Create a credible multi-substrate roadmap. | Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, and Apple Virtualization Framework each have explicit support status, constraints, and planned implementation paths. | Support matrix published; first follow-on voyages ready |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port on local Linux or remote Linux hosts and needs stronger machine lifecycle, status, and control surfaces. | One operator model for local and hosted execution. |
| Cloud Cost-Conscious Operator | Needs microVM isolation on cloud VMs without defaulting to expensive bare metal or nested-virt-only fleets. | A supportable story for KVM, PVM, and substrate selection by cost and availability. |
| macOS Operator | Wants Port to be a first-class local tool on macOS instead of only a remote-Linux wrapper. | A native Apple Virtualization Framework lane with canonical Port commands. |
| Automation Integrator | Embeds Port into a larger workflow or hosted system. | Stable control-plane, artifact, and guest-operation contracts. |

## Scope

### In Scope

- [SCOPE-01] Substrate-aware model and planning for Firecracker/KVM,
  Firecracker/PVM, Cloud Hypervisor, and Apple Virtualization Framework.
- [SCOPE-02] Hosted-control foundations: node-agent/API design, lifecycle
  semantics, and local machine inventory/status/stop as the first executable
  slice.
- [SCOPE-03] Canonical CLI expansion for machine lifecycle and future hosted
  operation.
- [SCOPE-04] Artifact mobility design for build, push, pull, and backend-aware
  variant selection.
- [SCOPE-05] Explicit support matrix and planning for x86_64 PVM, arm64
  protected-virtualization research, and first-class macOS support.

### Out of Scope

- [SCOPE-06] Shipping full production implementations for every substrate in a
  single voyage.
- [SCOPE-07] Claiming generic arm64 protected virtualization support before a
  backend mapping is proven.
- [SCOPE-08] Multi-tenant billing, quotas, or commercial hosted-product policy.
- [SCOPE-09] Backward-compatibility aliases for pre-expansion model or CLI
  shapes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must model execution substrate, hypervisor backend, protection mode, architecture, and artifact compatibility as first-class concepts. | GOAL-01, GOAL-04 | must | Provider identity alone is no longer sufficient to express runtime capability. |
| FR-02 | Port must provide machine lifecycle surfaces for inventory, status, and stop through the canonical CLI and runtime state model. | GOAL-02, GOAL-03 | must | Operators cannot manage local or hosted fleets without lifecycle visibility and control. |
| FR-03 | Port must define a hosted node-agent and control-plane contract that preserves one guest-operation model across local and remote execution. | GOAL-02 | must | Hosted Port should extend the current guest model, not fork it. |
| FR-04 | Port must define artifact mobility and compatibility contracts for local and remote workflows, including push/pull semantics and backend-aware variants. | GOAL-03 | must | Remote execution is not usable if artifacts only exist as local paths. |
| FR-05 | Port must document and partially implement a Firecracker/PVM lane that is honest about its x86_64-first support boundary and operational prerequisites. | GOAL-01, GOAL-04 | must | Cost-controlled cloud operation is now a primary objective, but the lane is specialized. |
| FR-06 | Port must plan first-class Apple Virtualization Framework support on macOS and expose that lane through the same model and CLI vocabulary. | GOAL-01, GOAL-04 | must | macOS is now a product lane, not only an operator-workstation note. |
| FR-07 | Port must retain Firecracker/KVM as the proven baseline while creating explicit seams for alternate backends such as Cloud Hypervisor. | GOAL-01, GOAL-04 | must | Port needs breadth without destabilizing the one lane that already works. |
| FR-08 | Port must publish a clear support matrix for Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, Apple Virtualization Framework, and arm64 protected-virtualization research. | GOAL-04 | must | Operators need to understand what is supported, experimental, or future work. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Port must preserve one canonical CLI and model vocabulary across local and hosted workflows. | GOAL-01, GOAL-02, GOAL-03 | must | Product expansion should not create separate local and remote mental models. |
| NFR-02 | New lifecycle and artifact surfaces must fail fast with substrate-aware, actionable guidance. | GOAL-02, GOAL-03, GOAL-04 | must | Operators need clear reasons when a backend or protection mode is unsupported. |
| NFR-03 | Every new backend or control-plane lane must publish explicit constraints, artifact expectations, and verification evidence before it is presented as supported. | GOAL-01, GOAL-04 | must | This reduces the risk of hand-wavy substrate claims. |
| NFR-04 | Local Firecracker/KVM workflows already shipped by Port must remain usable while the architecture broadens. | GOAL-01 | must | The expansion cannot regress the only proven execution lane. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Substrate model and CLI | Rust tests, help-text review, and model/docs inspection | `cargo test`, `port --help`, and story evidence |
| Machine lifecycle | Rust tests plus CLI proofs against local runtime state | `port machine list`, `status`, and `stop` evidence |
| Hosted-control contract | Design review, contract docs, and model/API tests where implemented | SDD, story evidence, and judge review |
| Artifact mobility | CLI/model tests, docs review, and recorded workflows | Story evidence, docs, and optionally VHS tapes |
| Substrate support matrix | Research-backed docs plus partial implementation evidence | Published support matrix and story reflections |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Firecracker/KVM remains Port's most reliable immediate execution lane. | The expansion would need to pivot away from its only proven backend sooner than planned. | Preserve local tests and lifecycle proofs throughout the epic. |
| Slicer's published Firecracker/PVM lane is a useful competitive reference but not a drop-in design. | Port could overfit to a specialized implementation and miss broader backend needs. | Keep the support matrix explicit and revisit during PVM implementation voyages. |
| arm64 protected virtualization should be treated as adjacent research until mapped to a Port backend. | Port could accidentally promise unsupported arm64 PVM behavior. | Maintain explicit docs and separate stories for arm64 research versus shipped lanes. |
| macOS first-class support is best achieved through Apple Virtualization Framework rather than through Firecracker emulation. | Port could waste effort on an unnatural macOS runtime path. | Plan AVF as the canonical macOS lane and validate with early design stories. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which Firecracker/PVM host-kernel and guest-image pipeline is sustainable enough for Port to own? | Substrate research / implementation | Open |
| How much of the guest transport can remain unchanged once it is brokered through a hosted control plane? | Runtime / control-plane design | Open |
| Should Cloud Hypervisor be first-class or specialized in the first implementation phase? | Architecture | Open |
| What artifact reference format best spans local cache, remote registry, and backend-specific variants? | Artifact design | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port ships a substrate-aware model and planning set that can represent Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, and Apple Virtualization Framework without fragmenting CLI semantics.
- [ ] Port exposes machine lifecycle commands for inventory, status, and stop through the canonical CLI with recorded evidence.
- [ ] Port publishes a hosted-control and node-agent design that explains how guest operations, lifecycle, and transport work for local and hosted execution.
- [ ] Port defines artifact build, push, and pull semantics with backend-aware compatibility expectations.
- [ ] Port documents a credible x86_64-first PVM lane, keeps KVM intact, and treats arm64 protected virtualization as an explicit follow-on research track instead of implied support.
- [ ] Port publishes a first-class macOS lane through Apple Virtualization Framework planning and corresponding CLI/model alignment.
<!-- END SUCCESS_CRITERIA -->
