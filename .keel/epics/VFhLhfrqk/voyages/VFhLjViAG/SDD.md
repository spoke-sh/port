# Export And Prove AWS PVM Host Kit Module - Software Design Description

> Publish a downstream-consumable Nix module and supporting package metadata for
> the AWS x86_64 PVM host kit, then document and verify the downstream AMI
> build path.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage exports Port's AWS x86_64 PVM host contract through the flake
itself instead of leaving the contract stranded in runtime code, sample config,
and docs only.

The design has three parts:

1. A canonical Nix host-kit helper derived from Port-owned contract data.
2. A downstream-consumable NixOS module and companion package surface.
3. Docs and proof showing how downstream AMI builders consume that surface.

## Context & Boundaries

### In Scope

- flake exports for `nixosModules.aws-pvm-host`
- a companion `firecracker-pvm-host-kit` package/metadata surface
- module config for the AWS PVM host contract
- docs showing the downstream AMI build handoff

### Out of Scope

- AWS VM Import/Export automation
- downstream AMI publication and consumption policy
- non-AWS or arm64 host-kit exports
- broader platform/bootstrap orchestration

```text
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ Canonical    │  │ Flake Exports   │ │
│  │ host-kit     │->│ module + pkg    │ │
│  │ metadata     │  └─────────────────┘ │
│  └──────────────┘           |          │
│                             v          │
│                    ┌─────────────────┐ │
│                    │ Docs + proof    │ │
│                    └─────────────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   Port model       Downstream AMI builders
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `examples/port.toml` | Repo contract | Source of the canonical AWS PVM host-kit identity already modeled by Port | repo-local |
| `flake.nix` | Repo build surface | Exposes packages and modules to downstream consumers | repo-local |
| Existing PVM doctor/readiness contract in `crates/port-model` and `crates/port-runtime` | Runtime contract | Defines the identity and checks the Nix surface must align with | repo-local |
| Downstream `infra` AMI build seam | External consumer | Consumes the exported module/package path without Port taking over AMI automation | current downstream contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical host-kit source | Derive the Nix host-kit metadata from Port-owned canonical data instead of duplicating values ad hoc | Reduces drift between Nix exports and `prepare-pvm-node` identity |
| Downstream consumption surface | Export both a NixOS module and a companion package containing the canonical `firecracker-pvm` path and host-kit metadata | Gives downstream repos a stable import surface and a stable module/package path for handoff tooling |
| Scope boundary | Keep the module focused on the Port-owned AWS x86_64 PVM host contract and leave AMI import/export and downstream orchestration outside Port | Matches the upstream request and prevents scope creep |

## Architecture

The host-kit export is structured as:

- Nix helper/library: resolves canonical AWS PVM host-kit metadata
- host-kit package: installs the canonical `firecracker-pvm` binary surface and
  packaged metadata for downstream use
- NixOS module: applies the AWS PVM host contract to a downstream host
- flake exports: publish the module/package to downstream consumers
- docs: explain direct flake import and the current `infra` handoff

## Components

### Canonical Host-Kit Metadata Helper

- Purpose: keep Nix exports aligned with the existing AWS PVM host-kit contract.
- Interface: imported by both the companion package and the NixOS module.
- Behavior: resolves package identity, boot args, and binary naming from
  Port's canonical contract.

### `firecracker-pvm-host-kit` Package

- Purpose: ship a downstream-consumable package surface that contains the
  canonical `firecracker-pvm` path and host-kit metadata.
- Interface: exported under `packages.<system>.firecracker-pvm-host-kit`.
- Behavior: exposes the binary surface and packaged metadata/path that
  downstream tooling can reference.

### `aws-pvm-host` NixOS Module

- Purpose: apply the Port-owned AWS x86_64 PVM host contract to a NixOS system.
- Interface: exported under `nixosModules.aws-pvm-host`.
- Behavior: configures boot args, binary path/env surface, and host-kit
  metadata while remaining focused on Port's host contract rather than infra
  automation.

### Docs And Proof

- Purpose: show downstream consumers how to use the exported surface.
- Interface: foundational AWS/Nix docs.
- Behavior: documents direct flake import and the current AMI-build handoff via
  the Port-owned module/package path.

## Interfaces

| Interface | Producer | Consumer | Contract |
|-----------|----------|----------|----------|
| `nixosModules.aws-pvm-host` | Port flake | downstream `nixosSystem` | importable module surface |
| `packages.<system>.firecracker-pvm-host-kit` | Port flake | downstream image builders and operators | package path, binary path, and packaged host-kit metadata |
| Updated AWS/Nix docs | Port docs | downstream maintainers | supported handoff from Port flake export to AMI build tooling |

## Data Flow

1. Port resolves canonical AWS PVM host-kit metadata from its own contract data.
2. The host-kit package and module both consume that metadata.
3. The flake exports the module/package to downstream users.
4. Downstream Nix systems import `port.nixosModules.aws-pvm-host`.
5. Downstream image builders reference the Port-owned module/package path in the
   existing AMI build pipeline.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Canonical host-kit metadata drifts from the Port contract | Verification of exported metadata against canonical values | Fail the proof and keep the story open | Update the Nix helper so module/package exports match the canonical contract |
| Module import does not evaluate downstream | Nix evaluation proof fails | Keep the flake export incomplete and block closure | Fix module defaults/exports until a plain `nixosSystem` import evaluates |
| Docs still point users at repo-local custom modules | Manual doc review | Keep the story open | Rewrite docs around the Port-owned module/package handoff |
