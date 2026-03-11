# Execute Hosted Services And Sandboxes - Software Design Description

> Run hosted service and sandbox commands through the live control plane and node agent instead of only storing desired state

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the existing hosted `service` surface from stored desired
state into a real execution path. The CLI and SDK keep the same verbs and
route contracts, while the runtime grows a managed-process layer that lets the
node agent launch, inspect, and stop guest processes through the in-guest
agent. Secrets stay runtime-owned on the node and are injected at launch time
without being reflected back through status output.

## Context & Boundaries

### In Scope

- managed guest-process contract for service and sandbox execution
- node-owned runtime state and secret materialization
- hosted control-plane and node-agent routing for `service apply|list|status|stop`
- operator docs and proof for the shipped hosted execution slice

### Out of Scope

- scheduler policy, host-group balancing, or multi-node placement changes
- secret-backend hardening and external secret stores
- restart policies, health checks, or auto-reconciliation loops
- dedicated `service logs` or `service exec` commands

```
CLI / SDK
   |
   v
control-plane service routes
   |
   v
node-agent runtime owner
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
| `port-agent-protocol` | crate | carry managed service start/status/stop requests over the existing guest bridge | workspace |
| `port-guest-agent` | crate/binary | own guest process launch, signal, and log capture | workspace |
| `port-hosted-protocol` | crate | reuse existing hosted route/auth context for service verbs | workspace |
| `port-sdk` | crate | keep hosted service requests aligned with canonical CLI/API surfaces | workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Preserve one service surface | Reuse `port service apply|list|status|stop` and existing SDK routes | Keeps hosted execution aligned with the canonical operator model |
| Split definition from runtime state | Keep service definition JSON and add runtime-state JSON under the node runtime root | Operators need desired state and observed state without exposing secrets |
| Make node agent the service owner | Control plane authorizes and routes; node agent reads secrets, owns runtime files, and brokers guest lifecycle | Matches the existing hosted ownership model |
| Extend guest agent instead of shell hacks | Add a managed-process contract rather than spawning background shells through `exec` | Gives deterministic start/status/stop behavior and better proof surfaces |

## Architecture

The implementation adds a managed-process layer beneath the existing `service`
surface:

1. `port service apply` still validates and persists the definition.
2. For hosted machines, the control plane routes the request to the selected
   node agent.
3. The node agent resolves machine secrets, builds the launch environment, and
   attaches to the live guest transport.
4. The guest agent starts a managed guest process, captures stdout/stderr, and
   returns runtime metadata.
5. The node agent writes runtime-state JSON under the machine runtime root so
   later `list`, `status`, and `stop` commands read live state instead of only
   desired state.

## Components

- `port-agent-protocol`
  - add managed process request/response types for start, status/list, and stop
  - keep the transport on the existing hosted and local guest bridge
- `port-guest-agent`
  - maintain a lightweight managed-process table
  - launch service/sandbox commands with supplied env and working directory
  - capture stdout/stderr into guest-visible files and report exit status
- `port-runtime`
  - merge service definitions with runtime state
  - materialize machine secrets into launch env without returning raw values
  - expose hosted runtime helpers for apply/list/status/stop
- `hosted_control_plane`
  - keep the current route family but turn it into real execution for hosted lanes
- `port-cli` and `port-sdk`
  - preserve command/request shapes
  - update rendering so operators see runtime state, exit metadata, and log paths

## Interfaces

Primary interfaces:

- guest managed-process contract:
  - `start(name, kind, command, env, cwd?) -> runtime state`
  - `list() -> runtime states`
  - `status(name) -> runtime state`
  - `stop(name) -> runtime state`
- hosted service routes:
  - reuse existing hosted `services` endpoints and auth headers
  - change semantics from persisted desired state only to desired state plus live execution
- runtime files:
  - `services/<name>.json` remains the desired-state definition
  - `services/runtime/<name>.json` becomes the observed runtime state record

## Data Flow

Apply:

1. CLI/SDK sends `service apply`.
2. Control plane authorizes and forwards to node agent.
3. Node agent loads definition and secret values from runtime-owned files.
4. Node agent attaches to the guest and sends managed-process start.
5. Guest agent launches the process and returns runtime metadata.
6. Node agent persists observed runtime state and returns merged status.

Status/List:

1. CLI/SDK requests service status or list.
2. Node agent reads definition plus runtime-state records.
3. If needed, node agent refreshes live process state through the guest bridge.
4. Response renders desired state, observed state, and operator-safe metadata.

Stop:

1. CLI/SDK sends `service stop`.
2. Node agent attaches to the guest and signals the managed process.
3. Guest agent reports the new state.
4. Node agent persists the stopped runtime record and returns updated status.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Secret binding references an unknown secret | node agent validation before launch | reject `service apply` with secret name and machine context | add the secret or fix the binding |
| Guest process fails to launch | guest-agent start returns failure | keep desired state, record failed runtime state, surface detail | inspect status detail and guest log paths |
| Runtime state becomes stale after process exit | node agent refresh or PID check | report exited/stale state with exit metadata | re-apply or stop depending on operator intent |
| Hosted proof flakes under nested shells | deterministic proof script catches long-path/startup issues | fail verification loudly | keep short temp roots and prebuilt binaries in proof scripts |
