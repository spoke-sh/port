# Port MVP - Product Requirements

## Problem Statement

Port currently has no executable runtime, artifact pipeline, or operator workflow.
The MVP must turn the project into a usable CLI-first Firecracker product: a
Linux operator should be able to build or obtain canonical artifacts, validate
the host, launch a microVM locally, reach a guest agent for exec/copy/pty/logs
and port forwarding, and understand what is or is not supported on cloud Linux,
macOS, and Windows.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Deliver a real local Linux Firecracker workflow through the Port CLI. | A new Linux operator can build artifacts, validate prerequisites, and boot a microVM using documented CLI commands. | One reproducible end-to-end proof recorded on the board |
| GOAL-02 | Expose guest operations through one coherent CLI and model. | `exec`, `copy`, `pty`, `logs`, and `forward` are all reachable from the canonical CLI surface and the shared machine model. | All five capabilities demonstrated with story evidence |
| GOAL-03 | Make artifact production reproducible and inspectable. | Kernel and guest-image pipelines produce documented outputs with validation steps and failure modes. | Both artifact classes have build and validation commands plus docs |
| GOAL-04 | Define a credible path beyond local Linux. | Cloud Linux scope is designed with a partial implementation, the PVM lane has an explicit decision, and macOS/Windows operator workflows are documented. | One cloud design slice shipped; operator docs published |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Linux Operator | A developer or platform engineer running Port on a Linux host with KVM access. | A supported, learnable path to create artifacts, boot a VM, and use guest capabilities without reading source code. |
| Automation Integrator | A tool or agent author embedding Port into a larger workflow. | A stable machine model and CLI semantics that can be scripted safely. |
| Cross-Platform Operator | A macOS or Windows user controlling Linux Firecracker hosts from a non-Linux workstation. | Clear constraints, supported workflows, and commands that map onto remote Linux execution. |

## Scope

### In Scope

- [SCOPE-01] Canonical CLI, shared model, and help text for artifacts, machines, local hosts, and guest actions.
- [SCOPE-02] Local Linux Firecracker launch, runtime state management, and host preflight validation.
- [SCOPE-03] Guest-agent transport and capabilities for `exec`, `copy`, `pty`, `logs`, and `forward`.
- [SCOPE-04] Kernel and guest-image artifact build and validation pipelines with documented contracts.
- [SCOPE-05] Cloud Linux design plus a partial implementation aligned to currently justified provider support.
- [SCOPE-06] Operator documentation for Linux, macOS, and Windows MVP workflows and limitations.

### Out of Scope

- [SCOPE-07] Non-Linux guest operating systems.
- [SCOPE-08] Multi-node scheduling, fleet reconciliation, or Kubernetes integration beyond documentation hooks.
- [SCOPE-09] Backward-compatibility shims for pre-MVP formats or commands.
- [SCOPE-10] Production hardening beyond the validation and observability needed for MVP evidence.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must provide a discoverable CLI and machine model for configuring artifacts, hosts, launches, and guest actions. | GOAL-01, GOAL-02 | must | The CLI is the product surface for both direct operators and automation. |
| FR-02 | Port must launch a Linux Firecracker microVM locally on a KVM-capable host through the CLI, end-to-end. | GOAL-01 | must | Local Linux launch is the anchor workflow for the MVP. |
| FR-03 | Port must expose guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical CLI and shared model. | GOAL-02 | must | Guest capability reachability is a core acceptance gate for the MVP. |
| FR-04 | Port must build and validate canonical kernel and guest-image artifacts for the MVP environment. | GOAL-03 | must | Artifact pipelines make the runtime reproducible instead of ad hoc. |
| FR-05 | Port must document artifact contracts, runtime behavior, platform constraints, and supported workflows where those behaviors become canonical. | GOAL-01, GOAL-03, GOAL-04 | must | The MVP is not usable if behavior exists only in code. |
| FR-06 | Port must design a cloud Linux path and land a partial implementation guided by current provider support, with an explicit keep-or-drop decision for the PVM lane. | GOAL-04 | must | Cloud work needs a justified MVP boundary instead of speculative scope creep. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | All MVP workflows must fail fast with actionable validation errors for unsupported hosts, missing artifacts, or unsupported platforms. | GOAL-01, GOAL-04 | must | Operators need to understand why a workflow is unsupported before runtime side effects happen. |
| NFR-02 | Artifact production and guest/runtime behavior must be reproducible from checked-in configuration and documented commands. | GOAL-03 | must | Reproducibility is required for evidence and future automation. |
| NFR-03 | Every implemented MVP behavior must have board-linked verification evidence and story reflection. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | Traceability is part of the delivery contract in this repository. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Local Linux runtime | Automated Rust tests plus CLI proofs on a KVM-capable Linux host | `cargo test`, host preflight output, and an end-to-end launch record |
| Guest capabilities | Automated protocol/unit tests plus CLI proofs for each guest action | Story evidence for `exec`, `copy`, `pty`, `logs`, and `forward` |
| Artifact pipelines | Build commands and validation commands recorded in docs and story evidence | Generated artifacts, validation output, and failure-path checks |
| Cloud/platform support | Research package, design docs, CLI/model tests, and manual documentation review | Bearing evidence, partial implementation proof, and published operator guides |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Linux hosts used for local MVP work expose `/dev/kvm` and can run Firecracker. | The local launch acceptance gate cannot be proven in this environment. | Host preflight checks and local launch evidence |
| Cloud Linux support can be represented as remote Linux hosts managed through the same Port model. | The cloud lane may require a different product shape. | Research package and partial implementation |
| macOS and Windows operators will control Linux hosts remotely rather than running Firecracker locally. | CLI and docs could promise unsupported local execution. | Platform documentation and command validation |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which cloud providers offer a supportable nested-virtualization lane for Firecracker today? | Research lane | In progress |
| Can a protected/confidential VM lane coexist with KVM-based Firecracker for MVP timelines? | Research lane | In progress |
| How small can the guest image stay while still supporting copy, PTY, forwarding, and logs? | Runtime implementer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A Linux operator can run a documented Port CLI workflow that validates the host, resolves artifacts, launches Firecracker, and records proof.
- [ ] The canonical CLI exposes `exec`, `copy`, `pty`, `logs`, and `forward` against the guest agent with recorded evidence.
- [ ] Kernel and guest-image artifact pipelines exist, are reproducible, and document inputs, outputs, and validation.
- [ ] Cloud Linux support has a documented design, a partial implementation, and an explicit PVM keep-or-drop decision backed by research.
- [ ] README and supporting docs let a new operator understand supported platforms, limitations, and MVP workflows without reading source code first.
<!-- END SUCCESS_CRITERIA -->
