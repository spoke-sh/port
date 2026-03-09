# PVM Host Kit And Artifact Delivery - Software Requirements Specification

> Make the x86_64 Firecracker/PVM lane reproducible and operable through canonical artifact build, pull, push, validate, and hosted node-preparation workflows.

**Epic:** [1vz3ck000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In scope

- [SCOPE-01] Deliver the missing host-kit and artifact-kit layers behind Port's
  `x86_64` Firecracker/PVM lane, including canonical artifact mobility,
  host-kit packaging metadata, hosted node preparation/import, and operator
  proof.
- [SCOPE-02] Keep the work constrained to canonical Port surfaces: model,
  runtime, CLI, docs, hosted inventory, and artifact workflows.

### Out of scope

- [SCOPE-03] Do not claim an `aarch64` Firecracker/PVM runtime, replace the
  standard Firecracker lane, or deliver Cloud Hypervisor in this voyage.
- [SCOPE-04] Do not add a substrate-specific sidecar CLI or alternate artifact
  workflow outside `port artifacts ...` and the hosted control-plane model.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Existing artifact build and mobility commands remain Port's canonical artifact surface. | product | A second artifact UX would fragment the operator model. |
| Prepared PVM hosts continue to advertise a dedicated host-kit contract instead of reusing standard Firecracker metadata. | architecture | Silent standard-lane reuse would make PVM placement and validation dishonest. |
| Hosted node registration remains the canonical way to advertise prepared cloud capacity. | system | A separate out-of-band host-prep registry would fork the hosted control-plane story. |

## Constraints

- No backward-compatible aliasing or dual artifact schemas.
- `aarch64` Firecracker/PVM remains research-only and must fail fast if an
  operator tries to treat it as ready.
- Verification must be repo-local and automatable through Rust tests or shell
  proofs that can run from the repository root.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model a canonical PVM host-kit package contract that binds together patched Firecracker, prepared host-kernel metadata, and required boot-line expectations for `x86_64` Firecracker/PVM nodes. | SCOPE-01 | FR-03 | Rust unit tests + CLI proof |
| SRS-02 | Port must expose canonical artifact build, validate, push, and pull workflows for `x86_64/firecracker/pvm` kernel and guest-image variants without reusing the standard lane implicitly. | SCOPE-01 | FR-03 | Rust unit tests + CLI proof |
| SRS-03 | Port must let a hosted operator prepare or import a node-local PVM host kit so hosted placement can distinguish ready kits from merely planned nodes. | SCOPE-01 | FR-02 | Rust unit tests + hosted CLI proof |
| SRS-04 | Port must publish the PVM host-kit and artifact workflow in the README, PVM docs, hosted docs, and CLI help with explicit repo-local proof commands. | SCOPE-01 | FR-05 | Docs inspection + help-text test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported PVM combinations, including `aarch64` Firecracker/PVM or missing host-kit contents, must fail fast with explicit diagnostics and no standard-lane fallback. | SCOPE-01 | NFR-01 | Rust unit tests + CLI proof |
| SRS-NFR-02 | PVM host-kit and artifact mobility metadata must be deterministic and portable across local and hosted workflows so the same contracts can be built locally and consumed remotely. | SCOPE-01 | NFR-02 | Rust unit tests + inspection |
| SRS-NFR-03 | The voyage must preserve one canonical CLI and artifact model; operators should use `port artifacts ...`, `port doctor`, hosted inventory flows, and `port machine launch` rather than substrate-specific side tools. | SCOPE-01 | NFR-03 | Help-text test + docs inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
