# Service Policy And Secret Runtime Foundations - Software Design Description

> Define and ship the first restart-policy, health-state, and hardened secret-materialization slice through the canonical port service workflow.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends Port's existing service runtime owner into a real managed
service supervisor. The design keeps one canonical surface:

- the shared model describes restart policy, health policy, and secret
  materialization intent
- the runtime owns process supervision, health evaluation, and materialized
  secret state
- the hosted control plane and SDK reuse the existing `port service` route
  family and response types, adding fields rather than introducing a second API
- docs, examples, and CLI help publish one operator workflow for local and
  hosted execution

## Context & Boundaries

### In Scope

- shared service policy and secret-backend contract changes
- runtime supervision and state reporting
- local and hosted CLI/API/SDK exposure of the same state
- operator workflow and evidence

### Out of Scope

- external secret managers, KMS, or tenant-aware auth
- advanced scheduler policy beyond existing placement
- autoscaling, preemption, or broader orchestration

```
┌──────────────────────────────────────────────────────────────────┐
│                          This Voyage                            │
│                                                                  │
│  port service CLI / SDK / hosted API                             │
│              │                                                   │
│              ▼                                                   │
│     shared service policy + secret contract                      │
│              │                                                   │
│              ▼                                                   │
│   runtime supervisor + health evaluator + secret materializer    │
│              │                                                   │
│              ▼                                                   │
│      guest managed process and status projection                 │
└──────────────────────────────────────────────────────────────────┘
         ↑                                   ↑
    local runtime owner                hosted node runtime owner
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | internal crate | Shared service policy, secret backend, and status schema | current workspace |
| `port-runtime` | internal crate | Runtime supervision, secret materialization, and CLI execution | current workspace |
| `port-hosted-protocol` | internal crate | Hosted route and payload contracts for service status and mutation | current workspace |
| `port-sdk` | internal crate | Typed hosted client mirror for the canonical service surface | current workspace |
| Existing guest managed-process plumbing | internal runtime contract | Starts, stops, and inspects managed guest processes | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical service surface | Keep `port service` as the only user-facing family | Preserves the hard-cutover requirement and avoids a hosted-only service vocabulary |
| Policy shape | Model restart and health policy in the shared config and status types | Lets local, hosted, CLI, and SDK speak one contract |
| Secret contract | Replace plaintext runtime JSON as the execution source with a stronger runtime-owned backend plus explicit materialization | Improves the operator story without requiring external secret-manager integrations in the first slice |
| Runtime owner | Reuse the existing resolved runtime owner for both supervision and secret materialization | Keeps state attributable and inspectable through one owner path |
| Verification | Prefer Rust tests plus a repo-local CLI proof; use VHS only if the operator workflow benefits from a recording | Matches the repository's detected verification stack and keeps proofs practical |

## Architecture

The design adds three coordinated layers:

1. Shared contracts
   - service spec fields for restart policy and health policy
   - secret backend metadata and materialization mode
   - service status fields for restart count, last exit, health state, and
     secret-source detail
2. Runtime supervision
   - a supervisor loop or reconciliation pass that inspects the managed guest
     process, applies restart policy, updates health, and stores status
   - secret backend resolution and per-process materialization under the
     resolved runtime owner
3. Product surfaces
   - CLI help/examples/status output
   - hosted control-plane projections
   - SDK request/status types

## Components

### Shared Service Policy Model

Purpose:
- define restart policy, health policy, and secret-backend intent once

Behavior:
- validates supported combinations early
- serializes through config, runtime manifests, hosted payloads, and SDK types

### Runtime Supervisor

Purpose:
- turn service desired state into running/healthy/stopped runtime state

Behavior:
- starts managed processes
- records exit status and restart count
- decides whether to restart based on the selected policy
- updates persisted service status for `list|status|stop`

### Secret Backend And Materializer

Purpose:
- store secret values in a stronger runtime-owned form and materialize them for
  service execution without making the service spec itself the plaintext store

Behavior:
- accepts writes from `port service secret put`
- resolves bindings during `service apply`
- prepares env/file materialization for the managed process
- reports secret-source metadata without leaking values

### Hosted Projection Layer

Purpose:
- keep control-plane and SDK behavior aligned with the local/runtime contract

Behavior:
- forwards policy/state fields through existing service routes
- does not invent hosted-only service endpoints

## Interfaces

- CLI:
  - `port service secret put|list|remove`
  - `port service apply|list|status|stop`
- Shared model:
  - service policy types
  - secret backend and binding types
  - service status types
- Hosted API:
  - existing `/services` route family with expanded request/status payloads
- SDK:
  - `services().apply|list|status|stop` mirrors the updated hosted route schema

## Data Flow

1. Operator stores a secret through `port service secret put`.
2. Port writes the secret into the stronger runtime backend for the resolved
   owner.
3. Operator applies a service or sandbox with restart and health policy plus
   secret bindings.
4. Runtime resolves the service spec, materializes needed secrets, and starts
   the managed guest process.
5. Supervisor inspects process state and health, updates status, and restarts
   the process if policy requires it.
6. `port service list|status` and hosted SDK/API calls project the stored state
   without reading raw runtime files directly.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unsupported restart or health policy | shared-model validation | fail `service apply` with explicit policy error | operator selects a supported policy |
| Missing or unreadable secret binding | runtime secret resolution | fail launch with binding context and no partial start | operator fixes or re-creates the secret |
| Materialization backend unavailable | runtime backend initialization | fail fast and leave service desired state explicit | operator repairs backend path/config and reapplies |
| Managed process exits unexpectedly | supervisor inspection | update last-exit state and restart if policy allows | automatic restart or operator stop/debug |
| Health check cannot execute | health evaluator | mark state unhealthy with detail instead of pretending healthy | operator inspects status and corrects the check or workload |
