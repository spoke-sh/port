# Prepared Linux Pvm Runtime - Software Design Description

> Ship the first executable x86_64 Firecracker/PVM runtime path on prepared Linux nodes through the canonical Port model, CLI, and hosted node-agent ownership.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends Port's existing Linux and hosted runtime seams rather than
inventing a separate protected-runtime stack. The design adds a prepared-node
PVM host-kit contract, teaches the node agent to select it during launch, and
replaces hosted provider guidance with live control-plane-to-node launch when a
machine is admission-ready.

## Context & Boundaries

### In Scope

- host-kit selection and validation for x86_64 PVM
- node-agent launch ownership for prepared Linux nodes
- hosted control-plane launch routing to prepared nodes
- CLI/docs/evidence updates for the prepared-node operator workflow

### Out of Scope

- arm64 Firecracker/PVM
- AVF runtime implementation
- generalized scheduler policy beyond explicit prepared-node targeting

```
┌──────────────────────────────────────────────────────┐
│              Prepared Linux PVM Runtime             │
│                                                      │
│  CLI/help/docs ──> control plane ──> node agent     │
│         │                               │            │
│         └──── artifact + host-kit model ┴─> launch  │
└──────────────────────────────────────────────────────┘
                 ↑                         ↑
            prepared host kit         PVM artifacts
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Prepared host kit | runtime | supplies patched Firecracker binary and host prerequisites | Port-defined contract |
| Existing hosted control split | internal | reuses control-plane and node-agent ownership | current Port runtime |
| PVM artifact variants | artifact | kernel and guest-image inputs for launch | current artifact catalog |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| PVM launch owner | node agent on prepared Linux host | preserves hosted runtime ownership model |
| PVM admission gate | existing hosted placement plus explicit host-kit validation | reuses current control-plane semantics |
| Standard-lane preservation | keep standard Firecracker path live in all proofs | prevents hidden regressions while adding PVM |

## Architecture

The voyage touches four components:

1. shared model and runtime validation for prepared PVM host kits
2. node-agent launch path for prepared-host Firecracker/PVM
3. hosted control-plane launch routing for admission-ready PVM machines
4. operator-facing CLI/help/docs and proof scripts

## Components

### Host-Kit Contract

- Purpose: describe the prepared binary and host prerequisites Port needs for
  executable PVM launch.
- Interface: shared model plus doctor/runtime validation surfaces.
- Behavior: selects the patched PVM Firecracker binary and fails fast when the
  prepared kit is absent.

### Node-Agent PVM Launch

- Purpose: own the actual PVM launch on the prepared Linux node.
- Interface: hosted node-agent machine-launch route and local runtime helpers.
- Behavior: launches with PVM-specific binary and artifacts while writing the
  same canonical runtime manifests.

### Hosted PVM Launch Routing

- Purpose: replace remote guidance for admission-ready PVM machines.
- Interface: control-plane machine-launch route and SDK/CLI request path.
- Behavior: routes only to prepared nodes that passed placement and host-kit
  selection.

### Operator Proof Surface

- Purpose: keep the workflow discoverable and auditable.
- Interface: CLI help, docs, repo-local scripts, and recorded evidence.
- Behavior: proves both prepared-node PVM launch and preserved standard launch.

## Interfaces

- shared model: prepared-node PVM host-kit and capability contract
- control plane: hosted machine launch route for admission-ready PVM machines
- node agent: prepared-host launch command path
- CLI/docs: canonical `port machine launch` and `port doctor` workflow

## Data Flow

1. Operator targets a hosted or local PVM machine through the canonical CLI.
2. Port resolves the machine, artifacts, and prepared-host requirements.
3. Hosted control-plane routing selects a prepared node that already passed
   placement and capability checks.
4. The node agent launches Firecracker/PVM with the prepared host kit and PVM
   artifacts.
5. Port records the same runtime ownership metadata used by the standard lane.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Host kit missing | doctor/runtime validation | fail fast with explicit host-kit detail | prepare host or retarget machine |
| Hosted node not PVM-ready | placement and launch route | reject launch before node execution | fix node inventory or choose prepared node |
| Standard lane regression | proof scripts and tests | block story completion | repair standard path before merge |
