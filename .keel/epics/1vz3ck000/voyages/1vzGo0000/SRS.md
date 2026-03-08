# X86 64 PVM Host Kit Foundation - Software Requirements Specification

> Define and begin implementing the x86_64 Firecracker/PVM host-kit, doctor, and artifact foundations for cost-controlled cloud execution.

**Epic:** [1vz3ck000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

This voyage turns the current PVM research and documentation into an executable
foundation slice for the `x86_64/firecracker/pvm` lane.

It delivers:

- an explicit PVM host-kit contract in the shared model and operator docs
- fail-fast `port doctor` checks for the x86_64 Firecracker/PVM host kit
- dedicated kernel and guest-image build/validate contracts for
  `x86_64/firecracker/pvm`
- CLI/help/docs updates plus repository-local evidence for the supported
  x86_64 keep and arm64 research-only boundary

It does not deliver:

- a full Firecracker/PVM launch implementation
- any claim of `aarch64/firecracker/pvm` support
- hosted scheduler placement for PVM-capable nodes
- production packaging of the host kernel or patched Firecracker binary

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The existing artifact selector model can keep using `architecture + substrate + protection_mode` without introducing a second PVM-only artifact vocabulary. | Assumption | Artifact work would need a wider model migration before implementation can start. |
| The x86_64 PVM lane should fail fast on host-kit absence instead of pretending the standard Firecracker lane is compatible. | Assumption | Port could silently blur supported and unsupported execution paths. |
| Repository-local evidence is sufficient for the foundation slice even though a true PVM boot proof still requires a prepared host kit outside the default developer machine. | Dependency | Verification would need external host infrastructure before the voyage could close. |

## Constraints

- Keep `x86_64` as the only planned PVM implementation lane in this voyage.
- Keep `aarch64/firecracker/pvm` research-only in model, CLI, docs, and doctor
  output.
- Do not add silent fallback from `pvm` to `standard` artifacts, doctor
  results, or launch behavior.
- Preserve the current local standard Firecracker lane while broadening the
  host/artifact contracts.
- Keep one canonical CLI and artifact model instead of introducing a separate
  PVM-only command family.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model an explicit x86_64 Firecracker/PVM host-kit contract, including the prepared host boundary, required boot-line expectation (`pti=off`), and dedicated patched Firecracker binary expectation. | SCOPE-01 | FR-03 | automated test + docs inspection |
| SRS-02 | `port doctor` must report x86_64 Firecracker/PVM host-kit readiness and fail fast when the host architecture, boot line, or required PVM binary contract is missing or incompatible. | SCOPE-01 | FR-03 | automated test + CLI proof |
| SRS-03 | Artifact build and validation flows must support `x86_64/firecracker/pvm` kernel and guest-image variants with no fallback to the standard Firecracker lane. | SCOPE-01 | FR-03 | automated test + command proof |
| SRS-04 | README, PVM docs, and CLI help must explain the x86_64 keep decision, the arm64 research-only boundary, and the executable repository-local proof workflow for the foundation slice. | SCOPE-01 | FR-05 | manual review + CLI/doc proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported architectures, host-kit states, and artifact selections must fail fast with explicit diagnostics and no silent fallback to the standard Firecracker lane. | SCOPE-01 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | PVM host-kit and artifact-kit selection must remain deterministic and reproducible across model rendering, doctor output, and artifact commands. | SCOPE-01 | NFR-02 | automated test + command proof |
| SRS-NFR-03 | The voyage must not regress the current local standard Firecracker lane while adding PVM-specific contracts and checks. | SCOPE-01 | NFR-03 | automated test + regression inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
