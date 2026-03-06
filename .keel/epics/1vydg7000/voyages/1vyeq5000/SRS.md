# Cloud Linux Control Lane - Software Requirements Specification

> Design and partially implement the remote Linux cloud lane, document provider boundaries, and encode the PVM drop decision.

**Epic:** [1vydg7000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Extend the Port host model and examples to represent remote Linux
  cloud targets and provider identity.
- [SCOPE-02] Surface provider-aware diagnostics and launch guardrails through
  the canonical CLI for remote Linux cloud hosts.
- [SCOPE-03] Publish the cloud Linux support matrix, remote operator workflow,
  and explicit PVM decision for the MVP.
- Out of scope: end-to-end remote Firecracker launch on a real cloud host, cloud
  credential management, and provider-specific network automation.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Firecracker still requires a Linux host with KVM even when the operator workstation is macOS or Windows. | Dependency | The cloud lane would need a different runtime technology than the current Port architecture. |
| AWS and GCP remain the only justified provider lanes for future Firecracker MVP work. | Assumption | The provider support matrix and partial implementation boundary would need to change. |
| Azure does not provide a supportable Firecracker MVP path today. | Assumption | The unsupported-provider diagnostics would need to be revised. |
| The existing `ssh` host-connection shape is the right partial implementation seam for remote Linux hosts. | Assumption | A new remote execution contract would be needed before cloud work can ship. |

## Constraints

- The voyage must stay honest about implementation status: remote Linux support
  is partial, not a hidden promise of working remote launch.
- Provider guidance must be explicit about what is supported, what is planned,
  and what is out of scope for the MVP.
- The PVM/confidential/protected VM lane needs an explicit keep-or-drop outcome
  in the shipped docs and planning artifacts.
- Verification must rely on model tests, CLI proofs, and documentation review
  rather than unavailable live cloud credentials.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model remote Linux hosts with explicit provider identity so the canonical config can distinguish local Linux, generic SSH Linux, AWS, GCP, and Azure targets. | SCOPE-01 | FR-06 | automated test + config proof |
| SRS-02 | Port must surface provider-aware diagnostics for remote Linux hosts, including AWS and GCP as justified future lanes and Azure as unsupported for Firecracker MVP. | SCOPE-02 | FR-06 | automated test + CLI proof |
| SRS-03 | Port must fail fast with actionable guidance when operators try to launch a machine against a remote cloud host that is not yet implemented. | SCOPE-02 | FR-06 | automated test + CLI proof |
| SRS-04 | Port must publish the cloud Linux support matrix and remote operator workflow in README and supporting docs using canonical CLI and model terms. | SCOPE-03 | FR-05 | manual review + doc proof |
| SRS-05 | Port must record an explicit PVM keep-or-drop decision for MVP and tie it to the current research outcome. | SCOPE-03 | FR-06 | manual review + research/doc proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Remote cloud guidance must not overstate implementation status; unsupported or partial states must remain explicit in CLI diagnostics and docs. | SCOPE-02, SCOPE-03 | NFR-01 | inspection + CLI proof |
| SRS-NFR-02 | The provider matrix and PVM decision must stay traceable to current research and checked-in planning artifacts. | SCOPE-03 | NFR-03 | manual review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
