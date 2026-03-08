# Executable Avf Runtime Foundation - Software Requirements Specification

> Ship the first local AVF launch, doctor, and guest attach workflow on macOS through the canonical Port command model.

**Epic:** [1vzJKE000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- local macOS AVF launch ownership through the canonical `port machine` verbs
- AVF-specific `port doctor` checks and explicit unsupported-boundary messages
- AVF guest attach and console/log mapping through the canonical guest protocol
- operator-facing help, docs, and proof commands for the first executable AVF lane

### Out of Scope

- hosted macOS node-agent ownership
- AVF directory sharing and Rosetta ergonomics beyond explicit boundary docs
- any AVF/PVM lane or arm64 Firecracker/PVM changes

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| macOS AVF development and validation will require conditional compilation because the current repo host is Linux | dependency | The voyage must keep non-macOS builds green while isolating AVF code paths |
| The existing machine-driver seam can absorb a new local substrate without changing the operator verbs | assumption | AVF delivery would need a broader CLI/runtime redesign |
| Existing guest protocol framing can be reused over an AVF-specific host transport | assumption | AVF would fork the guest command model, violating the epic goal |

## Constraints

- AVF is `standard` protection only; Port does not define an AVF/PVM lane.
- macOS entitlement and distribution boundaries must stay explicit in doctor and docs.
- Linux Firecracker and hosted PVM behavior must not regress while AVF lands.
- The canonical operator vocabulary remains `machine launch|list|status|stop` and `guest exec|copy|pty|logs|forward`.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define and validate an AVF local-runtime contract that selects macOS-only execution, `standard` protection, and AVF-specific artifact or runtime expectations without silently falling back to Linux substrates. | SCOPE-03 | FR-02 | automated test + CLI proof |
| SRS-02 | `port doctor` must surface AVF-focused macOS checks and explicit entitlement or unsupported-host boundaries through the canonical CLI output. | SCOPE-03 | FR-03 | automated test + inspection |
| SRS-03 | `port machine launch`, `status`, and `stop` must route AVF-targeted machines through a local AVF driver that writes canonical runtime manifests plus AVF-specific console or transport metadata. | SCOPE-03 | FR-02 | automated test + macOS demo |
| SRS-04 | `port guest exec`, `copy`, `pty`, `logs`, and `forward` must remain reachable through the canonical CLI and shared guest protocol when the selected machine uses the AVF lane. | SCOPE-03 | FR-02 | automated test + macOS demo |
| SRS-05 | CLI help, README, and `docs/avf.md` must publish the native macOS AVF workflow, prerequisites, and still-unsupported AVF boundaries. | SCOPE-03 | FR-03 | command proof + inspection |
| SRS-06 | The AVF local-driver rollout must preserve the existing Firecracker standard and prepared-node PVM workflows while the new substrate lands. | SCOPE-01, SCOPE-03 | FR-03 | automated test + CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Non-macOS hosts and macOS binaries without the required AVF conditions must fail fast with explicit substrate-specific guidance rather than ambiguous runtime errors. | SCOPE-03 | NFR-02 | automated test + inspection |
| SRS-NFR-02 | AVF runtime behavior and operator evidence must keep deterministic runtime metadata and explicit unsupported boundaries throughout the rollout. | SCOPE-03 | NFR-01 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
