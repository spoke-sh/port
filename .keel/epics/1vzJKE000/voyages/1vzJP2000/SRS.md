# Prepared Linux Pvm Runtime - Software Requirements Specification

> Ship the first executable x86_64 Firecracker/PVM runtime path on prepared Linux nodes through the canonical Port model, CLI, and hosted node-agent ownership.

**Epic:** [1vzJKE000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- executable x86_64 Firecracker/PVM launch on prepared Linux nodes
- node-agent ownership of the prepared-host launch path
- hosted control-plane launch routing to prepared nodes instead of provider
  guidance
- operator proofs and docs for prepared-node PVM launch

### Out of Scope

- arm64 Firecracker/PVM execution
- AVF runtime delivery
- broad scheduler policy beyond explicit prepared-node selection

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Prepared Linux nodes can supply a host kit with a patched Firecracker binary and required host boot state | dependency | No executable PVM launch path exists |
| Existing hosted node-agent ownership can be extended to launch rather than only inspect and stop | assumption | Hosted PVM launch needs control-plane redesign |
| PVM kernel and guest-image variants remain available through the existing artifact catalog | dependency | Launch path cannot remain canonical |

## Constraints

- `aarch64` Firecracker/PVM remains research-only throughout this voyage.
- The canonical operator model must stay on `port machine ...` and `port guest ...`.
- Standard Firecracker launch must remain intact; PVM is additive, not a
  fallback replacement.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define and validate a prepared-node PVM host-kit contract that selects the patched Firecracker binary and required host prerequisites explicitly. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-02 | The node-agent runtime must launch x86_64 Firecracker/PVM guests on prepared Linux nodes using the canonical artifact and runtime model. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-03 | Hosted `port machine launch` must route admission-ready PVM machines through the live control-plane and node-agent path instead of falling back to provider guidance. | SCOPE-01 | FR-01 | automated test + demo |
| SRS-04 | Documentation and CLI help must publish the prepared-node PVM workflow, failure boundaries, and proof commands. | SCOPE-01 | FR-03 | command proof + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Prepared-node PVM launch failures must surface explicit host-kit or placement causes rather than generic runtime errors. | SCOPE-01 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | The standard Firecracker lane must remain executable after the PVM launch path lands. | SCOPE-01 | NFR-02 | automated test + CLI proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
