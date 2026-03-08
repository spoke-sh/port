# PVM Runtime Admission And Placement - Software Requirements Specification

> Turn x86_64 Firecracker PVM from a documented foundation into a runtime
selection and placement contract across local and hosted Port surfaces

**Epic:** [1vz3ck000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

This voyage turns the existing PVM foundation into the first executable runtime
and placement contract.

In scope:

- represent x86_64 PVM readiness explicitly in Port's machine, host, and hosted
  node contracts
- select the PVM-specific Firecracker runtime inputs for local launch when a
  machine opts into `protection_mode = "pvm"`
- surface and enforce hosted PVM admission so a hosted machine cannot be placed
  onto a node that does not advertise the required host kit
- publish the resulting local and hosted operator workflows in canonical CLI and
  docs

Out of scope:

- a fully shipped `firecracker-pvm` host package for every target platform
- claiming `aarch64` Firecracker/PVM runtime support
- production scheduler heuristics or multi-node placement policies beyond the
  explicit PVM readiness gate
- a successful PVM boot proof on machines that do not have the prepared host
  kit installed

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The x86_64 Firecracker/PVM lane remains strategically required for cloud cost control. | product | If this changes, the runtime and placement work would no longer be the next priority. |
| Hosted control-plane and node-agent serve paths remain the canonical hosted transport seam. | architecture | A different hosted architecture would invalidate the planned admission and inventory contracts. |
| The current guest protocol stays unchanged while lifecycle ownership and substrate selection evolve. | architecture | A guest-protocol change would expand the voyage and break the epic's single-surface goal. |
| CI and most developer machines will not have a prepared PVM host kit. | verification | The voyage must rely on fail-fast proofs, targeted tests, and explicit diagnostics rather than assuming a live PVM boot environment. |

## Constraints

- Unsupported combinations must fail fast with explicit diagnostics and no
  fallback to standard Firecracker. `[NFR-01]`
- The shipped Linux Firecracker standard lane must remain intact while PVM
  support is added beside it. `[NFR-02]`
- Hosted behavior must preserve the current `port machine ...` and
  `port guest ...` vocabulary instead of introducing a second PVM-specific CLI.
  `[FR-05]`
- The voyage may rely on a prepared operator host kit for true runtime proof,
  but it must still provide executable evidence on an unprepared development
  host.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model x86_64 PVM readiness as an explicit capability across local Firecracker support and hosted node inventory so the runtime can distinguish `ready`, `planned`, and `research-only` states without inference. | SCOPE-01 | FR-03 | automated test + config/CLI proof |
| SRS-02 | The local Firecracker runtime path must resolve PVM-specific launch inputs, including the patched VMM binary contract and host-kit preflight, whenever a machine selects `protection_mode = "pvm"`. | SCOPE-02 | FR-03 | automated test + launch-path proof |
| SRS-03 | Hosted control-plane and node-agent flows must reject or surface PVM machine placement unless the resolved node advertises the required x86_64 PVM capability, while preserving the standard lane for other machines. | SCOPE-03 | FR-02 | automated test + hosted CLI proof |
| SRS-04 | README, `docs/pvm.md`, sample config, and CLI help must describe the local and hosted PVM admission workflow, including what Port can prove on an unprepared host and what still requires a prepared host kit. | SCOPE-04 | FR-05 | command proof + manual review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | PVM selection and placement must fail fast with capability-specific diagnostics and must never silently fall back to the standard Firecracker lane. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated test + CLI proof |
| SRS-NFR-02 | The voyage must preserve reproducible standard and PVM artifact/runtime behavior side-by-side so operators can keep using the standard lane while preparing the PVM host kit. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | automated test + command proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
