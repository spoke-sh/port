# Implement Hosted Control Plane Demo Lane - Software Requirements Specification

> Deliver a single-node hosted control plane and node-agent demo that executes
> canonical machine and guest verbs over authenticated HTTP transport.

**Epic:** [1vzETR000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

This voyage delivers the first live hosted runtime lane for Port:

- a long-lived control-plane server
- a long-lived node-agent server
- authenticated HTTP routing for hosted `machine` verbs
- authenticated HTTP routing for hosted `guest` verbs
- CLI, SDK, help text, and docs that prove the runnable demo workflow

This voyage does not deliver multi-node scheduling, hardened hosted auth, or
new substrate implementations. It establishes the runtime boundary those later
voyages will build on.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Existing hosted config already names control planes, nodes, and runtime roots coherently enough for a single-node demo. | Assumption | Additional model work will be needed before transport implementation can start. |
| The current `port-agent-protocol` payloads can remain the canonical guest request/response shape for remote routing. | Assumption | Hosted guest transport may need a second protocol layer, which would broaden the voyage. |
| Rust unit tests, CLI proofs, and repository-local demo commands are sufficient evidence for the first hosted lane. | Assumption | Verification planning would need extra infrastructure such as an external integration environment. |
| The local Linux Firecracker lane remains the node agent's execution backend for this voyage. | Dependency | Hosted runtime work would be blocked on an alternate substrate before the transport layer is proven. |

## Constraints

- Keep one canonical CLI model. Do not create hosted-only lifecycle or guest
  verbs.
- Keep one canonical guest protocol payload family. Do not fork local and
  hosted request shapes.
- Treat the control plane as the hosted source of truth and the node agent as
  the hosted runtime owner.
- Fail fast on auth, routing, or runtime-owner mismatches.
- Keep the first lane runnable from the repository without external cloud
  infrastructure.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must expose a live hosted control-plane server that authenticates requests and serves machine inventory, status, monitor, top, and stop routes for configured hosted machines. | SCOPE-01, SCOPE-03 | FR-01 | automated test + CLI demo |
| SRS-02 | Port must expose a live node-agent server that owns one node runtime root and serves machine inspection plus guest operation routes on behalf of the control plane. | SCOPE-02, SCOPE-04 | FR-02 | automated test + CLI demo |
| SRS-03 | Hosted `port machine ...` and `port guest ...` commands must use the live hosted HTTP path when a machine resolves to `hosted-control-plane` mode. | SCOPE-03, SCOPE-04, SCOPE-05 | FR-03 | automated test + CLI demo |
| SRS-04 | `port-sdk` must build the same hosted request paths that the live control plane now serves, and docs must identify that surface as shipped for the single-node hosted lane. | SCOPE-05, SCOPE-06 | FR-05 | automated test + manual review |
| SRS-05 | Port must publish a runnable hosted demo workflow that starts the control plane and node agent, then exercises canonical hosted machine and guest commands. | SCOPE-06 | FR-05 | manual review + CLI demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted transport errors must include enough auth, route, node, or runtime-owner context for operators to understand what failed. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | automated test + inspection |
| SRS-NFR-02 | The first hosted transport lane must remain repository-local and reproducible for verification. | SCOPE-06 | NFR-03 | CLI demo + inspection |
| SRS-NFR-03 | The transport and route contracts must preserve future expansion toward multi-node placement, PVM-aware nodes, and alternate substrates. | SCOPE-01, SCOPE-02, SCOPE-05 | NFR-04 | design review + docs inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
