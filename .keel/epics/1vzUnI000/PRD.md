# Durable Hosted Fleet Control - Product Requirements

> Extend Port's hosted lane from a repo-local registered-node demo into a
> durable fleet control contract with persisted node registration, external
> inventory import, and explicit fleet policy boundaries.

## Problem Statement

Port now supports live node self-registration and canonical hosted machine
inspection, but the fleet state is still process-local and repo-local. If the
control plane restarts, registered nodes disappear. Operators also have no way
to bridge existing fleet sources into Port or to express broader hosted
placement policy beyond the current deterministic demo contract. That leaves
Port short of the hosted product story the user requested when compared to
Slicer-class systems.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Persist hosted node registration and freshness state across control-plane restarts. | A registered node remains visible after a control-plane restart until its freshness window expires or it is explicitly removed. | First voyage complete |
| GOAL-02 | Give operators an explicit way to import or mirror external fleet inventory into Port's hosted model. | Operators can materialize imported node inventory and see it through the same hosted control-plane status surface. | First voyage complete |
| GOAL-03 | Make broader hosted fleet policy explicit without pretending autoscaling or full scheduler policy already ship. | Docs and CLI output distinguish persisted registration, imported inventory, and current policy limits. | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Hosted Platform Operator | Runs Port control planes and node agents across customer or internal fleet hosts. | Durable hosted inventory that survives restarts and can ingest existing fleet membership without hand-recreating it each run. |
| Port Hosted Product Builder | Evolves the hosted control plane and node agent. | Stable contracts for persistence, freshness, and imported inventory that later policy and autoscaling work can reuse. |
| Workload Operator | Uses canonical `port machine` and `port service` flows against hosted infrastructure. | Consistent fleet visibility and explicit policy limits rather than hidden demo-only behavior. |

## Scope

### In Scope

- [SCOPE-01] Persisted hosted node registration state, including freshness or heartbeat metadata.
- [SCOPE-02] Control-plane and node-agent behavior that refreshes persisted registration through the existing hosted transport and auth model.
- [SCOPE-03] A first imported-inventory contract that can materialize external node membership into Port's hosted fleet view.
- [SCOPE-04] Operator-facing CLI and docs that publish the durable registration and imported inventory workflow plus the remaining fleet-policy limits.

### Out of Scope

- [SCOPE-05] Autoscaling, rebalancing, or price-aware/region-aware scheduling.
- [SCOPE-06] Multi-tenant auth, billing, quotas, or hosted SaaS account management.
- [SCOPE-07] A fully general external discovery backend for every cloud provider in one slice.
- [SCOPE-08] Replacing the canonical `port machine` or `port service` verbs with hosted-only command families.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must persist hosted node registration and freshness state so the control plane can recover fleet membership after restart. | GOAL-01 | must | A hosted product cannot depend on in-memory registration only. |
| FR-02 | Port must let node agents refresh persisted registration through the existing hosted control-plane transport and auth contract. | GOAL-01 | must | The durable registry must remain compatible with the current live hosted path. |
| FR-03 | Port must define and ship a first imported-inventory contract that can materialize external fleet nodes into the hosted control plane. | GOAL-02 | must | The user explicitly asked for a stronger cost-control and hosted story that can bridge real cloud fleets. |
| FR-04 | Port must surface persisted registration and imported inventory through canonical operator-visible machine or fleet outputs. | GOAL-02, GOAL-03 | must | Durable state is not useful if operators cannot inspect it through the product surface. |
| FR-05 | Port must document the shipped durable registration and inventory workflow together with the explicit boundaries that remain after the slice. | GOAL-03 | should | Discoverability and honest limits are required before the hosted lane can be treated as a product surface. |
| FR-06 | Port must expose one shared durable hosted registry and imported-inventory contract across `port-model` and `port-hosted-protocol`. | GOAL-01, GOAL-02 | must | The hosted runtime, CLI, and future SDK work need one canonical fleet-state schema instead of ad hoc payloads. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Persisted registration and imported inventory must remain deterministic and inspectable in CLI proofs and tests. | GOAL-01, GOAL-02, GOAL-03 | must | Hosted fleet regressions are difficult to reason about without deterministic evidence. |
| NFR-02 | Restart, freshness-expiry, and import mismatch failures must include explicit control-plane context and node detail. | GOAL-01, GOAL-02 | must | Operators need to distinguish stale state from missing nodes or import drift. |
| NFR-03 | The first voyage must end with executable stories and recorded evidence, not a design-only backlog. | GOAL-01, GOAL-02, GOAL-03 | should | The user asked for continuous autonomous delivery, not a pause at planning. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prefer Rust tests for registration persistence, heartbeat freshness, and imported inventory resolution.
- Use canonical CLI proofs that survive a control-plane restart and still surface the expected fleet state.
- Record docs/help proof that the durable registration and import workflow is discoverable and that remaining policy limits stay explicit.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing hosted control-plane and node-agent HTTP path is still the right carrier for durable registration refresh. | A deeper hosted transport redesign may be required. | Validate in the first contract story. |
| Operators can accept a bootstrap import contract before Port supports live cloud-provider discovery. | The first imported inventory slice may need to be broader sooner. | Validate in planning and docs review. |
| Deterministic-first-fit remains the only shipped placement policy after this slice. | The epic may need to pull in broader scheduler work sooner. | Reassess after imported inventory is visible. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should imported inventory live as a file-backed contract first or be fetched from a live external API in the first slice? | Product/Architecture | Open |
| What freshness window is sufficient before a persisted node becomes ineligible for routing or placement? | Architecture | Open |
| Which CLI surface should publish imported fleet inventory without inventing a second hosted-only vocabulary? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The board contains executable stories for durable registration persistence, freshness, imported inventory, and operator publication.
- [ ] A repository-local proof shows hosted node state surviving a control-plane restart or being marked stale through an explicit freshness rule.
- [ ] Operators can inspect imported fleet state and the current fleet-policy limits through the canonical Port surface and docs.
<!-- END SUCCESS_CRITERIA -->
