# Hosted Control And Substrate Foundations - Software Requirements Specification

> Define the substrate-aware model, hosted control-plane contract, and first machine lifecycle plus artifact mobility slices for Port's expansion beyond the local Firecracker MVP.

**Epic:** [1vz2eV000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Replace the current provider-only runtime framing with a
  substrate-aware model that can represent hypervisor, protection mode, and
  artifact-reference variants.
- [SCOPE-02] Add canonical machine lifecycle surfaces for local runtimes:
  `port machine list`, `port machine status`, and `port machine stop`.
- [SCOPE-03] Define and partially scaffold the hosted Port control-plane
  contract so the same lifecycle and guest semantics can span local and managed
  environments.
- [SCOPE-04] Define artifact mobility contracts for build, publish, pull, cache,
  and variant selection without yet shipping every remote backend.
- [SCOPE-05] Update CLI help and operator docs so the new substrate and hosted
  boundaries are discoverable and honest.
- Out of scope: a fully working PVM runtime, full Apple Virtualization
  Framework execution, full scheduler/host-group support, and production auth.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current runtime manifest and PID inspection flow are sufficient to bootstrap machine inventory/status/stop for local runtimes. | Dependency | Lifecycle surfaces would need an immediate datastore or daemon before they can ship. |
| Port's existing guest protocol can remain the canonical guest-operation model across local and future hosted transports. | Assumption | Hosted control may need a separate guest API surface, increasing migration cost. |
| PVM, Cloud Hypervisor, and Apple Virtualization Framework can be represented as substrate tokens before their runtime lanes are fully implemented. | Assumption | The model might need a different decomposition or an earlier prototype before planning can continue. |
| Artifact mobility can begin with contract and CLI/model changes before a full remote registry implementation exists. | Assumption | Artifact planning would have to be deferred until a concrete backend is chosen. |

## Constraints

- The canonical operator surface remains the `port` CLI and shared model; new
  runtime or hosted work cannot rely on hidden internal-only concepts.
- The local Firecracker/KVM lane must keep working while the broader model is
  introduced.
- Unsupported substrate or protection combinations must fail fast with explicit
  guidance.
- The first hosted-control slice should stay small enough to test locally and
  should not require full multi-user auth or a scheduler.
- The voyage must keep documentation, help text, and board evidence in lockstep
  with any new surface area.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The shared model must represent execution substrate, protection mode, and artifact references or variants without overloading the current provider-only host fields. | SCOPE-01 | FR-01 | automated test + doc review |
| SRS-02 | Port must expose `port machine list`, `port machine status`, and `port machine stop` as canonical lifecycle surfaces for local runtime state and shape them so they can extend to hosted runtimes later. | SCOPE-02, SCOPE-05 | FR-02 | automated test + CLI proof |
| SRS-03 | Runtime state and manifests must carry enough data to support deterministic lifecycle inspection and stop behavior without requiring Firecracker's REST API. | SCOPE-02 | FR-02 | automated test + inspection |
| SRS-04 | Port must author and partially implement a hosted control-plane contract that separates long-lived control ownership from the local CLI process while preserving the canonical guest-operation model. | SCOPE-03 | FR-03 | automated test + design review |
| SRS-05 | Port must define artifact publish, pull, cache, and variant-selection contracts in the model and documentation so later remote backends can be implemented without changing the canonical operator story. | SCOPE-01, SCOPE-04, SCOPE-05 | FR-04 | model test + doc review |
| SRS-06 | CLI help and operator documentation must explain the new substrate-aware support matrix, including which lanes are supported, partial, experimental, or design-only. | SCOPE-05 | FR-08 | doc review + CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported combinations of substrate, protection mode, and platform must fail fast with explicit diagnostics rather than implicit fallback behavior. | SCOPE-01, SCOPE-05 | NFR-01 | automated test + CLI proof |
| SRS-NFR-02 | Local lifecycle surfaces and hosted-control scaffolding must preserve one canonical command model and one canonical guest-operation model. | SCOPE-02, SCOPE-03, SCOPE-05 | NFR-02 | inspection + doc review |
| SRS-NFR-03 | Artifact references and variant resolution must remain deterministic across architecture and substrate lanes. | SCOPE-01, SCOPE-04 | NFR-03 | automated test + inspection |
| SRS-NFR-04 | The voyage must add automated coverage and recorded CLI/documentation evidence for each newly exposed operator surface. | SCOPE-02, SCOPE-03, SCOPE-05 | NFR-04 | automated test + CLI proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
