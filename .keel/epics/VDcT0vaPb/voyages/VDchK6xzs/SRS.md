# Release Matrix And Packaging Foundations - Software Requirements Specification

> Define the first supported Linux and macOS release matrix, canonical package
> output, install workflow, and AVF distribution boundary so delivery can ship
> an installable Port surface.

**Epic:** [VDcT0vaPb](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] A concrete support matrix for the first installable Linux and
  macOS targets, including operator-facing boundaries for unsupported
  environments.
- [SCOPE-02] One canonical package artifact format and deterministic output
  layout for the `port` CLI on the supported targets.
- [SCOPE-03] A repo-local build, package, and install proof workflow driven by
  canonical `just` and `port` commands.
- [SCOPE-04] The macOS AVF launcher-helper and entitlement boundary as part of
  the install and release contract.
- [SCOPE-05] A release-validation path that keeps package proof tied to the
  canonical board, test, and doctor workflows.

### Out of Scope

- [SCOPE-90] Automated release publication, notarization, or CI-driven release
  orchestration.
- [SCOPE-91] Homebrew taps, native installer bundles, GUI launchers, or
  auto-update systems.
- [SCOPE-92] New runtime substrates or a second macOS-specific command family.
- [SCOPE-93] Windows-native install artifacts; Windows remains a Linux-backed
  workflow through WSL or a remote Linux host.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Cargo and the repo-local shell toolchain are sufficient to build first-slice install artifacts for the selected targets. | dependency | The voyage would need a larger packaging toolchain decision before delivery can start. |
| A tarball-style CLI package is acceptable as the first canonical install artifact for Linux and macOS. | assumption | The voyage would need a broader installer decision and likely more implementation stories. |
| The macOS AVF lane can remain first-class if the core package documents the launcher-helper and entitlement boundary explicitly, even if the helper is not yet a polished app-bundle experience. | assumption | The macOS scope may need to expand into separate helper packaging work immediately. |

## Constraints

- Keep `port` as the only published operator command surface; do not add a
  second macOS-specific launcher or packaging CLI.
- Use canonical repo workflows for proof: `just`, `port doctor`, workspace
  tests, and board health checks.
- Preserve the hard-cutover policy: one package contract, one support matrix,
  and no compatibility aliasing to older source-first guidance.
- Keep Windows documented as a Linux-backed operator workflow for this slice,
  not as a native packaging target.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `README.md`, `RELEASE.md`, and the install-focused docs must define the first supported Linux and macOS targets, their canonical package artifact, and the operator boundary for unsupported environments. | SCOPE-01, SCOPE-04 | FR-01 | inspection + command proof |
| SRS-02 | The repo must provide a deterministic package workflow that emits one canonical install artifact per supported target with explicit version, target, and included-file reporting. | SCOPE-02, SCOPE-03 | FR-03 | command proof + automated test |
| SRS-03 | The first install workflow must let an operator extract or install the package and run the canonical `port` binary without falling back to `cargo run -p port-cli`. | SCOPE-02, SCOPE-03 | FR-02 | command proof + recording |
| SRS-04 | macOS release guidance and doctor/help surfaces must explain the AVF launcher-helper requirement, the distributed-target entitlement boundary, and the expected failure guidance on unsupported hosts. | SCOPE-04 | FR-04 | automated test + inspection |
| SRS-05 | The release checklist for this slice must anchor validation on `just`, `port doctor`, workspace tests, and board health rather than a disconnected packaging-only checklist. | SCOPE-05 | FR-05 | command proof + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Package names, staging layout, and included files must be deterministic for each supported target across repeated runs. | SCOPE-02, SCOPE-03 | NFR-01 | automated test |
| SRS-NFR-02 | Unsupported targets and missing packaging prerequisites must fail fast with explicit guidance and no silent fallback to a source-only workflow. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-02 | automated test + inspection |
| SRS-NFR-03 | The first package proof must remain repo-local and must not require external release credentials or hosted publication infrastructure. | SCOPE-03, SCOPE-05 | NFR-01 | command proof + recording |
| SRS-NFR-04 | The voyage must leave the board with implementation stories and verification commands that map directly to this installable slice rather than another roadmap-only placeholder. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | inspection + board review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VDchsxHfp](../../../../stories/VDchsxHfp/README.md) Publish Installable Support Matrix And Release Contract | SRS-01 |
| [VDchsvWfn](../../../../stories/VDchsvWfn/README.md) Implement Canonical CLI Package Workflow | SRS-02, SRS-NFR-01, SRS-NFR-02 |
| [VDchsxHfm](../../../../stories/VDchsxHfm/README.md) Add Install Proof For Packaged Port | SRS-03, SRS-NFR-03 |
| [VDchsw9fh](../../../../stories/VDchsw9fh/README.md) Surface AVF Distribution Boundary In Docs And Doctor | SRS-04, SRS-05, SRS-NFR-02 |
