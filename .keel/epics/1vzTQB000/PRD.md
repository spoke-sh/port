# Hosted Fleet Registration And Machine Placement - Product Requirements

> Replace explicit control-plane node bindings with a registered hosted fleet
> contract and route canonical machine placement onto eligible registered nodes
> without inventing a second CLI model.

## Problem Statement

Port still relies on explicit control-plane node bindings and cannot register hosted nodes or place canonical machine launches across a live fleet, which keeps the hosted lane below the requested Slicer-class operating model.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Replace explicit node bindings with a control-plane-owned registration contract. | A hosted node can register itself and remain visible to the control plane without `--node-binding` startup flags. | First voyage complete |
| GOAL-02 | Route canonical hosted machine placement onto eligible registered nodes. | `port machine launch` selects a registered node and records placement evidence through the existing machine surface. | First voyage complete |
| GOAL-03 | Keep hosted fleet behavior explicit to operators. | CLI/docs/status surfaces show registered-node or placement detail plus current hosted limits. | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port across prepared cloud nodes and wants a credible hosted fleet story instead of hand-wired demo bindings. | A durable way to register nodes and place machines without editing server startup flags per node. |
| Hosted Port Builder | Extends the control plane and node-agent runtime. | Stable registration, placement, and inventory contracts that can grow into a real hosted control plane. |
| Workload Operator | Launches machines and services through the canonical CLI. | The same `port machine` verbs should work against hosted fleets with placement made visible instead of hidden. |

## Scope

### In Scope

- [SCOPE-01] A control-plane-owned registration contract for hosted nodes.
- [SCOPE-02] Node-agent runtime behavior for registering and refreshing one node against a hosted control plane.
- [SCOPE-03] Deterministic placement of canonical hosted machine launches onto eligible registered nodes.
- [SCOPE-04] Operator-visible machine placement detail and workflow documentation for the registered-node lane.

### Out of Scope

- [SCOPE-05] Autoscaling, fleet rebalancing, or broader price-aware scheduling.
- [SCOPE-06] Multi-tenant RBAC, billing, or external node catalogs.
- [SCOPE-07] Full node health scoring, spread/pack policies, or restart automation beyond the first placement slice.
- [SCOPE-08] A full Slicer-class cluster manager in one voyage.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define a hosted node registration contract that the control plane can store and reason about without relying on transient `--node-binding` CLI flags. | GOAL-01, GOAL-03 | must | Explicit startup flags are the biggest remaining reason the hosted lane is still only a repo-local demo. |
| FR-02 | Port must let a node agent register itself and refresh its registration against the hosted control plane using the shared hosted transport and auth vocabulary. | GOAL-01 | must | The control plane cannot own hosted fleet inventory until nodes can join it directly. |
| FR-03 | Port must route hosted `machine launch` onto an eligible registered node through the canonical `port machine` surface and record which node was selected. | GOAL-02, GOAL-03 | must | Machine placement is the hosted counterpart to the service scheduler work already shipped. |
| FR-04 | Port must surface registered-node and machine-placement detail through operator-visible outputs and docs instead of hiding hosted fleet state behind generic transport success or failure. | GOAL-03 | must | Hosted credibility depends on inspectable fleet behavior, not only launch success. |
| FR-05 | Port must publish a repository-local workflow and proof for registered hosted machine placement, including the explicit limits that remain after the first slice. | GOAL-02, GOAL-03 | should | Docs and evidence are required for discoverability and board traceability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Placement behavior must be deterministic and operator-visible for the same registered-node input set. | GOAL-02, GOAL-03 | must | Hosted placement debugging is impossible if node selection appears arbitrary. |
| NFR-02 | Registration and placement failures must include explicit hosted route and node detail. | GOAL-01, GOAL-03 | must | Operators need concrete failure reasons when a node is absent, stale, or ineligible. |
| NFR-03 | The first voyage must end with executable implementation stories rather than another hosted design-only backlog. | GOAL-01, GOAL-02 | should | The user objective requires continued delivery, not a pause at planning. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use runtime and protocol tests for the registration contract and placement rules.
- Prefer canonical CLI proofs for hosted `machine launch|list|status` against a repository-local registered-node demo.
- Publish docs/help proofs that make the registered-node workflow and remaining hosted limits explicit.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Registered prepared nodes remain the first hosted rollout target. | Placement work may need a second substrate or health abstraction sooner. | Validate in the first voyage. |
| Existing hosted control-plane and node-agent transports can carry registration without inventing a second auth model. | The hosted protocol contract may need a broader redesign. | Validate in the first contract story. |
| Canonical `port machine` verbs remain the right surface for hosted machine placement. | A separate hosted-only machine vocabulary may be needed sooner. | Reassess after the first registered-node launch slice. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Does the first slice also need an explicit operator node inventory command, or is machine-placement visibility sufficient? | Product/Architecture | Open |
| How much node freshness or heartbeat behavior is required before a registered node is eligible for placement? | Architecture | Open |
| Should service placement reuse the same registered-node inventory immediately or stay on explicit config inventory until a follow-on voyage? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted node registration is decomposed into executable stories with traceable verification paths.
- [ ] The first voyage proves a repository-local workflow where a node agent registers with the control plane and `port machine launch` places onto an eligible registered node.
- [ ] Operator docs and help text make the shipped registered-node slice and remaining hosted fleet limits explicit.
<!-- END SUCCESS_CRITERIA -->
