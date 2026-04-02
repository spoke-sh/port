# Mirror Keel And Sift Cargo-Dist Release Contract - Software Design Description

> Port publishes cargo-dist installers and release artifacts through the same workflow shape as Keel and Sift so the CLI can upgrade through a canonical installer path.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage gives Port the same release backbone used by Keel and Sift: cargo-dist
defines release artifacts and installer scripts, GitHub Actions executes the
tag-driven release flow, and the CLI grows one `port upgrade` command that uses
that installer surface for both released binaries and locally built revisions.

## Context & Boundaries

### In Scope

- release metadata and workflow files required for cargo-dist
- CLI upgrade logic in `port-cli`
- installed-asset lookup changes needed for cargo-dist layouts
- release and install docs

### Out of Scope

- making Port portable to unsupported native targets
- introducing a second installer family outside cargo-dist
- redesigning unrelated machine, guest, or hosted workflows

```
┌───────────────────────────────────────────────────────────────┐
│                         This Voyage                           │
│                                                               │
│  cargo-dist config  ->  GitHub release workflow               │
│         │                         │                            │
│         v                         v                            │
│  release archives/installers  <-  `port upgrade`              │
│         │                         │                            │
│         └──── installed asset lookup adapts both layouts ─────┘
└───────────────────────────────────────────────────────────────┘
             ↑                               ↑
      GitHub Releases                 Local Rust toolchain
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| cargo-dist | release tooling | Generate release plans, archives, and installer artifacts | 0.30.x line to match Keel |
| GitHub Actions | CI/hosting | Execute plan/build/host release phases on tags and PRs | Repo workflow API |
| rustup toolchain | local build toolchain | Build requested tags or SHAs during `port upgrade` source installs | Compatible with workspace rust-version |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Dist config shape | Use a root `dist-workspace.toml` patterned after Keel rather than older inline metadata | This matches the newer release contract already used in spoke-sh projects. |
| Supported targets | Keep Port's release matrix to the targets the current codebase already supports | Release automation must not promise unsupported binaries. |
| Upgrade install path | Default upgrades consume the published shell installer; source installs build a local artifact then reuse local installer logic | One installer contract keeps release and source installs aligned. |
| Asset lookup | Support both legacy `share/port/...` paths and cargo-dist root-level include paths | This lets existing proof flows coexist with new release installers. |

## Architecture

The work spans three layers:
- repository release metadata: `Cargo.toml`, `dist-workspace.toml`, workflow files, docs
- CLI orchestration: `port-cli` parses and executes `port upgrade`
- runtime asset discovery: `port-runtime` resolves scripts and examples from either packaged layout

## Components

- `dist-workspace.toml`
  Purpose: define targets, installers, hosting, and included runtime assets.
- `.github/workflows/release.yml`
  Purpose: run plan, build, and host release phases with the same shape as Keel.
- `port upgrade`
  Purpose: install the latest release or a requested git revision through one user-facing command.
- asset resolution helpers in `port-runtime`
  Purpose: keep installed binaries functional across both archive layouts.

## Interfaces

- CLI:
  `port upgrade`
  `port upgrade --tag <tag>`
  `port upgrade --sha <sha>`
- Installer contract:
  published shell installer for latest release
  local installer helper for source-built revisions
- Cache contract:
  source checkout and build artifacts rooted under `~/.cache/port`

## Data Flow

1. Maintainer pushes a version tag.
2. GitHub Actions runs cargo-dist plan, build, and host phases and publishes Port artifacts plus installer scripts.
3. Operator runs `port upgrade`; the CLI resolves the requested mode.
4. For latest-release installs, the CLI downloads and executes the published installer script.
5. For tag/SHA installs, the CLI refreshes `~/.cache/port`, checks out the requested revision, locates a supported toolchain, builds Port, and installs the result through the local installer path.
6. Installed Port binaries resolve bundled assets from either `share/port/...` or dist-style root includes.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| cargo-dist config does not match the supported matrix | plan/build validation fails | Stop the release path with actionable errors | Fix config and docs before tagging |
| No supported Rust toolchain is available for a source install | toolchain detection fails | Return a clear CLI error that names the required toolchain or target | Install the matching toolchain and rerun |
| Requested tag or SHA cannot be fetched | git fetch or checkout fails | Return a revision-specific error | Correct the ref or network access and retry |
| Installed binary cannot find packaged assets | focused tests or runtime checks fail | Broaden asset search to the new layout before shipping | Keep both layouts supported in code |
