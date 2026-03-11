# Mac Operator Shell Compatibility - Software Design Description

> Keep the Port development shell usable on macOS by removing Linux-only package assumptions while preserving Linux launch tooling on Linux hosts.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage fixes a real operator breakage: the current `flake.nix` unconditionally
pulls Linux-only packages into the default dev shell, which prevents Darwin
evaluation. The design is intentionally small:

- keep one shared shell for repo tooling,
- gate Linux-only runtime packages behind `stdenv.isLinux`,
- optionally emit macOS shell guidance so operators understand the boundary,
- leave the Linux local-launch toolchain intact on Linux hosts.

## Context & Boundaries

### In Scope

- `flake.nix` package selection for `devShells.default`,
- Darwin evaluation proof,
- Linux shell preservation proof,
- minimal operator messaging for the macOS boundary.

### Out of Scope

- AVF runtime execution,
- macOS local Firecracker launch,
- changing the current Linux MVP execution requirements.

```
┌─────────────────────────────────────────┐
│              flake.nix                  │
│      shared repo shell contract         │
└───────────────────┬─────────────────────┘
                    │
      ┌─────────────┴─────────────┐
      │                           │
┌─────▼─────┐               ┌─────▼─────┐
│ Linux     │               │ macOS     │
│ includes  │               │ omits     │
│ runtime   │               │ Linux-only│
│ tools     │               │ tools     │
└───────────┘               └───────────┘
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `flake.nix` | repo config | shared development shell definition | repo current |
| `nixpkgs` platform metadata | package source | platform-aware package availability | nixos-unstable |
| Port operator docs | repo docs | explain shell boundary and Linux runtime requirements | repo current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Shell topology | Keep one default shell | Operators should not need separate flake entrypoints for Linux vs macOS |
| Linux-only inputs | Gate them with `stdenv.isLinux` | Prevents Darwin evaluation failures while preserving Linux tooling |
| macOS messaging | Keep it minimal and explicit | macOS users need to know the shell is for repo tooling, not local Firecracker launch |

## Architecture

`devShells.default` remains the only entrypoint. Its package list is split into:

- cross-platform repo tooling: Rust toolchain, `just`, `cargo-nextest`,
  `keel`, `curl`;
- Linux-only runtime tooling: `firecracker`, `iproute2`, `iptables`,
  `busybox`, `e2fsprogs`, and `mold`.

The shell hook remains shared, but macOS can emit a short guidance message
instead of attempting to provide Linux runtime packages.

## Components

- flake package selection:
  chooses which packages belong to every platform versus Linux only.
- shell hook:
  preserves the shared cargo target directory and can surface macOS guidance.
- operator docs:
  explain why the shell works on macOS even though Firecracker launch still
  requires Linux.

## Interfaces

- `nix develop` on Linux:
  unchanged entrypoint, still expected to provide local Firecracker tooling.
- `nix develop` on macOS:
  same entrypoint, but only for repo tooling and docs/tests that do not require
  Linux runtime packages.

## Data Flow

1. Nix evaluates `flake.nix` for the requested host platform.
2. The shell builds the shared package list.
3. Linux hosts receive the runtime toolchain additions.
4. macOS hosts skip those Linux-only packages and enter a reduced but valid
   repo shell.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Darwin evaluation fails on a Linux-only package | `nix eval .#devShells.aarch64-darwin.default.drvPath` fails | gate the package behind `isLinux` | re-run Darwin evaluation |
| Linux shell loses required launch tooling | Linux shell inspection no longer shows `firecracker`, `iproute2`, or `iptables` | reject the change | keep those packages in the Linux-only list |
| macOS shell implies local launch support | docs or shell hook message is absent or misleading | reject the change | add explicit macOS operator guidance |
