# Hosted API And Inventory - Software Design Description

> Define and begin implementing the first authenticated hosted control surface for node-aware inventory, lifecycle control, and guest bridge attachment.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage establishes the first implementable hosted-control foundation
without pretending Port already ships a full daemon or multi-tenant platform.
The design keeps the current CLI and guest protocol intact while defining the
minimum new contracts that let later implementation slices land coherently:

- a hosted auth and API identity contract for the `port` client,
- a node and host-group inventory model for placement and ownership,
- hosted `machine list|status|stop` routing over that control plane,
- a hosted guest bridge attach contract that reuses the existing guest
  protocol instead of creating a hosted-only guest API.

The main architectural rule is that hosted Port changes the owner and route of
machine and guest operations, not the operator verbs or guest semantics.

## Context & Boundaries

### In Scope

- token-shaped hosted API identity and endpoint vocabulary,
- node records, host-group records, and lifecycle ownership terms,
- hosted machine inventory and lifecycle routes for `list|status|stop`,
- guest bridge attachment between CLI, control plane, node agent, and
  in-guest agent,
- CLI/help/docs wording that explains what is shipped versus planned.

### Out of Scope

- a production-ready hosted daemon implementation,
- scheduler policy beyond node and host-group vocabulary,
- monitoring or `top`,
- secrets, services, and sandboxes,
- detached forwarding, Unix-socket forwarding, and SDK packaging.

```
┌────────────────────────────────────────────────────────────────┐
│                          port CLI                              │
│      doctor / machine / guest / future sdk and hosted api     │
└───────────────────────────────┬────────────────────────────────┘
                                │
                 hosted target selection and auth
                                │
                      ┌─────────▼─────────┐
                      │ control plane API │
                      │ inventory + auth  │
                      └───────┬─────┬─────┘
                              │     │
                    ┌─────────▼─┐ ┌─▼──────────┐
                    │ inventory │ │ guest      │
                    │ lifecycle │ │ bridge     │
                    └──────┬────┘ └────┬───────┘
                           │           │
                     ┌─────▼───────────▼─────┐
                     │ node agent / host kit │
                     │ substrate owner       │
                     └───────────┬───────────┘
                                 │
                        ┌────────▼────────┐
                        │ port-guest-agent │
                        │ current protocol │
                        └──────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | workspace crate | canonical vocabulary for host, machine, artifact, and hosted contracts | workspace current |
| `port-runtime` | workspace crate | current local lifecycle and guest transport semantics that hosted work must preserve | workspace current |
| `docs/hosted.md` | canonical doc contract | current role split and hosted-machine vocabulary | repo current |
| future control-plane service | external component | hosted API endpoint, auth, inventory, and command routing | planned |
| future node agent | external component | host-local lifecycle owner and guest bridge broker | planned |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Auth model | Start with token-based API identity tied to an explicit hosted endpoint | It is the smallest credible surface for the first hosted slice and leaves room for future multi-user auth |
| Operator surface | Keep `machine` and `guest` as the only canonical verbs | Port already has a strong CLI model; hosted work must not fork it |
| Placement vocabulary | Model nodes and host groups before scheduler policy | Hosted inventory must know where machines live before it can schedule them |
| Lifecycle routing | Make hosted `list|status|stop` explicit shared contracts before implementing a full daemon | Prevents hidden or ad hoc hosted behavior |
| Guest transport | Preserve the existing guest protocol and add a bridge attach contract above it | Avoids separate local and hosted guest APIs |

## Architecture

The hosted foundation is layered deliberately:

1. `port-model`
   carries typed hosted endpoint, auth, node, host-group, lifecycle, and guest
   bridge contracts.
2. `port-cli`
   remains the canonical operator surface and exposes hosted-aware help and
   machine reporting without introducing hosted-only verb families.
3. `port-runtime`
   continues to own the local execution lane, but gains shared routing and
   status abstractions that hosted control can later implement.
4. future control plane
   owns API identity, inventory, desired lifecycle state, and host selection.
5. future node agent
   owns host-local runtime roots, hypervisor processes, and byte-stream
   bridging to the guest agent.

The first voyage focuses on contracts and operator visibility. It does not
claim a production hosted service already exists.

## Components

- hosted auth and API identity:
  typed endpoint and token contract that lets config, docs, and future CLI
  routing name the hosted control plane explicitly.
- node and host-group inventory:
  typed records that express node ownership, host-group membership, and the
  future placement boundary for machines.
- hosted lifecycle contract:
  shared machine-surface fields that distinguish local runtime-root control
  from hosted control-plane and node-agent ownership.
- hosted guest bridge attach contract:
  typed route that describes how the CLI obtains a guest protocol stream for a
  hosted machine without redefining the guest protocol itself.
- CLI/docs alignment:
  help text and docs that teach the hosted story while keeping shipped versus
  planned behavior explicit.

## Interfaces

- hosted API identity:
  a named control-plane endpoint plus operator token, expressed in shared model
  types rather than environment-only convention.
- machine inventory interface:
  hosted machine summaries map onto the same `machine list|status|stop`
  surface already used locally, but the reported control contract identifies
  hosted ownership and routing.
- guest bridge interface:
  control plane authorizes attachment, node agent opens the host-local guest
  transport, and the existing request and response frames continue unchanged.
- host-group interface:
  host groups are inventory and placement scopes, not a second machine model.

## Data Flow

1. The operator runs `port machine ...` or `port guest ...`.
2. The CLI resolves whether the target machine is local or hosted.
3. For hosted targets, the CLI targets a named hosted endpoint and includes the
   hosted auth token contract.
4. The control plane resolves inventory, host group, node ownership, and
   machine lifecycle state.
5. For lifecycle reads and stop, the control plane answers directly or asks the
   owning node agent for host-local state.
6. For guest attachment, the control plane authorizes the request and asks the
   node agent to attach to the machine guest transport.
7. The node agent bridges the existing guest protocol stream to the in-guest
   `port-guest-agent`.
8. The CLI renders results through the existing `machine` or `guest` surface.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted config invents a second operator model | doc or code review finds hosted-only verbs or mismatched status vocabulary | reject the change | keep lifecycle and guest work under the existing `machine` and `guest` surfaces |
| Hosted auth is implied rather than modeled | endpoint or token semantics only exist in prose | fail the story | require typed model fields and CLI/docs exposure |
| Node inventory lacks host-group or ownership semantics | contract review finds no placement vocabulary | fail the story | add explicit node and host-group records before scheduling work |
| Hosted lifecycle claims shipped behavior without runtime support | help text or docs overstate availability | fail review | distinguish current contract from implemented control-plane behavior |
| Hosted guest work forks the guest protocol | new hosted-only request format appears | reject the design | preserve the current guest protocol and add only attach or bridge routing above it |

## Story Decomposition

1. Define Hosted Auth And API Contract
   Publish the first endpoint and token contract in the shared model, surface
   it in docs and CLI help, and keep the auth story explicit rather than
   implicit.
2. Define Hosted Node Inventory Model
   Publish node and host-group records, including ownership and placement
   vocabulary that later scheduler work can build on.
3. Define Hosted Machine Lifecycle Surface
   Extend the shared machine lifecycle contract so hosted `list|status|stop`
   can be represented explicitly through the canonical CLI model.
4. Define Hosted Guest Bridge Attach Contract
   Publish the first hosted guest attach route and how it preserves the current
   guest protocol over control-plane and node-agent brokerage.
5. Sequence Hosted Follow-On Work
   Record the ordered backlog and operator-facing boundary for monitoring,
   secrets, services, sandboxes, detached forwarding, Unix-socket forwarding,
   and SDK work once the hosted foundation contracts are in place.
