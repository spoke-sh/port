# Release Matrix And Packaging Foundations - Software Design Description

> Define the first supported Linux and macOS release matrix, canonical package
> output, install workflow, and AVF distribution boundary so delivery can ship
> an installable Port surface.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the installable developer-experience bearing into one bounded
product slice. The first release contract will define a concrete Linux/macOS
support matrix, publish a single canonical tarball-style package format for the
`port` CLI, add a repo-local packaging workflow through `just`, and make the
macOS AVF launcher-helper plus entitlement boundary explicit in docs and
doctor/help surfaces. The design keeps the product surface small: one CLI,
one packaging contract, and one release-validation path.

## Context & Boundaries

### In Scope

- release/support matrix publication
- deterministic CLI package creation and install proof
- repo-local packaging orchestration through `just`
- AVF launcher-helper and entitlement boundary publication

### Out of Scope

- release automation or hosted publication
- native installers, Homebrew taps, or auto-update systems
- GUI wrappers or a second macOS-only workflow
- runtime substrate changes beyond doc/help/doctor boundary updates

```
┌─────────────────────────────────────────────────────────────────┐
│            Release Matrix And Packaging Foundations             │
│                                                                 │
│  release docs ─────┐                                            │
│                    ├──> just package / package script ───┐      │
│  doctor/help  ─────┘                                     │      │
│                                                          ├──> install proof
│  canonical port CLI <────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
                 ↑                                 ↑
            supported hosts                  extracted package
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Cargo workspace build | toolchain | build the release `port` binary for the selected target | current workspace |
| `just` workflow surface | toolchain | expose a canonical packaging and install-proof entrypoint | current repo `justfile` |
| shell packaging tools such as `tar` and checksum utilities | local tooling | create deterministic package archives and integrity metadata | dev shell / host shell |
| AVF runtime and doctor contract | internal docs + runtime | keep macOS packaging guidance aligned with the shipped AVF lane | current `docs/avf.md` and `port doctor` behavior |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First canonical package format | versioned tarball-style CLI package per supported target | bounded, cross-platform enough for Linux/macOS, and consistent with a repo-local first slice |
| Package ownership | package only the canonical `port` CLI plus explicit metadata/docs, not a second launcher app | preserves one operator model and keeps the slice smaller |
| macOS AVF helper strategy | document the helper as a required sidecar/precondition for AVF workflows, not as a bundled app-distribution system in this slice | keeps the first release credible without expanding into app-bundle/notarization work |
| Packaging orchestration | root `just` entrypoint backed by a small packaging script and deterministic staging directory | matches existing repo workflow patterns and keeps release proof auditable |

## Architecture

The voyage introduces four coordinated layers:

1. release-contract documents
2. package build/staging workflow
3. install-proof workflow
4. doctor/help/documentation boundary updates for macOS AVF

## Components

### Release Contract Documents

- Purpose: publish supported targets, artifact format, install steps, and
  release validation expectations.
- Interface: `README.md`, `RELEASE.md`, and any focused install/release docs
  added by the voyage.
- Behavior: define the support matrix and point operators to the canonical
  package and verification path.

### Packaging Workflow

- Purpose: build the `port` binary, stage deterministic package contents, and
  emit versioned artifacts.
- Interface: a root `just` recipe backed by a packaging script.
- Behavior: accept one supported target at a time, build the binary, create a
  stable directory layout, archive it, and report the resulting artifact path.

### Install Proof Workflow

- Purpose: prove the package is usable without repo-local cargo invocation.
- Interface: repo-local proof command and optional VHS tape.
- Behavior: extract or install the packaged artifact into a temporary prefix,
  run `port --version` and `port doctor`, and report success with the canonical
  binary path.

### AVF Packaging Boundary Surface

- Purpose: keep macOS install guidance aligned with the real AVF runtime
  contract.
- Interface: docs, help text, and `port doctor` messaging.
- Behavior: explain the launcher-helper expectation, distributed-target
  entitlement boundary, and unsupported-host failure guidance without changing
  the runtime ownership model.

## Interfaces

- `just package <target>`
- optional `just package-proof <target>`
- package archive output under a deterministic release-artifact directory
- `port --version`
- `port doctor`

## Data Flow

1. Maintainer selects a supported target from the published release matrix.
2. `just package <target>` invokes the packaging script.
3. The packaging script builds the release binary, stages deterministic package
   contents, writes integrity metadata, and emits the archive path.
4. The proof workflow extracts the archive into a temporary prefix.
5. The proof runs the packaged `port` binary with `--version` and `doctor`.
6. Docs and release checklist link the resulting package workflow back to the
   canonical validation path.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| unsupported target requested | target validation before build | fail fast with the published support matrix and accepted targets | choose a supported target or update planning intentionally |
| required packaging tool missing | script/tool spawn failure | fail with explicit missing-tool guidance | install the tool in the dev environment and rerun |
| built artifact layout is incomplete or non-deterministic | packaging validation or test | fail the proof and report missing or unexpected files | fix the staging logic and rerun packaging |
| macOS AVF operator expects a bundled helper that is not shipped in this slice | doc review or install-proof gap | document the launcher-helper as a required sidecar and make that boundary explicit in release docs | plan a follow-on story or epic if bundled helper packaging becomes necessary |

## Story Decomposition

1. [VDchsxHfp](../../../../stories/VDchsxHfp/README.md) publishes the support
   matrix and release contract.
2. [VDchsvWfn](../../../../stories/VDchsvWfn/README.md) implements the
   deterministic package build workflow.
3. [VDchsxHfm](../../../../stories/VDchsxHfm/README.md) adds the install proof
   and recording path for packaged Port.
4. [VDchsw9fh](../../../../stories/VDchsw9fh/README.md) aligns docs and doctor
   output with the AVF distribution boundary.
