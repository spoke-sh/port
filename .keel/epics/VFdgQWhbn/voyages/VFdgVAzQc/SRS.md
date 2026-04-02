# Mirror Keel And Sift Cargo-Dist Release Contract - SRS

## Summary

Epic: VFdgQWhbn
Goal: Port publishes cargo-dist installers and release artifacts through the same workflow shape as Keel and Sift so the CLI can upgrade through a canonical installer path.

## Scope

### In Scope

- [SCOPE-01] Add cargo-dist configuration, release automation, and release documentation for Port's supported release targets.
- [SCOPE-02] Implement `port upgrade` for latest-release installs and revision-pinned source installs.
- [SCOPE-03] Adapt installed asset lookup so released binaries can find bundled Port docs, examples, and scripts.

### Out of Scope

- [SCOPE-04] Expanding Port to unsupported native release targets.
- [SCOPE-05] Adding new hosting backends or package managers beyond the first cargo-dist release and upgrade flow.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a cargo-dist workspace config that mirrors the Keel/Sift release model while keeping Port's supported target matrix explicit. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Port must define a release GitHub Actions workflow that plans releases on pull requests and publishes artifacts when version tags are pushed. | SCOPE-01 | FR-01 | automated |
| SRS-03 | `port upgrade` without an explicit revision must install the latest released Port binary through the published shell installer contract. | SCOPE-02 | FR-02 | automated |
| SRS-04 | `port upgrade --tag <tag>` and `port upgrade --sha <sha>` must reuse a cached checkout under `~/.cache/port`, build with a supported local Rust toolchain, and install the built revision through the local installer path. | SCOPE-02 | FR-03 | automated |
| SRS-05 | Installed Port binaries must resolve bundled runtime assets from both the legacy packaged layout and the cargo-dist installer layout. | SCOPE-03 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Release and upgrade behavior must be covered by repository tests or scripted validation commands. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated |
| SRS-NFR-02 | Docs and help text must describe the supported release targets and upgrade behavior without overstating platform support. | SCOPE-01, SCOPE-02 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
