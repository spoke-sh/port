# Foundation And Hosted Cloud Hypervisor Lane - Software Design Description

> Model, launch, and document the first Cloud Hypervisor standard lane through Port's canonical CLI, runtime driver boundary, and hosted control path.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes Cloud Hypervisor a real executable substrate without changing
Port's operator model. The same `machine`, `guest`, `doctor`, and hosted
control-plane verbs remain canonical. The work fits behind the existing
`MachineDriver` boundary, the shared artifact selectors, and the shared guest
protocol.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────────┐
│                       This Voyage                           │
│                                                              │
│  port-model  ──>  driver selection  ──>  cloud-hypervisor    │
│      │                 │                    local driver      │
│      │                 │                         │            │
│      └────> hosted control plane ──> node agent ─┘            │
│                                │                              │
│                                └──> shared guest protocol     │
└──────────────────────────────────────────────────────────────┘
          ↑                        ↑                    ↑
      Linux host             kernel/guest artifacts   docs/help
```

In scope:

- Cloud Hypervisor `standard` lane on Linux
- local and hosted lifecycle ownership
- guest transport parity using the existing Port guest protocol
- docs/help/example updates

Out of scope:

- Cloud Hypervisor confidential or protected modes
- new guest APIs
- macOS-native Cloud Hypervisor execution

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `cloud-hypervisor` host binary | external binary | launches local and hosted Cloud Hypervisor VMs | current host-installed binary |
| Linux KVM plus host prerequisites | platform | runs Cloud Hypervisor guests | host kernel API |
| Port kernel and guest-image artifacts | repo/runtime contract | boots Cloud Hypervisor machines with explicit substrate variants | current Port artifact model |
| Existing hosted control-plane and node-agent HTTP contracts | internal | routes hosted lifecycle and guest traffic | `port-hosted-protocol` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Substrate modeling | Reuse the current `architecture/substrate/protection_mode` selectors with `substrate = "cloud-hypervisor"` and `protection_mode = "standard"` | Keeps one artifact and machine vocabulary across substrates |
| Local ownership | Reuse the `MachineDriver` seam with a `CloudHypervisorLocalDriver` | The Firecracker and AVF work already established the correct runtime ownership seam |
| Guest transport | Reuse the existing Port guest protocol and host-side attach abstraction, with a Cloud Hypervisor-specific host transport adapter if needed | Avoids inventing a substrate-specific guest API |
| Hosted routing | Keep hosted control-plane routes unchanged and select Cloud Hypervisor through machine and node metadata | Hosted product shape should not fork per hypervisor |
| Stop/status behavior | Start with Port-owned runtime manifests plus process ownership, adding Cloud Hypervisor API usage only if it materially improves correctness | Keeps the first slice incremental and verifiable |

## Architecture

The voyage touches five layers:

1. `port-model` and examples gain executable Cloud Hypervisor substrate
   contracts and sample machine/host declarations.
2. `port-runtime` gains Cloud Hypervisor driver selection, local preflight, and
   launch/status/stop behavior.
3. Guest attach logic gains a Cloud Hypervisor transport path while preserving
   the shared request/response protocol.
4. Hosted control-plane and node-agent launch logic reuse the same runtime
   substrate selection and guest attach path.
5. CLI help and docs publish the new lane coherently.

## Components

### Model and Artifact Selection

- Purpose: make Cloud Hypervisor selectable in config, examples, and artifact
  resolution.
- Interface: existing model enums and runtime artifact lookup.
- Behavior: select only Cloud Hypervisor variants when a Cloud Hypervisor
  machine is requested; fail when those variants are absent.

### Local Cloud Hypervisor Driver

- Purpose: own local launch, status, stop, and doctor/preflight checks.
- Interface: `MachineDriver` trait plus runtime manifest files.
- Behavior: validate Linux/KVM plus `cloud-hypervisor` availability, assemble
  launch arguments, start the hypervisor, and persist runtime metadata for later
  status/stop and guest attach.

### Guest Transport Adapter

- Purpose: let Cloud Hypervisor guests speak the same Port guest protocol.
- Interface: runtime guest-attach functions and `port-agent-protocol`.
- Behavior: bridge host-side Cloud Hypervisor guest transport onto the canonical
  runtime guest socket/stream ownership expected by the CLI and hosted layers.

### Hosted Routing

- Purpose: execute the same Cloud Hypervisor lane through the control-plane and
  node-agent split.
- Interface: existing hosted machine routes plus node inventory/runtime root
  ownership.
- Behavior: placement and launch select Cloud Hypervisor-capable nodes, then
  reuse the same runtime driver and guest attach path on the node.

### Operator Surfaces

- Purpose: make the new lane discoverable and operable.
- Interface: `port --help`, README, `docs/cloud.md`, `docs/operators.md`, and
  example config.
- Behavior: publish one local workflow and one hosted workflow plus explicit
  unsupported boundaries.

## Interfaces

- Machine config: `substrate = "cloud-hypervisor"` with `protection_mode =
  "standard"`
- Artifact variants: selected through the existing
  `architecture/substrate/protection_mode` tuple
- Hosted routes: reuse current `/v1/machines/...` machine and guest route
  families without adding a Cloud Hypervisor-specific route namespace
- Runtime state: persist the same manifest-oriented runtime ownership used by
  Firecracker and AVF so `machine status`, `stop`, and guest attach stay
  coherent

## Data Flow

### Local flow

1. CLI resolves the machine and artifact variants.
2. Runtime selects `CloudHypervisorLocalDriver`.
3. Driver performs preflight and launches the hypervisor.
4. Runtime writes manifest and process metadata.
5. Guest verbs attach through the Cloud Hypervisor transport adapter and speak
   the shared guest protocol.

### Hosted flow

1. CLI sends machine or guest request to the control plane.
2. Control plane selects a Cloud Hypervisor-capable node from current hosted
   inventory.
3. Node agent launches or inspects the machine using the same runtime driver.
4. Guest routes proxy through control plane to node agent and then through the
   same guest transport adapter.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Host lacks `cloud-hypervisor` binary or KVM prerequisites | doctor or local preflight | return substrate-specific failure with host detail | install binary or move to compatible host |
| Cloud Hypervisor artifact variant is missing | artifact lookup | fail with selected variant detail and no Firecracker fallback | build or pull the matching variant |
| Guest transport bridge is unavailable | runtime guest attach | return machine/runtime-root context plus Cloud Hypervisor transport detail | inspect runtime artifacts or restart machine |
| Hosted node is not eligible for Cloud Hypervisor | placement | return rejected-node detail instead of generic unsupported-host output | register or prepare a suitable node |
| Operator requests unsupported protection mode or host OS | model/runtime validation | fail fast with explicit boundary | choose supported substrate or host |
