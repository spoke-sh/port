# Registered Nodes And Machine Launch Placement - Software Design Description

> Let a node agent register with the hosted control plane and route canonical machine launch onto an eligible registered node with operator-visible placement detail.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the hosted machine lane from an explicit-binding demo into
the first registered-fleet workflow. The control plane keeps the existing
machine and guest vocabulary, but now owns a persisted registered-node view.
Node agents register themselves into that view, machine launch chooses a
registered eligible node deterministically, and canonical machine output
surfaces where placement landed or why it failed.

## Context & Boundaries

### In Scope

- registration state stored by the control plane
- node-agent registration and refresh behavior
- machine placement through registered hosted nodes
- operator-visible placement detail in `port machine` output and docs

### Out of Scope

- full cluster management or autoscaling
- richer fleet scoring and rebalance policy
- external service discovery or non-config node catalogs
- replacing the current hosted service scheduler in this voyage

```
CLI / SDK
   |
   v
control-plane machine routes
   |
   +--> registered node inventory
   |
   +--> deterministic machine placement
   |
   v
selected node-agent runtime owner
   |
   v
local machine driver + guest transport
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | crate | carry registered-node state, placement metadata, and machine-facing contracts | workspace |
| `port-hosted-protocol` | crate | extend hosted control-plane and node-agent route vocabulary for registration | workspace |
| `port-runtime` | crate | own registration persistence, placement, and canonical machine routing | workspace |
| `port-cli` | crate | keep placement on canonical machine verbs and publish help/docs evidence | workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Preserve model-backed node identity | Registered nodes still map to configured node names and capabilities | Keeps the first fleet slice understandable and avoids a second inventory taxonomy |
| Let node agents self-register | `node-agent serve` publishes its endpoint/token/freshness to the control plane | Removes the largest manual hosted demo step without building a full external registry |
| Keep deterministic first-fit placement | Registered nodes are filtered for freshness/capability and selected in stable order | Matches the service-scheduler precedent and gives repeatable operator evidence |
| Surface placement through machine verbs | Reuse `machine launch|list|status|monitor|stop` instead of hosted-only placement commands | Keeps Port CLI-first and coherent with local lanes |

## Architecture

The implementation extends the current hosted machine flow:

1. `port node-agent serve` registers one node against the hosted control plane
   and refreshes that registration while it is live.
2. The control plane persists registered-node state alongside the configured
   hosted inventory contract.
3. Hosted `machine launch` resolves eligible nodes from the intersection of
   configured inventory and live registrations.
4. Deterministic placement selects one node and routes the existing launch path
   through that node agent.
5. Machine status/list/monitor/stop read placement metadata and registered-node
   freshness detail back through the canonical machine surface.

## Components

- `port-model`
  - add registered-node and placement-facing structures
  - expose freshness and placement detail through machine-facing summaries
- `port-hosted-protocol`
  - add control-plane and node-agent registration routes and payloads
  - preserve the current hosted auth and route-context vocabulary
- `hosted_control_plane`
  - store registered-node records
  - use registered-node state to resolve launch/list/status/monitor/stop
  - surface stale or missing registration detail explicitly
- `node_agent`
  - register one node and refresh that registration on a fixed interval
  - continue serving the existing node-owned machine and guest routes
- `port-cli`
  - keep canonical `machine` verbs
  - add help/examples for the registered-node workflow
  - render placement and freshness detail from existing machine output

## Interfaces

Primary interfaces:

- registered-node contract
  - `node_name`
  - `endpoint`
  - `token`
  - `registered_at`
  - `last_seen_at`
  - `capability_summary`
  - `registration_state`
- machine placement result
  - `selected_node`
  - `placement_detail`
  - `registration_detail`
  - `runtime_owner`
- canonical machine responses
  - extend current machine status/list/monitor output with registered-node and
    placement metadata instead of inventing a second hosted surface

## Data Flow

Registration:

1. Operator starts `port control-plane serve`.
2. Operator starts `port node-agent serve` with one configured node and a
   registration target.
3. Node agent posts its endpoint/token/freshness to the control plane.
4. Control plane stores that node as registered and eligible for placement.

Launch:

1. Operator runs `port machine launch --machine <name>` against a hosted
   machine.
2. Control plane resolves configured candidate nodes and intersects them with
   live registered nodes.
3. Placement selects one eligible node deterministically.
4. Selected node agent executes the existing local launch path.
5. Placement metadata is written for later machine inspection.

Inspect:

1. Operator runs `port machine list|status|monitor|stop`.
2. Control plane resolves the stored placement and current registration state.
3. Response merges runtime state, placement detail, and registration freshness.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Node never registers | control plane has no live registered record for a configured node | launch/list/status fail with explicit missing-registration detail | start node agent with registration enabled or inspect control-plane logs |
| Node registration goes stale | `last_seen_at` exceeds freshness window | mark node ineligible and surface stale-registration detail | restart or refresh the node agent |
| No eligible registered nodes | candidate filter removes all nodes | reject machine launch with aggregated placement detail | repair node capabilities/freshness or choose another machine/host |
| Selected registered node launch fails | node-agent runtime returns launch error | preserve selected-node detail and surfaced runtime failure | inspect node/runtime logs, then retry |
