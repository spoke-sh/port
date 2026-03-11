# Persistent Registration And Inventory Sync - Software Design Description

> Define and deliver persistent node registration, heartbeat freshness, and
> imported fleet inventory contracts for the hosted control plane.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the registered-node hosted lane into a durable fleet
contract. The hosted control plane gains a persisted node registry and imported
inventory store; node agents refresh their registration into that store; and
canonical hosted inspection surfaces report merged fleet state plus freshness
detail. The design deliberately stops short of autoscaling or policy
automation, but it leaves those later slices with durable state instead of a
repo-local demo.

## Context & Boundaries

### In Scope

- persisted hosted node registry state and freshness metadata
- node-agent registration refresh against that persistent store
- imported fleet inventory materialization into the hosted control-plane view
- canonical operator visibility for persisted/merged fleet state

### Out of Scope

- autoscaling, rebalancing, or richer placement policy
- live per-provider discovery backends for all clouds
- replacing the current hosted control-plane or node-agent transport
- multi-tenant auth, billing, or HA control-plane work

```
CLI / SDK
   |
   v
hosted control plane
   |
   +--> persisted node registry
   |
   +--> imported inventory store
   |
   +--> merged fleet resolver
   |
   v
canonical machine / fleet inspection routes
   ^
   |
node-agent registration refresh
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | crate | carry durable node records, freshness, and imported inventory provenance | workspace |
| `port-hosted-protocol` | crate | extend hosted routes and payloads for persistence-aware registration and fleet inspection | workspace |
| `port-runtime` | crate | own persistent registry I/O, freshness rules, merge logic, and hosted route handlers | workspace |
| `port-cli` | crate | publish durable fleet inspection and workflow help through canonical Port surfaces | workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Persist registry under the control-plane runtime root | Reuse Port-owned runtime files rather than adding a database in the first slice | Keeps the durable fleet contract executable inside the current repo-local hosted lane |
| Keep configured nodes as the canonical identity namespace | Imported inventory and runtime registration resolve onto existing node names with provenance metadata | Avoids a second fleet taxonomy while Port is still configuration-led |
| Merge imported inventory and live registration explicitly | Inspection surfaces report configured state, imported state, and live freshness separately | Operators need to distinguish model intent from imported or live runtime fact |
| Expire by explicit freshness window | Stale nodes remain visible but ineligible instead of disappearing silently | Makes restart and heartbeat failures inspectable |

## Architecture

The implementation adds three cooperating state layers:

1. A persisted registry file for live node registration records, updated by the
   control plane when node agents register or refresh.
2. A persisted imported-inventory file that records externally supplied fleet
   membership and provenance.
3. A merged fleet resolver that combines configured node definitions, imported
   inventory, and live registration/freshness when machine or fleet inspection
   routes execute.

Canonical CLI flows keep using `port control-plane serve`, `port node-agent
serve`, and `port machine ...`. The new data only changes how the hosted
control plane resolves and reports the fleet view.

## Components

- `port-model`
  - add durable registry and imported-inventory structures
  - expose fleet provenance and freshness fields in machine or fleet summaries
- `port-hosted-protocol`
  - add route payloads for persisted registration refresh and fleet inspection
  - keep the current hosted auth contract unchanged
- `hosted_control_plane`
  - load/store persistent registry and import files
  - merge configured, imported, and live state for route handlers
  - mark stale nodes explicitly instead of deleting them from output
- `node_agent`
  - refresh registration into the control plane using the existing token model
  - continue serving node-owned machine and guest routes unchanged
- `port-cli`
  - render fleet state through canonical status/help/docs surfaces
  - publish the durable registration/import workflow and explicit remaining
    limits

## Interfaces

Primary interfaces:

- durable node registration record
  - `node_name`
  - `endpoint`
  - `token_fingerprint` or equivalent non-secret identity marker
  - `registered_at`
  - `last_seen_at`
  - `freshness_state`
  - `source = live-registration`
- imported inventory record
  - `node_name`
  - `provider`
  - `provenance`
  - `imported_at`
  - `capability_summary`
  - `source = imported-inventory`
- merged fleet summary
  - `node_name`
  - `configured`
  - `imported`
  - `registered`
  - `freshness_state`
  - `routing_eligibility`
  - `detail`

## Data Flow

Registration refresh:

1. Operator starts `port control-plane serve`.
2. Operator starts `port node-agent serve`.
3. Node agent posts its endpoint and freshness update to the control plane.
4. Control plane writes or updates the durable registry record.
5. Future route handlers reload that record after restart.

Imported inventory:

1. Operator supplies an imported fleet contract through the planned CLI/runtime
   surface.
2. Control plane validates and writes the imported inventory file.
3. Merged fleet resolution combines configured nodes with imported records.

Inspect:

1. Operator runs canonical inspection commands.
2. Control plane loads configured model state, imported inventory, and durable
   registration state.
3. Merged resolution marks each node as fresh, stale, missing, or imported-only
   according to the current rules.
4. CLI output surfaces the merged detail and explicit policy limits.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Registry file missing or unreadable | control plane load fails | return explicit durable-registry load error with path context | repair file or recreate by restarting registration |
| Node freshness expired | `last_seen_at` exceeds configured freshness window | keep node visible but mark it stale and ineligible | restart or reconnect the node agent |
| Imported inventory contains unknown or conflicting nodes | merge validation detects mismatch | return explicit import mismatch detail and skip invalid entries | fix the import source and reapply |
| Control-plane restart loses in-memory state | restart occurs with persisted files present | rebuild fleet state from the persisted registry and import files | automatic on restart if files are valid |
