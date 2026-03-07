# Clarify Help Examples - Software Requirements Specification

> Make port --help examples explicit about their environment prerequisites and runnable workflow order.

**Epic:** [1vydg7000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Update the canonical CLI help and operator-facing docs so
  published examples state their environment prerequisites and a runnable
  workflow order.
- [SCOPE-02] Prove the documented help examples directly against the shipped CLI
  surface.
- Out of scope: changing Port runtime behavior, dependency installation
  mechanics, or artifact contents.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current breakage is guidance drift, not a hidden runtime regression in the example command syntax. | Assumption | The voyage would need to widen into a runtime bug fix instead of a help/doc correction. |
| `nix develop` remains the canonical way to obtain Firecracker and the artifact-build dependencies in this repository. | Dependency | The help and docs would need a different prerequisite story. |

## Constraints

- The CLI remains a first-class product surface, so the fix has to land in
  `port --help`, not only in README prose.
- Published examples must stay honest about the difference between command
  syntax and environment prerequisites.
- Verification must demonstrate that the updated examples are runnable in the
  stated environment.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port --help` must make the environment prerequisites for local artifact and launch workflows explicit and present the example commands in a runnable order. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-02 | Operator-facing docs must align with the help text so a repo user can tell when to use `nix develop`, when `port doctor` is the gate, and why a launch example may fail outside that environment. | SCOPE-01 | FR-05 | manual review + doc proof |
| SRS-03 | The recorded evidence for the voyage must execute the published help-example workflow in the stated environment and show the expected outcome for the local launch prerequisite check. | SCOPE-02 | FR-01 | CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The revised help/examples must not overpromise local launch availability outside the documented environment; prerequisite boundaries must stay explicit. | SCOPE-01, SCOPE-02 | NFR-01 | inspection + CLI proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
