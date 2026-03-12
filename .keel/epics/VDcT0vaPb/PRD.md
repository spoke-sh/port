# Installable Linux And Mac Developer Experience - Product Requirements

> Make Port installable as a supported Linux and macOS product surface so
> operators can evaluate and reuse it without starting from a repo checkout.

## Problem Statement

Port already ships real Linux and macOS lanes, but the release contract is
still repo-centric:

- `RELEASE.md` still treats packaged binaries, installers, checksums or
  signatures, and automated publication as open follow-on work
- `README.md` publishes Linux and macOS as first-class environments, but it
  does not yet give operators a canonical install artifact or target-triple
  support matrix
- the macOS AVF lane has a real runtime contract, yet the install/distribution
  boundary for the launcher helper, virtualization entitlement, and related
  constraints is still only partially productized

That makes Port credible inside this repository while slowing adoption in other
projects. Operators can build it from source, but they cannot yet rely on a
clear first install path with explicit platform boundaries.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Publish a concrete Linux and macOS support matrix. | `README.md` and `RELEASE.md` define the supported targets and operator boundary for the first installable slice. | First voyage complete |
| GOAL-02 | Ship one canonical package contract for the `port` CLI. | Repo-local workflows can produce a stable install artifact per supported target with explicit included files and install steps. | First voyage complete |
| GOAL-03 | Make the macOS AVF distribution boundary explicit. | Docs and doctor/help surfaces explain the AVF launcher-helper requirement and entitlement boundary for distributed macOS targets. | First voyage complete |
| GOAL-04 | Keep release validation anchored on canonical Port workflows. | The first release proof uses `just`, `port doctor`, workspace tests, and board checks instead of ad hoc packaging-only scripts. | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| External Operator / Evaluator | Wants to try Port on a supported Linux or macOS host without learning the repo first. | A supported install artifact, clear platform boundary, and a short verification path |
| Maintainer / Release Steward | Owns versioned artifacts and release readiness for the project. | A deterministic package workflow and explicit release checklist that matches shipped behavior |
| macOS AVF Operator | Runs Port through the AVF lane on macOS. | Clear launcher-helper, entitlement, and doctor expectations before treating macOS as a supported install lane |

## Scope

### In Scope

- [SCOPE-01] The first supported Linux and macOS target matrix for installable Port artifacts.
- [SCOPE-02] One canonical package output and install workflow for the `port` CLI across the supported targets.
- [SCOPE-03] Release-contract updates in `README.md`, `RELEASE.md`, and related operator docs so the install surface is explicit and auditable.
- [SCOPE-04] The macOS AVF distribution boundary, including launcher-helper expectations and entitlement guidance for distributed targets.
- [SCOPE-05] Repo-local verification and proof commands for the new installable workflow.

### Out of Scope

- [SCOPE-90] Automated release publication, Homebrew tap management, notarization pipelines, or auto-update systems.
- [SCOPE-91] A GUI launcher, app bundle, or a second platform-specific command family.
- [SCOPE-92] New runtime substrates or changes to the canonical `machine`, `guest`, `service`, or hosted control-plane ownership model.
- [SCOPE-93] Generic Windows-native packaging; Windows remains a Linux-backed workflow through WSL or a remote Linux host for this slice.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must publish a concrete support matrix for the first installable Linux and macOS targets, including explicit operator boundaries for unsupported environments. | GOAL-01, GOAL-04 | must | The current platform story is too high-level to serve as a product install contract. |
| FR-02 | Port must define one canonical package artifact format and output layout for the supported targets so operators can install the `port` CLI without starting from a repo checkout. | GOAL-01, GOAL-02 | must | Packaging needs one source of truth before broader release automation can exist. |
| FR-03 | The repository must provide a repo-local packaging workflow that builds the supported install artifacts and documents the install steps and included files. | GOAL-02, GOAL-04 | must | The first installable slice is not credible unless maintainers can produce it repeatably inside the repo. |
| FR-04 | Port must make the macOS AVF launcher-helper and entitlement boundary explicit in the install and release contract without introducing a second macOS-only operator model. | GOAL-03, GOAL-04 | must | macOS is already a first-class lane, but its distribution boundary is still too implicit. |
| FR-05 | Release validation for the installable slice must remain anchored on canonical `just` and `port` commands, including board health and doctor checks. | GOAL-02, GOAL-04 | should | The release path should prove the real product workflow, not a disconnected packaging sidecar. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Package naming, included files, and install instructions must be deterministic for each supported target. | GOAL-01, GOAL-02 | must | Determinism keeps the first release contract auditable and testable. |
| NFR-02 | Unsupported targets or incomplete packaging prerequisites must fail fast with explicit guidance rather than silently falling back to a source-only workflow. | GOAL-01, GOAL-02, GOAL-03 | must | Hard cutover keeps the install contract understandable and prevents hidden compatibility paths. |
| NFR-03 | The first voyage must end with executable implementation stories and concrete verification commands, not another packaging roadmap placeholder. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | should | The mission goal is to convert research into executable board work immediately. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| support matrix and release contract | inspection + command proof | `README.md`, `RELEASE.md`, and related docs linked to the install workflow |
| packaging workflow | command proof + automated test | repo-local package build/install commands and tests for deterministic artifact metadata or layout |
| macOS AVF boundary | automated test + inspection | doctor/help output, docs, and explicit launcher/entitlement guidance |
| release validation path | command proof | `just mission`, `just doctor`, `just test`, and install-workflow proof commands |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A tarball-style CLI package is an acceptable first installable contract for both Linux and macOS. | The first slice may need target-specific installers sooner than planned. | Validate through the first voyage and operator-facing install docs. |
| Port can keep macOS as a first-class lane by documenting the launcher-helper and entitlement boundary before implementing a full signed app-distribution pipeline. | The macOS lane may still feel incomplete for some users. | Keep the boundary explicit and defer notarization/app-bundle work intentionally. |
| Repo-local `just` workflows are sufficient proof for the first packaging slice before CI-driven release automation exists. | Manual packaging may be too fragile or slow. | Prove determinism through build commands, tests, and release docs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first canonical package output be tarballs only, or tarballs plus a thin installer wrapper? | Maintainer / Product | Open |
| How far should checksum or signing work go in the first slice before automated release publication exists? | Maintainer | Open |
| Does the AVF launcher helper ship as a separate artifact, bundled sidecar, or documented external prerequisite for the first macOS release? | Maintainer / Runtime | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `README.md`, `RELEASE.md`, and the install-focused docs define a concrete first support matrix for Linux and macOS instead of a repo-only platform summary.
- [ ] The board contains executable work to build, package, and verify installable Port artifacts for the supported targets through one canonical workflow.
- [ ] The macOS AVF lane publishes a clear launcher-helper and entitlement boundary for distributed targets without adding a second macOS-specific operator surface.
- [ ] The first release-validation path is expressed through `just` and `port` commands plus board health checks, not a disconnected packaging-only checklist.
<!-- END SUCCESS_CRITERIA -->

