# Executable Avf Runtime Foundation - Software Design Description

> Ship the first local AVF launch, doctor, and guest attach workflow on macOS through the canonical Port command model.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the existing machine-driver seam so AVF becomes another
real local runtime owner instead of a model-only substrate. The design keeps
one command surface, one machine manifest contract, and one guest protocol
while adding macOS-specific doctor checks, AVF launch ownership, and AVF
console or transport metadata.

## Context & Boundaries

In scope:

- AVF local driver selection and macOS doctor checks
- canonical machine lifecycle for AVF-backed local VMs
- guest attach transport mapping and console/log capture contract
- operator docs and proof commands for the first local macOS lane

Out of scope:

- hosted macOS nodes
- AVF directory sharing or Rosetta convenience workflows beyond explicit docs
- any AVF/PVM work or Linux Firecracker substrate changes

```
┌─────────────────────────────────────────────────────┐
│           Executable Avf Runtime Foundation         │
│                                                     │
│  CLI/help/docs ──> machine driver ──> AVF runtime   │
│         │                    │             │         │
│         └──── doctor/macOS ──┴─ guest transport ───┘
└─────────────────────────────────────────────────────┘
                ↑                          ↑
         macOS/AVF APIs             guest protocol
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Apple Virtualization Framework | platform | local macOS virtualization substrate | Apple Virtualization APIs |
| Existing `MachineDriver` seam | internal | preserve one operator model across substrates | current Port runtime |
| Shared guest protocol crates | internal | reuse exec/copy/pty/logs/forward semantics | current Port guest protocol |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Driver ownership | add `AvfLocalDriver` behind the existing machine-driver trait | keeps AVF local without inventing a second runtime model |
| Guest transport | reuse the shared guest protocol over an AVF-specific host transport adapter | preserves canonical guest verbs |
| Console capture | record AVF serial output into the same runtime-root log surfaces used elsewhere | keeps `machine status` and operator inspection coherent |
| Non-macOS behavior | compile and validate the contract on all hosts, but fail fast outside macOS | keeps the repo shippable from Linux while making boundaries explicit |

## Architecture

The voyage touches five layers:

1. model validation for AVF-local machine selection and unsupported boundaries
2. macOS doctor checks for AVF availability and entitlement expectations
3. an AVF local driver in `port-runtime`
4. AVF guest attach plus console/log plumbing
5. CLI/docs/evidence updates for the macOS operator workflow

## Components

### AVF Driver Contract

- Purpose: launch AVF-backed Linux guests through the existing machine-driver seam.
- Interface: `MachineDriver::launch|status|stop|guest_endpoint`.
- Behavior: write canonical runtime manifests plus AVF-specific transport or serial metadata.

### macOS Doctor Layer

- Purpose: make AVF prerequisites and unsupported boundaries visible before launch.
- Interface: `port doctor` and `port --config ... doctor`.
- Behavior: report AVF API availability, entitlement/distribution boundaries, and unsupported-host detail.

### Guest Transport Adapter

- Purpose: bridge the existing guest protocol onto the AVF host transport.
- Interface: runtime guest endpoint resolution and transport connection helpers.
- Behavior: keep exec/copy/pty/logs/forward identical at the CLI and protocol level.

### Operator Surface

- Purpose: keep AVF discoverable and auditable for macOS operators.
- Interface: CLI help, README, `docs/avf.md`, and story proof scripts.
- Behavior: publish the native macOS workflow and explicit unsupported edges.

## Interfaces

- `MachineDriverKind::AvfLocal` and AVF driver selection in runtime
- AVF-focused doctor checks and machine-validation messages
- guest endpoint transport metadata for AVF-backed machines
- canonical `port machine ...` and `port guest ...` operator surfaces

## Data Flow

1. Operator targets an AVF machine through `port machine launch`.
2. Port validates that the machine selects AVF on macOS with `standard` protection.
3. `port doctor` and launch preflight resolve AVF-specific prerequisites or fail explicitly.
4. The AVF driver boots the VM, writes canonical runtime metadata, and exposes console plus guest transport endpoints.
5. `port guest ...` connects through the AVF transport adapter and reuses the existing guest protocol.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Non-macOS host selects AVF | model validation / doctor | fail fast with explicit macOS-only guidance | retarget machine or run on macOS |
| AVF APIs or entitlements unavailable | doctor / launch preflight | surface explicit AVF availability or entitlement failure | use a compatible macOS build/runtime |
| Guest transport cannot attach | AVF transport adapter | return attach-specific error with runtime context | inspect AVF listener state or relaunch |
| Firecracker regression while AVF lands | automated tests and proof scripts | block story completion | repair Linux lane before merge |
