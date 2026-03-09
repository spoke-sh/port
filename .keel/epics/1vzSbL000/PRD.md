# Hosted Scheduler And Multi-Node Services - Product Requirements

> Expand Port's hosted lane from a single-node demo into a host-group-aware
> scheduler and service plane that can place and operate workloads across
> prepared nodes without inventing a second CLI or service model.

## Problem Statement

Port now ships a live hosted control plane, node agent, and hosted service
execution path, but the hosted lane is still fundamentally a single-node demo.
Compared with the requested Slicer-class direction, Port still lacks:

- host-group-aware placement policy
- multi-node scheduling for services and sandboxes
- operator-visible placement state and admission failures
- a coherent published workflow for running the same `port service` surface
  against a prepared hosted fleet

Without those capabilities, Port cannot tell a credible hosted story for
cost-aware cloud operation across prepared nodes, even though the lower-level
control plane and guest transport now exist.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Add the first host-group-aware scheduler slice for hosted services and sandboxes. | Operators can target a host group and Port selects an eligible node with recorded placement evidence. | First voyage complete |
| GOAL-02 | Preserve one canonical operator surface. | Hosted placement continues to use `port service` and existing status/list surfaces rather than introducing hosted-only verbs. | First voyage complete |
| GOAL-03 | Make multi-node hosted behavior explicit and inspectable. | CLI/docs/status surfaces show selected node, host group, and placement failure detail. | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs Port on prepared cloud nodes and cares about cost, placement, and fleet behavior. | A coherent way to place services onto eligible nodes without direct node micromanagement. |
| Hosted Port Builder | Extends the control plane and node-agent runtime. | Stable contracts for host groups, scheduler selection, and placement evidence. |
| Application Operator | Runs sandboxes or long-lived services on hosted Port. | One `port service` workflow that works locally or across hosted nodes. |

## Scope

### In Scope

- [SCOPE-01] Host-group contracts in the shared model and hosted inventory.
- [SCOPE-02] Scheduler/admission policy for selecting a prepared node for a
  hosted service or sandbox launch.
- [SCOPE-03] Hosted `port service apply|list|status|stop` updates that surface
  placement state, selected node, and host-group identity.
- [SCOPE-04] Operator docs, help text, and evidence for the first multi-node
  hosted workflow.

### Out of Scope

- [SCOPE-05] Autoscaling, fleet rebalancing, spot/price optimization, or quota
  control.
- [SCOPE-06] Full fairness policy, priorities, or preemption.
- [SCOPE-07] Multi-tenant RBAC, billing, or external service catalogs.
- [SCOPE-08] Restart policies, health checks, or secret-backend hardening
  beyond what the hosted service lane already ships.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define host-group and scheduler-policy contracts that can be carried in the shared model, hosted inventory, and placement evidence. | GOAL-01, GOAL-03 | must | Multi-node service placement needs an explicit contract before runtime behavior can stay coherent. |
| FR-02 | Port must implement the first hosted scheduler slice for `port service apply --kind service|sandbox`, selecting an eligible prepared node from a target host group and returning explicit failure detail when no node qualifies. | GOAL-01, GOAL-02, GOAL-03 | must | Service placement is the core missing capability between the current single-node demo and a credible hosted lane. |
| FR-03 | Port must keep `port service list|status|stop` as the canonical operator surface while surfacing selected node, host group, and placement/runtime state through those existing verbs. | GOAL-02, GOAL-03 | must | Operators should not have to learn a second hosted-only service vocabulary. |
| FR-04 | Port must publish an operator workflow and proof for multi-node hosted placement, including the explicit limits that remain after this slice. | GOAL-02, GOAL-03 | should | Product credibility depends on discoverable and verifiable behavior, not runtime code alone. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Scheduler behavior must be deterministic and operator-visible for the same input inventory. | GOAL-01, GOAL-03 | must | Placement debugging is impossible if node selection feels arbitrary or opaque. |
| NFR-02 | Docs, help text, and board artifacts must distinguish the shipped multi-node scheduler slice from still-planned work such as autoscaling, broader policy, and fleet management. | GOAL-03 | must | Prevents the hosted roadmap from overstating what the first scheduler slice does. |
| NFR-03 | The first voyage must leave the board with executable implementation stories rather than another hosted design dead end. | GOAL-01, GOAL-02 | should | The user objective requires continued execution, not another purely conceptual backlog. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use shared-model and runtime tests to prove host-group and placement
  contracts.
- Prefer hosted CLI proofs and runtime inspection for scheduler selection and
  placement failure detail.
- Publish docs/help proofs to show the multi-node workflow and remaining limits
  clearly.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Prepared hosted nodes remain the first deployment target for scheduler work. | Placement logic may need a second substrate abstraction sooner. | Validate in the first implementation story. |
| One host group and one selected node are sufficient for the first service placement slice. | Broader policies such as weighted groups or spreading may be needed earlier. | Reassess after the first voyage. |
| Existing hosted service runtime state can carry placement metadata without inventing a new service model. | The operator surface may need a wider runtime schema change. | Validate in the placement/runtime story. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first scheduler slice place both services and sandboxes, or services first with sandbox reuse verified in the same runtime path? | Product/Architecture | Open |
| How much node health/admission signal is required before scheduler output is credible enough for operators? | Architecture | Open |
| Does multi-node placement need a dedicated `service reschedule` action, or is `apply` plus `stop` sufficient for the first slice? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Host-group-aware scheduler work is decomposed into executable stories with traceable verification paths.
- [ ] The first voyage defines a concrete hosted workflow where `port service` can target a host group and Port records which node was selected.
- [ ] Operator docs and help text make the shipped scheduler slice and remaining policy limits explicit.
<!-- END SUCCESS_CRITERIA -->
