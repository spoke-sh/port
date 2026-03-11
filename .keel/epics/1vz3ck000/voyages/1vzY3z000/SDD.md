# PVM Host Kit And Artifact Delivery - Software Design Description

> Make the x86_64 Firecracker/PVM lane reproducible and operable through canonical artifact build, pull, push, validate, and hosted node-preparation workflows.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the existing prepared-node PVM story into a delivery lane.
The design keeps one canonical Port control model:

- model and runtime own one `PvmHostKit` package contract
- artifact commands own PVM kernel and guest-image lifecycle
- hosted node inventory owns prepared-host advertisement
- docs and help text publish one operator workflow that spans local artifact
  work and hosted node consumption

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                    This Voyage                              │
│                                                             │
│  artifacts build/pull/push/validate ──┐                    │
│                                        ├─> PVM package      │
│  hosted node prep/import ──────────────┘    contracts       │
│                                                 │           │
│  doctor / placement / launch consume the same contract      │
└─────────────────────────────────────────────────────────────┘
                 ↑                           ↑
          prepared host kit             hosted operator
```

### In Scope

- PVM host-kit package metadata and validation
- PVM artifact mobility on canonical Port commands
- hosted node-preparation/import and readiness advertisement
- operator docs and proof

### Out of Scope

- new guest protocol work
- `aarch64` Firecracker/PVM runtime claims
- Cloud Hypervisor or AVF implementation changes

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | workspace crate | canonical host-kit, node, and artifact contracts | workspace |
| `port-runtime` | workspace crate | artifact and hosted runtime behavior | workspace |
| `port-cli` | workspace crate | operator surface, help, and proofs | workspace |
| existing artifact scripts under `scripts/artifacts/` | local tooling | reproducible image and kernel variants | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Keep one PVM package contract | Model host-kernel, boot-line, and patched Firecracker as a named host kit instead of free-form notes | Hosted readiness and artifact mobility need a stable identifier |
| Reuse `port artifacts ...` | Extend the existing artifact surface rather than inventing `port pvm artifacts ...` | Keeps CLI coherent and discoverable |
| Keep hosted prep inventory-driven | Node preparation materializes into hosted inventory and node-agent state, not a separate registry | Hosted placement already consumes inventory and runtime state |
| Fail fast on unsupported architectures | `aarch64` stays explicitly research-only | Prevents accidental support claims |

## Architecture

The voyage adds a narrow vertical slice:

1. `port-model` grows PVM package metadata that can describe both host-kit and
   artifact-kit identities.
2. `port-runtime` resolves those packages into artifact pipeline commands,
   validation checks, and hosted node-preparation/import behavior.
3. `port-cli` exposes preparation and artifact operations through the existing
   command tree and documents the repo-local proof flow.
4. Hosted control-plane inventory and node-agent state use the same package
   identifiers for placement and operator inspection.

## Components

- `PvmHostKitPackage`
  - Purpose: define the canonical transportable description of a prepared PVM
    host kit.
  - Behavior: carries patched VMM identity, kernel package metadata, and boot
    requirements.
- PVM artifact mobility path
  - Purpose: build, validate, push, and pull `x86_64/firecracker/pvm` variants.
  - Behavior: forbids standard-lane fallback and records explicit variant
    outputs.
- Hosted node preparation/import path
  - Purpose: attach a real host kit to a hosted node or imported inventory
    record.
  - Behavior: upgrades node readiness from planned to ready only when the full
    package contract is present.
- Documentation and proof scripts
  - Purpose: make the lane discoverable and verifiable at the CLI level.
  - Behavior: provide deterministic repo-root proof commands and workflow docs.

## Interfaces

- `port artifacts build --artifact <name>`
- `port artifacts validate --artifact <name>`
- `port artifacts push --artifact <name>`
- `port artifacts pull --artifact <name>`
- hosted node-preparation/import CLI and control-plane inventory surfaces
- existing `port doctor` and hosted placement summaries

## Data Flow

1. Operator selects a PVM kernel or guest artifact variant.
2. Artifact runtime resolves the `x86_64/firecracker/pvm` variant contract and
   produces or validates outputs.
3. Operator prepares or imports a hosted node with a matching PVM host-kit
   package.
4. Hosted placement consumes node readiness and artifact variant availability.
5. `port doctor` and docs reflect the same package identities and readiness
   boundaries.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Operator targets `aarch64` Firecracker/PVM as ready | model/runtime validation | fail fast with research-lane guidance | use standard Firecracker on arm64 or stay in research lane |
| Host kit omits patched Firecracker or required boot-line entries | host-kit validation and doctor checks | reject prep/import and explain missing contract fields | repair host kit and re-run preparation |
| Artifact push/pull targets a missing PVM variant | artifact resolution | fail with explicit variant name | build or publish the PVM variant first |
| Hosted node remains planned instead of ready | placement summary and CLI proof | deny placement with node-specific reason | complete host-kit preparation and re-import |
