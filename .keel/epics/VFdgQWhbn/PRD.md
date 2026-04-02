# Ship Cargo-Dist Release And Upgrade Path - Product Requirements

## Problem Statement

Port needs the same cargo-dist release contract as Keel and Sift so published installers exist and the CLI can upgrade through one canonical release path.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Publish Port through a tag-driven cargo-dist release flow aligned with Keel and Sift. | A release tag can produce archives and installer artifacts for every supported Port target without manual packaging steps. | One cargo-dist workflow and config checked into the repo |
| GOAL-02 | Give operators a canonical self-upgrade path for Port. | `port upgrade` can install the latest release or a requested git revision through the same installer contract. | One CLI command with automated coverage for release and source-install flows |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | A Port user who installs the CLI directly from release artifacts. | A stable install and upgrade path that does not require a repo checkout. |
| Contributor | A Port maintainer or evaluator working from source. | A reproducible way to build and install a pinned Port revision locally. |

## Scope

### In Scope

- [SCOPE-01] Cargo-dist configuration, GitHub release automation, and release documentation for Port's supported target matrix.
- [SCOPE-02] A `port upgrade` command that installs the latest release by default and can build/install a tag or git SHA from a cached checkout.
- [SCOPE-03] Runtime asset resolution changes required so cargo-dist-installed binaries can find bundled Port docs, scripts, and examples.

### Out of Scope

- [SCOPE-04] Expanding Port to new release targets beyond the currently supported Linux and macOS matrix.
- [SCOPE-05] New hosting backends or package managers beyond the cargo-dist release artifacts needed for the first install and upgrade flow.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define a cargo-dist release configuration and GitHub workflow that publish archives and installer artifacts through the same tag-driven release model used by Keel and Sift. | GOAL-01 | must | This establishes the canonical release surface Port currently lacks. |
| FR-02 | Port must expose a `port upgrade` command that installs the latest released CLI through the release installer script when no explicit revision is requested. | GOAL-02 | must | Operators need one default upgrade path that matches the release contract. |
| FR-03 | Port must allow `port upgrade` to install a requested git tag or git SHA by reusing a cached checkout, building with a supported local Rust toolchain, and installing the result through the installer contract. | GOAL-02 | must | Contributors need a revision-pinned upgrade path without ad hoc local scripts. |
| FR-04 | Installed Port binaries must continue to locate bundled runtime assets needed by released workflows. | GOAL-01, GOAL-02 | must | Release installers are not useful if the installed CLI cannot find its packaged assets. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The release configuration and CLI upgrade flow must be verified with automated tests or scripted proof paths that run in the repository. | GOAL-01, GOAL-02 | must | Release and upgrade paths need proof before operators depend on them. |
| NFR-02 | Port's release docs, install docs, and CLI help must describe the shipped support matrix accurately. | GOAL-01, GOAL-02 | must | Release automation must not overstate platform support. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Release automation | Cargo-dist plan/build validation plus workflow file review | Story-level test output and release config diffs |
| Upgrade flow | CLI tests covering release-install and source-build installs | Automated test evidence linked from the implementing story |
| Installed behavior | Packaged/runtime asset resolution checks from an installed binary layout | Scripted proof or focused test coverage |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Port's current supported release matrix remains Linux and macOS for this slice. | Windows-native installers would require more build and runtime work than this epic can safely absorb. | Preserve the matrix explicitly in docs, config, and tests. |
| Cargo-dist's installer layout can be adapted to Port's runtime asset lookup without changing the operator-facing CLI vocabulary. | The release flow may need a larger packaging redesign. | Validate installed asset resolution in tests before shipping. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether Port should later add Windows-native release targets once the CLI and runtime become portable there. | Epic owner | Deferred |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A release tag can drive cargo-dist planning, build, and hosting for Port's supported targets using checked-in config and workflow files.
- [ ] `port upgrade` installs the latest release through the installer contract and can install a requested tag or git SHA from a cached local source build.
- [ ] Installed Port binaries continue to find bundled release assets needed by supported workflows.
<!-- END SUCCESS_CRITERIA -->
