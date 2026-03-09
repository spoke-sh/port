# Enable Hosted Standard Cloud Launch - Software Design Description

> Route the sample `generic-linux`, `aws`, and `gcp` standard Firecracker lanes through the live hosted control-plane and node-agent path so remote cloud launch becomes executable instead of guidance-only.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage reuses the existing hosted control-plane and node-agent split
instead of inventing a second remote-launch system. The new work is to extend
placement and launch admission so standard provider-backed machines can resolve
onto registered nodes, then let the control plane ask the selected node agent
to localize and launch the machine under that node's runtime root. The canonical
`machine status|stop` flow remains the same surface, but now follows the stored
hosted placement for the standard lane instead of stopping at provider-aware
guidance.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────┐
│                  Hosted Standard Cloud Launch              │
│                                                            │
│  CLI/SDK ──> control-plane placement ──> node selection    │
│      │                           │                         │
│      └──── status/stop follow stored placement ───────────┤
│                                  │                         │
│                         node-agent localized launch        │
│                                  │                         │
│                         Firecracker runtime root           │
└────────────────────────────────────────────────────────────┘
             ↑                                  ↑
      configured provider host           registered hosted node
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing hosted control-plane HTTP routes | internal | Reuse authenticated launch, status, stop, and guest transport instead of introducing a second remote runtime API. | current workspace |
| Existing node-agent runtime owner | internal | Own hypervisor processes and runtime roots on the selected execution host. | current workspace |
| Sample hosted node config in `examples/port.toml` | config | Provides provider-aware remote hosts, nodes, and machine samples for proof. | repository sample |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Remote standard launch path | Reuse hosted control plane and node agent instead of direct provider-host CLI ownership | The hosted runtime split already owns placement, runtime roots, and guest routing. |
| Provider identity | Keep `generic-linux`, `aws`, and `gcp` explicit in placement and output | Cost-control and operator routing depend on provider-specific visibility. |
| Canonical surface | Keep `port machine ...` and update docs/help instead of adding a hosted-only machine command family | Matches Port's CLI model and avoids a second discovery surface. |

## Architecture

- `port-model`
  - continues to represent provider-backed hosts, hosted nodes, and machine
    execution-lane selection
  - exposes placement-ready summaries for standard-lane hosted machines
- `port-runtime`
  - extends hosted placement selection to admit standard provider-backed lanes
  - routes hosted standard launch through the control-plane client and node
    launch contract
  - preserves local and PVM launch branches
- hosted control plane
  - validates placement and forwards launch requests to the selected node agent
  - records selected-node inventory so later `status|stop` calls follow the
    same owner
- node agent
  - localizes the selected machine into its runtime root and reuses the shared
    local launcher for standard Firecracker
- CLI/docs
  - update operator help and workflow text from denial-only to executable hosted
    launch

## Components

- Placement summary
  - purpose: resolve candidate and selected nodes for standard provider-backed
    machines
  - interface: runtime helper used by `machine launch|status|stop`
  - behavior: returns explicit routing context for both success and rejection
- Hosted launch client
  - purpose: send canonical launch requests to the control plane for the
    standard lane
  - interface: existing hosted control-plane client path
  - behavior: preserves machine, host, provider, and selected-node metadata
- Node runtime localizer
  - purpose: materialize one provider-backed standard machine under the
    selected node's runtime root
  - interface: existing node-agent launch handler
  - behavior: reuses the local Firecracker launcher instead of inventing a
    second hypervisor contract

## Interfaces

- `machine launch`
  - input: canonical machine name for a provider-backed standard lane
  - output: hosted routing detail including control plane, provider host, and
    selected node
- hosted control-plane launch contract
  - input: machine spec plus resolved placement summary
  - output: runtime metadata and selected-node ownership detail
- `machine status|stop`
  - input: machine name
  - output: hosted inventory plus selected-node runtime detail using stored
    placement

## Data Flow

1. CLI resolves the named machine and determines it targets a provider-backed
   remote Linux host on the standard Firecracker lane.
2. Runtime placement logic resolves candidate hosted nodes and either rejects
   with explicit routing detail or chooses one node.
3. The hosted control plane receives the launch request and forwards it to the
   selected node agent.
4. The node agent localizes the machine to its runtime root and reuses the
   shared local Firecracker launcher.
5. The control plane records selected-node ownership so later `status|stop`
   calls route back to the same node runtime.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| No eligible node for the provider-backed standard machine | placement summary returns empty or only rejected candidates | fail with machine, host, provider, control plane, and candidate-node detail | operator updates node registration or machine placement inputs |
| Selected node is registered but cannot launch | control plane or node-agent launch returns error | surface selected node and runtime-root detail without falling back locally | fix node runtime prerequisites and retry |
| Stored placement is stale during status or stop | hosted inventory points at a missing node binding | return explicit hosted routing failure | re-register node or relaunch to refresh placement |
