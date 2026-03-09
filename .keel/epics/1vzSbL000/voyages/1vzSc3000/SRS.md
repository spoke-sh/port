# Host Groups And Service Placement - Software Requirements Specification

> Define and land the first host-group-aware scheduler slice for hosted services and sandboxes, including explicit placement evidence and operator workflow.

**Epic:** [1vzSbL000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:

- [SCOPE-01] Define host-group and scheduler-policy contracts in the shared
  model, sample config, and hosted inventory/state surfaces.
- [SCOPE-02] Select an eligible prepared node from a target host group when a
  hosted `port service apply --kind service|sandbox` request executes.
- [SCOPE-03] Persist and surface placement metadata, including selected node,
  host group, and scheduler/admission detail, through the canonical `port
  service list|status|stop` workflow.
- [SCOPE-04] Publish help, docs, and proof for the first multi-node hosted
  operator workflow and the remaining limits after this slice.

Out of scope:

- [SCOPE-05] Autoscaling, fleet rebalancing, spread/anti-affinity, or weighted
  placement.
- [SCOPE-06] Service restart policy, health checks, or reconciliation loops.
- [SCOPE-07] Quota, billing, RBAC, or broader multi-tenant policy.
- [SCOPE-08] External secret stores or service catalog features.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted nodes continue to expose runtime roots and capability metadata through the current control-plane and node-agent contracts. | dependency | Scheduler selection would need a new inventory transport before this slice can ship. |
| Hosted service execution remains the canonical runtime path for proving multi-node placement. | assumption | Placement might need to move to another product surface first. |
| One selected node per service definition is sufficient for the first scheduler slice. | assumption | Multi-instance or replicated services would need a different runtime contract immediately. |

## Constraints

- Keep one canonical `port service` surface for local and hosted workflows.
- Make scheduler selection deterministic for the same inventory input.
- Fail with explicit placement detail when a host group or node is ineligible.
- Keep proof automation reliable under `keel verify`, including repo-local demo
  setups.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define host-group and scheduler-policy contracts that can be configured, diagnosed, serialized in hosted inventory, and attached to hosted service runtime state. | SCOPE-01, SCOPE-03 | FR-01 | rust tests + CLI proof |
| SRS-02 | Hosted `port service apply --kind service` and `--kind sandbox` must select one eligible node from the requested host group, using explicit admission checks and deterministic scheduler ordering. | SCOPE-02 | FR-02 | rust tests + hosted runtime proof |
| SRS-03 | Hosted `port service list`, `status`, and `stop` must surface placement metadata, selected node identity, host-group identity, and explicit placement or admission failures through the canonical service surface. | SCOPE-03 | FR-03 | rust tests + CLI proof |
| SRS-04 | CLI help, README, hosted docs, and operator proof must publish the multi-node hosted workflow and the limits that remain after this first scheduler slice. | SCOPE-04 | FR-04 | doc/help proof + demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Scheduler selection must be deterministic for equal inventory input and stable when no inventory fields change. | SCOPE-02, SCOPE-03 | NFR-01 | rust tests |
| SRS-NFR-02 | Placement failures must be operator-visible and distinguish missing host groups, ineligible nodes, and runtime launch failures. | SCOPE-02, SCOPE-03 | NFR-02 | rust tests + CLI proof |
| SRS-NFR-03 | Docs/help text must distinguish the shipped multi-node scheduler slice from still-planned work such as autoscaling, broader policy, and fleet management. | SCOPE-04 | NFR-02 | doc/help proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Host Group And Scheduler Contracts | Shared model, inventory, and doctor/help contracts can represent host groups and scheduler policy explicitly. | SRS-01, SRS-NFR-02 |
| Implement Hosted Service Placement Scheduler | Hosted service apply selects an eligible node deterministically and records placement state. | SRS-02, SRS-NFR-01, SRS-NFR-02 |
| Surface Placement State Through Canonical Service Commands | Hosted list/status/stop expose selected node, host group, and admission/runtime detail through the canonical service workflow. | SRS-03, SRS-NFR-02 |
| Publish Multi-Node Hosted Service Workflow | Docs, help text, and proof show how to target a host group and what still remains follow-on work. | SRS-04, SRS-NFR-03 |
