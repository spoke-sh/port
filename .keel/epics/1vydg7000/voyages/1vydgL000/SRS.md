# Local Linux CLI Runtime - Software Requirements Specification

> Deliver a coherent local Linux Firecracker workflow through the Port CLI, including artifact contracts, launch orchestration, guest agent reachability, and operator-facing documentation for the first MVP path.

**Epic:** [1vydg7000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Define the canonical Port CLI and shared model for local hosts, artifacts, instances, and guest operations.
- [SCOPE-02] Validate a Linux host for local Firecracker execution before launch.
- [SCOPE-03] Launch and manage a local Firecracker microVM from Port-managed artifact inputs.
- [SCOPE-04] Reach a guest agent over a supported transport and expose `exec`, `copy`, `pty`, `logs`, and `forward`.
- [SCOPE-05] Produce the kernel and guest-image artifacts needed for the local Linux MVP path and validate their contracts.
- [SCOPE-06] Publish Linux/macOS/Windows operator guidance for the MVP workflows supported by this voyage.
- Out of scope: cloud-host orchestration beyond the interfaces needed to keep the model extensible.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Firecracker 1.14.x is available from Nix and runnable on supported Linux hosts. | Dependency | Local launch cannot be executed through the CLI. |
| The host exposes `/dev/kvm`, `iproute2`, and firewall/NAT tools needed for networking. | Dependency | Port must fail early and document the unsupported state. |
| The guest image can boot a small userspace with the Port guest agent as the primary control surface. | Assumption | Guest capabilities would need a different transport or base image. |
| macOS and Windows workflows operate Port against Linux hosts rather than local Firecracker. | Assumption | Platform docs and CLI validation would need a different support model. |

## Constraints

- Firecracker requires Linux and KVM; non-Linux operators must target a Linux host rather than attempting local execution.
- The MVP uses a hard cutover policy: one canonical config/model path, no compatibility aliases.
- CLI discoverability is part of scope; commands are incomplete until help text, examples, and docs exist.
- Verification must be scriptable where possible and explicitly recorded through `keel`.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must expose a canonical CLI and serialized model for artifacts, hosts, instances, and guest actions. | SCOPE-01 | FR-01 | automated test + CLI help proof |
| SRS-02 | Port must provide a local host preflight that validates Linux, `/dev/kvm`, Firecracker availability, and required networking tools before launch. | SCOPE-02 | FR-02 | automated test + CLI proof |
| SRS-03 | Port must launch a Firecracker microVM locally from model-backed kernel and guest-image artifacts, persist runtime metadata, and surface console/log locations. | SCOPE-03 | FR-02 | automated test + end-to-end demo |
| SRS-04 | Port must connect to a guest agent over the canonical transport and expose `exec`, `copy`, `pty`, `logs`, and `forward` through CLI commands that map onto the shared model. | SCOPE-04 | FR-03 | automated test + CLI proofs |
| SRS-05 | Port must build and validate the kernel artifact used by the local Linux MVP path. | SCOPE-05 | FR-04 | build command + validation command |
| SRS-06 | Port must build and validate the guest-image artifact that boots the guest agent and required userspace. | SCOPE-05 | FR-04 | build command + validation command |
| SRS-07 | Port must document artifact contracts, runtime behavior, supported workflows, and platform limitations in README and supporting docs. | SCOPE-05, SCOPE-06 | FR-05 | manual review + CLI/doc consistency check |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported hosts, missing artifacts, and unsupported platforms must return actionable diagnostics before Port mutates runtime state. | SCOPE-02, SCOPE-06 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | CLI help text, examples, and documentation must remain consistent with the current command model and supported workflows. | SCOPE-01, SCOPE-06 | NFR-03 | manual review + CLI help proof |
| SRS-NFR-03 | Artifact outputs and runtime state must use deterministic paths or names derived from checked-in configuration or explicit CLI flags. | SCOPE-03, SCOPE-05 | NFR-02 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
