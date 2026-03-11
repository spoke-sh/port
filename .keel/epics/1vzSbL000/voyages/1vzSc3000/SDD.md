# Host Groups And Service Placement - Software Design Description

> Define and land the first host-group-aware scheduler slice for hosted services and sandboxes, including explicit placement evidence and operator workflow.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns hosted services from a single-node demo into the first
multi-node workflow. The control plane and node agent keep the current service
verbs and guest/runtime ownership model, while the shared model grows
host-group and scheduler-policy contracts that let a hosted service target a
prepared pool of nodes. The first scheduler slice selects one eligible node,
records why it was selected, and surfaces that placement state through the
canonical `port service` status path.

## Context & Boundaries

### In Scope

- shared model and sample config for host groups and scheduler policy
- deterministic node selection for hosted service and sandbox execution
- placement metadata in hosted runtime state and CLI rendering
- docs/help/proof for the multi-node hosted workflow

### Out of Scope

- autoscaling, rebalance, or policy beyond first-fit deterministic placement
- replicated or multi-instance services
- restart policy, health checks, or richer reconciliation
- quota, RBAC, and billing

```
CLI / SDK
   |
   v
control-plane service routes
   |
   v
host-group admission + scheduler selection
   |
   v
selected node agent runtime owner
   |
   v
guest attach tunnel
   |
   v
guest-agent managed process supervisor
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | crate | carry host-group membership, scheduler policy, and placement state in config plus runtime structures | workspace |
| `port-hosted-protocol` | crate | preserve hosted route/auth vocabulary while placement metadata grows | workspace |
| `port-runtime` | crate | own scheduler selection, service runtime state, and CLI-facing status rendering | workspace |
| `port-cli` | crate | keep placement on canonical `port service` and publish help/docs evidence | workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Preserve one service surface | Reuse `port service apply|list|status|stop` rather than adding scheduler-specific verbs | Keeps local and hosted Port aligned as one product |
| Make host groups explicit in the shared model | Add first-class host-group contracts instead of overloading node labels ad hoc | Operators need a durable vocabulary for prepared fleets |
| Start with deterministic first-fit placement | Select the first eligible node in a stable sorted order | Gives repeatable operator evidence before adding broader policy |
| Persist placement alongside runtime state | Record selected node, host group, and admission detail in the service runtime record | Lets list/status/stop explain where a hosted workload lives and why |

## Architecture

The implementation extends the hosted service flow:

1. The operator targets a hosted machine whose backing host declares a host
   group or placement target.
2. The control plane resolves candidate nodes in that host group.
3. The scheduler applies deterministic admission checks against node
   capabilities and runtime state.
4. The selected node agent executes the existing hosted service runtime path.
5. The runtime record persists placement metadata that later `list`, `status`,
   and `stop` calls read back through the same service surface.

## Components

- `port-model`
  - add host-group and scheduler-policy config/state structures
  - publish placement metadata in service runtime state
- `port-runtime`
  - gather eligible nodes for a host group
  - evaluate admission rules and deterministic ordering
  - execute hosted service launch on the selected node
  - persist placement observation and failure detail
- `hosted_control_plane`
  - keep current route family
  - forward service actions to the selected node and surface placement context
- `port-cli`
  - keep command shapes unchanged
  - render host-group/node placement detail in service status/list
  - update help/examples for the multi-node workflow

## Interfaces

Primary interfaces:

- host-group contract
  - host groups contain named membership plus optional scheduling policy
  - hosted nodes declare group membership and capability metadata
- scheduler result
  - `selected_node`
  - `host_group`
  - `admission_detail`
  - `placement_state`
- canonical service responses
  - extend existing service status/list JSON and CLI rendering with placement
    fields instead of inventing a second hosted surface

## Data Flow

Apply:

1. Operator runs `port service apply` against a hosted target.
2. Control plane resolves the target host group and candidate nodes.
3. Scheduler filters nodes by readiness/capability and selects one
   deterministically.
4. Selected node agent executes the existing managed-service runtime path.
5. Runtime state is written with placement metadata.

Status/List:

1. Operator runs `port service list|status`.
2. Control plane resolves the current runtime owner.
3. Response merges desired state, runtime state, and placement metadata.

Stop:

1. Operator runs `port service stop`.
2. Control plane uses the stored placement metadata to reach the current node.
3. Node agent stops the managed process and updates runtime state in place.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unknown host group | scheduler lookup returns no group | reject request with host-group name and target context | choose a valid host group or update config |
| No eligible nodes in host group | admission filters remove all candidates | fail with aggregated ineligibility detail | fix node capabilities or choose another group |
| Stored placement points to a missing node | list/status/stop cannot resolve the selected node | surface `malformed` or stale placement state with detail | re-apply or repair inventory |
| Selected node launch fails after admission | node-agent runtime error | keep placement failure detail and surfaced runtime error | inspect node/runtime logs, then retry |
