# Hosted Runtime Transport - Product Requirements

> Turn Port's hosted story into a runnable product lane by shipping a real
> control plane and node-agent transport behind the canonical CLI and SDK.

## Problem Statement

Port now has credible hosted contracts, inventory vocabulary, runtime-root
ownership rules, and SDK request builders, but hosted Port is still not a real
runtime product. There is no live control-plane server, no node agent, and no
networked transport that lets `port machine ...` and `port guest ...` operate
against a hosted environment the way they already do locally.

Compared with the user objective and the SlicerVM comparison, this leaves Port
with a major product gap:

- hosted execution is still config-backed and in-process rather than remote
- the hosted API is documented but not actually served
- the SDK can build requests, but there is no daemon pair to satisfy them
- the canonical CLI cannot yet prove that hosted machine and guest verbs work
  end-to-end over an authenticated control path

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship a runnable hosted demo lane | Operators can start a control plane and node agent, then run canonical machine and guest verbs over authenticated HTTP | First voyage complete |
| GOAL-02 | Preserve one Port operator model | Local and hosted use the same `machine` and `guest` verbs plus the same guest protocol payloads | First voyage complete |
| GOAL-03 | Create the foundation for multi-node and PVM-aware hosted work | Placement, auth, and transport decisions leave clear follow-on hooks for scheduler, host groups, PVM host kits, and alternate substrates | Epic planned coherently |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port locally today and wants a credible hosted lane next | A real hosted workflow that feels like the local CLI, not a separate product |
| Hosted Port Builder | Builds the future control plane, node agent, and API surface | Clear ownership boundaries, transport contracts, and routing primitives |
| SDK / API Consumer | Wants to automate Port without shelling out for every call | Stable request and response contracts backed by a real service endpoint |

## Scope

### In Scope

- [SCOPE-01] A long-lived hosted control-plane server with bearer-token auth.
- [SCOPE-02] A long-lived node-agent server that owns one execution node's
  runtime root.
- [SCOPE-03] Live hosted machine inventory, status, monitor, top, and stop
  routes over authenticated HTTP.
- [SCOPE-04] Live hosted guest `exec`, `copy`, `pty`, `logs`, and `forward`
  routes that preserve the current guest protocol payloads.
- [SCOPE-05] CLI and SDK routing that switches hosted machines from in-process
  runtime inspection to remote HTTP transport.
- [SCOPE-06] Runnable docs and evidence for a single-node hosted demo lane.

### Out of Scope

- [SCOPE-07] Multi-tenant auth, RBAC, and hosted billing.
- [SCOPE-08] Generalized scheduler policy beyond explicit node or host-group
  targeting.
- [SCOPE-09] PVM host-kit implementation itself.
- [SCOPE-10] Apple Virtualization Framework implementation itself.

Those lanes remain critical, but this epic must first make hosted Port real
enough that later substrate and placement work has a runtime to attach to.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must run a hosted control-plane server that authenticates clients and routes canonical machine and guest verbs. | GOAL-01, GOAL-02 | must | Hosted Port is not credible until the documented API is actually served. |
| FR-02 | Port must run a node-agent server that owns host-local runtime state and performs machine plus guest work on behalf of the control plane. | GOAL-01, GOAL-03 | must | A control plane without a runtime owner is only another contract layer. |
| FR-03 | Hosted machine list, status, monitor, top, and stop operations must execute through the remote control-plane path without inventing hosted-only verbs. | GOAL-01, GOAL-02 | must | Port's CLI model is one of its strongest assets and must stay canonical. |
| FR-04 | Hosted guest exec, copy, pty, logs, and forward operations must execute through the remote control-plane plus node-agent path while preserving the existing guest protocol payloads. | GOAL-01, GOAL-02 | must | Guest transport must remain shared across local and hosted execution. |
| FR-05 | Port must publish runnable docs, examples, help text, and SDK/API usage for the hosted demo lane. | GOAL-01 | should | A capability is not done until operators can discover and run it. |
| FR-06 | The shipped hosted transport must remain compatible with later multi-node scheduling, host groups, PVM host kits, and alternate substrates. | GOAL-03 | should | This epic must unblock, not constrain, later strategic work. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Local and hosted Port must keep one canonical CLI and one canonical guest protocol. | GOAL-02 | must | Prevents product fragmentation and duplicate transport logic. |
| NFR-02 | Hosted failures must surface clear auth, routing, and runtime-owner context to operators. | GOAL-01, GOAL-03 | must | Remote failures are otherwise too opaque to debug safely. |
| NFR-03 | The first hosted transport lane must be testable and demonstrable entirely from the repository. | GOAL-01 | must | The product needs repeatable end-to-end evidence, not only design claims. |
| NFR-04 | Planning must explicitly preserve follow-on room for multi-node, PVM-aware, and AVF-aware execution work. | GOAL-03 | should | Hosted runtime work should reinforce the broader Port strategy. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use story-level Rust tests for protocol, routing, and CLI behavior.
- Use CLI-level proof to show operators can start the demo daemons and issue
  canonical hosted commands.
- Prefer explicit route, auth, and runtime-owner evidence in story artifacts so
  future hosted and substrate work can reuse it.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A single-node hosted demo is enough to validate the control-plane and node-agent split. | The first voyage may need multi-node behavior earlier. | Reassess after the first runnable demo lands. |
| Bearer-token auth is sufficient for the first live hosted slice. | Auth work may need to expand before the first transport lane is useful. | Validate during the control-plane story. |
| Reusing the existing guest protocol payloads is sufficient for live hosted guest operations. | Hosted guest traffic may need a second protocol layer sooner than expected. | Validate during guest transport implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first hosted demo use plain HTTP on loopback or require TLS for local proof? | Architecture | Open |
| How much streaming fidelity can the first remote `pty` and `forward` slice carry without a websocket layer? | Architecture | Open |
| Should services and secrets stay config-backed until hosted runtime transport is proven? | Product/Architecture | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Operators can run a documented single-node hosted demo lane and issue canonical hosted `machine` plus `guest` verbs against it.
- [ ] The CLI and SDK use the same live hosted HTTP surface instead of diverging into separate transport models.
- [ ] Follow-on planning for scheduler, PVM-aware hosted placement, and alternate substrates can build on this transport boundary without redoing the operator model.
<!-- END SUCCESS_CRITERIA -->
