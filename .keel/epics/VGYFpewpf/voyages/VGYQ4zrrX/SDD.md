# Define Builder And Promotion Runtime Class Contracts - Software Design Description

> Establish the shared Port runtime-class contract that makes
> `workspace-scratch-builder` an explicit, inspectable lane without smuggling
> trust or publication policy into Port.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a first-class runtime-class vocabulary inside Port's
existing machine model. The new contract does three things:

1. names the runtime lane explicitly instead of relying on machine names
2. records trust posture and writable-state categories as machine metadata
3. exposes that metadata through Port-authored inspection surfaces

The voyage intentionally stops short of owning promotion policy or full
storage provisioning. It establishes the shared contract that later stories
and the adjacent promotion epic can execute against.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────┐
│                    This Voyage                            │
│                                                            │
│  machine model ──> runtime-class contract ──> CLI/status   │
│       │                   │                    surfaces     │
│       │                   └────> validation rules          │
│       │                                                    │
│       └────────────────────> builder lane identity         │
└────────────────────────────────────────────────────────────┘
          ↑                              ↑
      infra/spoke planning          later runtime execution
```

### In Scope

- shared runtime-class modeling in `port-model`
- builder-lane trust and writable-state contract
- reserved promotion-runner naming in the same contract surface
- operator inspection surfaces for machine launch and status

### Out of Scope

- build execution orchestration for builder workloads
- trusted publication logic or signing
- creator-facing policy or workspace admission semantics

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` machine configuration | internal model | carry the shared runtime-class contract | current workspace |
| `port-runtime` machine metadata/status structs | internal runtime | surface runtime-class identity in launch and status output | current workspace |
| `port-cli` machine output rendering | internal CLI | make the contract visible to operators | current workspace |
| `infra` builder/promotion planning contracts | external planning input | keep naming and trust boundaries aligned | current adjacent repo planning docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime-class location | Attach runtime-class metadata to `MachineSpec` rather than inventing a second top-level execution object. | Builder and promotion lanes still execute as machines in Port today; this keeps the first slice incremental. |
| Vocabulary | Add explicit `workspace-scratch-builder` and `blessed-closure-promotion-runner` kinds in one shared enum. | Downstream repos need one canonical name set, and the trust boundary depends on stable distinction. |
| Writable-state modeling | Represent writable state as named contract categories rather than as fully provisioned backend-specific mounts. | The first slice needs truthful semantics before every storage backend is implemented. |
| Inspection-first rollout | Surface runtime class in launch/status contracts immediately. | A runtime class that cannot be inspected is not a usable contract for downstream systems. |

## Architecture

The voyage touches three layers:

1. `port-model` gains runtime-class types, serialization, and validation.
2. `port-runtime` propagates the resolved runtime-class contract into machine
   launch and status metadata.
3. `port-cli` prints the runtime-class contract so operators can inspect the
   lane directly.

## Components

### Runtime-Class Types

- Purpose: define the shared contract surface for builder and promotion lanes.
- Interface: `MachineSpec`-attached metadata plus helper types for trust
  posture and writable-state categories.
- Behavior: deserialize from config, serialize back out, and provide a stable
  contract to the runtime and CLI layers.

### Validation Rules

- Purpose: prevent unsafe or incomplete builder declarations from silently
  entering the board.
- Interface: config validation inside `port-model`.
- Behavior: require runtime-class declarations to carry the expected trust
  posture and writable-state contract, and reject contradictory declarations.

### Machine Inspection Surfaces

- Purpose: show runtime-class identity in Port-authored operator output.
- Interface: launch metadata, machine status, and CLI rendering.
- Behavior: propagate resolved runtime-class fields into the returned machine
  data without changing the canonical `port machine ...` verbs.

## Interfaces

- Config surface: a machine declares a `runtime_class` block or equivalent
  runtime-class metadata
- Validation surface: `port doctor` and config validation reject contradictory
  runtime-class declarations
- Inspection surface: `port machine launch` and `port machine status` show the
  runtime class, trust posture, and writable-state contract

## Data Flow

1. Operator or downstream config loads a machine definition.
2. `port-model` resolves and validates the runtime-class declaration.
3. `port-runtime` carries the resolved runtime-class contract into machine
   launch metadata and status surfaces.
4. `port-cli` renders the runtime-class data so the operator can confirm the
   lane and posture.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Machine declares a runtime class but omits required trust or writable-state fields | config validation | fail fast with machine and field detail | repair the runtime-class declaration and rerun |
| Scratch builder declaration tries to imply trusted publication or admin credentials | config validation | reject the machine as unsafe | keep the machine untrusted and move trust material to the promotion lane |
| Runtime-class inspection data is missing from launch or status output | automated tests or CLI proof | keep story open; the contract is incomplete | wire the runtime-class contract through runtime structs and rendering |
| Downstream planning names drift from Port's runtime vocabulary | planning review | treat as contract drift and correct naming in Port before more execution work lands | update the shared enum and docs deliberately |
