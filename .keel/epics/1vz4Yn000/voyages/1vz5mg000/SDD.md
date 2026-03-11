# Hosted Runtime And Service Expansion - Software Design Description

> Implement the first hosted runtime control path, then sequence forwarding,
> monitoring, secrets, services, sandboxes, and SDK surfaces on top of the
> hosted foundation contracts.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the hosted foundation contracts from `1vz4cU000` into a real
runtime and product sequence. The order is deliberate:

1. implement hosted machine runtime control
2. implement hosted guest runtime operations
3. extend forwarding with detached and Unix-socket modes
4. add monitoring and `top`
5. add secrets, services, and sandboxes
6. publish SDK and API clients on the stabilized runtime surface

## Context & Boundaries

### In Scope

- hosted runtime driver work in the CLI/runtime/model
- control-plane and node-agent runtime APIs and transport brokerage
- forwarding, monitoring, secrets, services, sandboxes, and SDK/API sequencing

### Out of Scope

- multi-tenant RBAC and enterprise auth
- scheduler policy beyond existing nodes and host groups
- hosted billing or tenancy management
- substrate-specific product splits that would fork the CLI model

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted auth and API identity contract | internal contract | identify and authenticate control-plane calls | `1vz4gb000` |
| Hosted node and host-group inventory | internal contract | route lifecycle, monitoring, and placement through owned nodes | `1vz4hB000` |
| Hosted machine lifecycle contract | internal contract | preserve canonical `machine` verbs for hosted runtime | `1vz4h3000` |
| Hosted guest attach contract | internal contract | preserve canonical `guest` verbs and protocol framing | `1vz4gc000` |
| Future hosted control plane | external component | authorize, route, and report hosted machine and service state | planned |
| Future hosted node agent | external component | own host-local runtime, guest attach, monitoring, and service actions | planned |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime first | Implement machine and guest runtime paths before productized monitoring, services, or SDK work | Later surfaces need a real hosted control path, not more modeling |
| Keep canonical verbs | Continue using `machine`, `guest`, and later `service` surfaces instead of hosted-only aliases | Port's operator model stays coherent |
| Forwarding before monitoring/services | Detached and Unix-socket forwarding depend on the hosted guest runtime and unblock operational workflows earlier than higher-level services | Keeps the follow-on order technically justified |
| SDK last | Publish SDK and API clients after the runtime surface stabilizes | Prevents freezing premature API shapes |

## Architecture

The voyage extends the current three-layer Port structure:

- shared contracts in `port-model`
- runtime and control-plane routing in `port-runtime`
- operator surfaces in `port-cli`

Hosted runtime work adds two external actors that Port already names:

- control plane: authentication, routing, inventory, lifecycle/state API
- node agent: host-local runtime, guest transport brokerage, monitoring, and
  service execution owner

## Components

- hosted runtime driver:
  resolves whether a machine is local or hosted, then targets the appropriate
  control path without changing the CLI verbs
- hosted guest runtime bridge:
  obtains a guest protocol stream through control-plane authorization and
  node-agent transport ownership
- forwarding extension:
  adds detached and Unix-socket forwarding on top of the hosted guest bridge
- monitoring surface:
  adds machine-level status streams and `top`-style process/runtime inspection
- service control surface:
  layers secrets plus service/sandbox orchestration on top of hosted runtime
- SDK/API clients:
  expose typed clients only after the runtime verbs and payloads stabilize

## Interfaces

- machine runtime interface:
  hosted `machine list|status|stop` map onto the existing control-plane and
  node-agent ownership contracts
- guest runtime interface:
  hosted `guest exec|copy|pty|logs|forward` preserve the existing guest request
  and response frames
- forwarding interface:
  detached and Unix-socket forwarding extend `guest forward` instead of adding a
  new command family
- monitoring interface:
  hosted machine and node metrics/status stream through canonical machine/top
  surfaces
- secrets/services/sandboxes interface:
  hosted service execution consumes secrets and runtime ownership through the
  same control-plane and node-agent boundary
- SDK interface:
  typed clients mirror the canonical machine, guest, and service verbs

## Data Flow

1. The operator invokes a canonical CLI command.
2. The CLI resolves whether the target is local or hosted.
3. Hosted targets authenticate to the named control plane.
4. The control plane resolves node ownership and current runtime state.
5. The control plane either answers directly, or asks the owning node agent for
   host-local state or action.
6. Guest and forwarding operations broker a byte stream through the node agent
   to the existing in-guest agent.
7. Monitoring, secrets, and services reuse the same ownership and routing path.
8. SDK clients mirror those same verbs once the wire surface stabilizes.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted runtime lands without canonical machine/guest verbs | CLI review or code review finds hosted-only command families | reject the slice | keep runtime work behind the existing verbs |
| Guest runtime redefines the guest protocol | protocol review finds hosted-only framing | reject the slice | tunnel the existing guest protocol instead |
| Detached forwarding lands before hosted guest runtime | board review shows inverted ordering | block the story | finish hosted guest runtime first |
| Monitoring/services outrun runtime ownership | design review shows no node-agent data owner | block the story | land runtime ownership and attach first |
| SDK freezes unstable APIs | client review shows churn against moving control-plane contracts | defer SDK publication | ship runtime and service surfaces first |

## Story Order

1. Implement Hosted Control Plane Runtime Path
2. Implement Hosted Guest Operations Runtime Path
3. Add Detached And Unix-Socket Forwarding
4. Add Hosted Monitoring And Top
5. Add Hosted Secrets Services And Sandboxes
6. Publish Hosted SDK And API Clients
