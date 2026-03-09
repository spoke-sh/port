# Cloud Hypervisor Execution Lane - Product Requirements

## Problem Statement

Port still models Cloud Hypervisor as a planned substrate only. That leaves a
meaningful product gap versus the target hosted multi-substrate shape and
versus Slicer-class operator expectations, where a second Linux hypervisor lane
exists alongside Firecracker.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship an executable local Cloud Hypervisor lane through the canonical Port CLI and runtime model. | `port doctor`, `machine launch|status|stop`, and guest verbs work for a Cloud Hypervisor machine without a substrate-specific command tree. | First voyage |
| GOAL-02 | Ship the first hosted Cloud Hypervisor lane through the existing control-plane and node-agent ownership split. | Hosted placement, launch, status, stop, and guest attach all route through live hosted contracts for Cloud Hypervisor machines. | First voyage |
| GOAL-03 | Keep Cloud Hypervisor operator discovery and artifact handling coherent. | README, help text, examples, and docs publish one canonical workflow plus explicit unsupported boundaries. | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform operator | Maintains Linux hosts and chooses the right Port substrate for cost, compatibility, or rollout constraints. | A second executable Linux substrate with explicit diagnostics and artifact expectations. |
| Hosted control-plane operator | Runs Port as a hosted system with node agents on execution hosts. | The same control-plane and node-agent model working across Cloud Hypervisor as well as Firecracker. |
| macOS or Windows Port operator | Uses Port from a non-Linux workstation while Linux nodes execute the actual hypervisor. | A coherent hosted workflow that does not force Linux-only operator vocabulary. |

## Scope

### In Scope

- [SCOPE-01] Model Cloud Hypervisor as an executable `standard` Linux substrate in the shared Port config and artifact contract.
- [SCOPE-02] Implement local Cloud Hypervisor launch, status, stop, and doctor/preflight checks through the existing machine-driver seam.
- [SCOPE-03] Reuse the canonical guest protocol for Cloud Hypervisor guest exec, copy, pty, logs, and forward.
- [SCOPE-04] Implement the hosted control-plane and node-agent path for Cloud Hypervisor machines.
- [SCOPE-05] Publish canonical help text, docs, examples, and proof workflows for the shipped Cloud Hypervisor lane.

### Out of Scope

- [SCOPE-06] Confidential-VM or TDX/SEV-style Cloud Hypervisor protection modes.
- [SCOPE-07] A macOS-native Cloud Hypervisor lane.
- [SCOPE-08] Alternate API packages, multi-tenant auth redesign, or scheduler-policy changes unrelated to Cloud Hypervisor routing.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must let a machine declare Cloud Hypervisor as its substrate and resolve the matching kernel and guest-image artifact variants explicitly. | GOAL-01, GOAL-03 | must | Operators need one canonical model and artifact vocabulary across substrates. |
| FR-02 | `port doctor` plus local machine launch, status, and stop must execute Cloud Hypervisor machines through the existing runtime driver seam with actionable host-preflight failures. | GOAL-01 | must | A second substrate is not credible if it only exists in docs or model rendering. |
| FR-03 | Cloud Hypervisor machines must expose the same guest verb surface as Firecracker and AVF through the canonical CLI and protocol. | GOAL-01, GOAL-02 | must | Port's product shape depends on one guest-control model across substrates. |
| FR-04 | Hosted control-plane and node-agent flows must place and execute Cloud Hypervisor machines without falling back to Firecracker-specific assumptions. | GOAL-02 | must | Hosted Port needs substrate breadth, not only local experimentation. |
| FR-05 | README, CLI help, examples, and dedicated docs must publish one executable Cloud Hypervisor workflow, including local and hosted proof steps plus explicit unsupported boundaries. | GOAL-03 | must | A capability is not done until it is discoverable and learnable from the CLI surface. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Unsupported or incomplete Cloud Hypervisor combinations must fail fast with substrate, architecture, machine, and host detail; Port must not silently route them to Firecracker. | GOAL-01, GOAL-02, GOAL-03 | must | Hard cutover keeps substrate semantics coherent. |
| NFR-02 | Cloud Hypervisor must reuse the existing Port guest protocol and hosted route families rather than inventing a second guest API surface. | GOAL-01, GOAL-02 | must | Reusing the shared protocol is Port's strongest architectural advantage today. |
| NFR-03 | The first Cloud Hypervisor slice must include automated Rust verification plus CLI-level proof and updated board evidence. | GOAL-01, GOAL-02, GOAL-03 | must | The repo's verification-driven workflow requires concrete proof, not only design claims. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Local Cloud Hypervisor runtime | Rust unit tests plus CLI proof | `cargo test`, story verify scripts, and runtime artifacts under an isolated runtime root |
| Hosted Cloud Hypervisor routing | Rust integration tests plus CLI proof | control-plane/node-agent route tests and CLI story proofs |
| Operator discovery | doc/help inspection plus CLI proof | story verify scripts that inspect README, docs, help text, and example workflows |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Cloud Hypervisor remains a viable Linux KVM substrate with `x86_64` and `aarch64` support and primitives Port can map onto its runtime contract. | The epic may need to narrow its architecture promise or switch priority to another substrate. | Upstream docs plus repo-local implementation proofs during the first voyage. |
| Port's existing machine-driver seam and guest protocol are sufficient for a second Linux hypervisor without major control-plane redesign. | The first voyage could stall in architecture work instead of delivering user value. | Story-level implementation and hosted-route proof. |
| The current artifact selector vocabulary can absorb Cloud Hypervisor variants cleanly. | Artifact handling could become inconsistent across substrates. | Model/runtime tests and doc/help review. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much of Cloud Hypervisor's REST API should Port own directly versus simple process ownership plus static config generation? | Epic owner | Open |
| Host networking and console wiring differ from Firecracker; gaps there could delay end-to-end local proof. | Epic owner | Open |
| Current development infrastructure verifies primarily on Linux x86_64; `aarch64` support may remain modeled and tested rather than directly exercised in this repo. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port can boot a Cloud Hypervisor machine locally through `port machine launch --machine ...` and manage it with `status`, `stop`, and guest verbs through the canonical CLI.
- [ ] Hosted control-plane and node-agent flows can place and run a Cloud Hypervisor machine through the existing Port control contracts.
- [ ] Cloud Hypervisor artifact, doctor, and guest-transport behavior are documented and fail fast without Firecracker fallback.
- [ ] The board contains executable Cloud Hypervisor stories with verification plans and no placeholder planning content.
<!-- END SUCCESS_CRITERIA -->
