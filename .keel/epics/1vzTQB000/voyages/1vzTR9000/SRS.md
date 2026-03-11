# Registered Nodes And Machine Launch Placement - Software Requirements Specification

> Let a node agent register with the hosted control plane and route canonical machine launch onto an eligible registered node with operator-visible placement detail.

**Epic:** [1vzTQB000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Define a hosted node registration contract and persisted
  registration state for the control plane.
- [SCOPE-02] Let `port node-agent serve` register and refresh one node against a
  hosted control plane without `--node-binding`.
- [SCOPE-03] Route hosted `port machine launch` onto an eligible registered
  node and persist placement evidence.
- [SCOPE-04] Surface registered-node and placement detail through canonical
  machine output and docs/help.

### Out of Scope

- [SCOPE-05] Autoscaling, fleet rebalance, or weighted/spread placement.
- [SCOPE-06] Full node health scoring, restart automation, or reconciliation
  loops.
- [SCOPE-07] Multi-tenant RBAC, billing, or external node catalogs.
- [SCOPE-08] Replacing the existing config-backed hosted inventory model in one
  voyage.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted nodes continue to be declared in the shared model with capability metadata and runtime roots. | dependency | Registration would need a broader discovery/catalog system before placement could stay coherent. |
| Existing control-plane and node-agent HTTP transport can carry registration traffic without a second hosted auth model. | dependency | A larger hosted protocol redesign would be required before execution can resume. |
| One selected registered node per machine launch is sufficient for the first placement slice. | assumption | Replicated or fallback machine placement would need a wider lifecycle contract immediately. |

## Constraints

- Keep canonical `port machine` verbs intact; do not introduce hosted-only
  machine launch aliases.
- Make registered-node placement deterministic for the same input state.
- Keep failure detail explicit when a node is missing, stale, or ineligible.
- Preserve repo-local proofability under `keel verify` and the current hosted
  demo environment.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a hosted node registration contract that the control plane can persist and inspect independently of transient `--node-binding` flags. | SCOPE-01 | FR-01 | rust tests + CLI proof |
| SRS-02 | `port node-agent serve` must register and refresh one configured node against a hosted control plane using the shared hosted transport and auth model. | SCOPE-02 | FR-02 | rust tests + hosted runtime proof |
| SRS-03 | Hosted `port machine launch` must select one eligible registered node, route launch through that node, and persist the selected-node placement detail. | SCOPE-03 | FR-03 | rust tests + CLI proof |
| SRS-04 | Canonical hosted machine lifecycle output must surface registered-node identity, placement detail, and stale-registration failures instead of collapsing into generic hosted errors. | SCOPE-03, SCOPE-04 | FR-04 | rust tests + CLI proof |
| SRS-05 | CLI help, README, hosted docs, and proof must publish the registered-node hosted machine workflow and the limits that remain after this slice. | SCOPE-04 | FR-05 | doc/help proof + demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Registered-node placement must be deterministic for equal node registration and capability input. | SCOPE-02, SCOPE-03 | NFR-01 | rust tests |
| SRS-NFR-02 | Registration and placement failures must include explicit hosted route, node, and freshness detail. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | rust tests + CLI proof |
| SRS-NFR-03 | Docs/help text must distinguish the shipped registered-node slice from still-planned work such as autoscaling, broader fleet policy, and durable external inventory. | SCOPE-04 | NFR-02 | doc/help proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Registered Node Contract And State | Shared model, hosted protocol, and runtime state can represent registered nodes and freshness explicitly. | SRS-01, SRS-NFR-02 |
| Implement Node Agent Registration Refresh | Node agents register with the control plane and refresh their presence without `--node-binding`. | SRS-02, SRS-NFR-02 |
| Route Hosted Machine Launch Through Registered Nodes | Hosted machine launch selects an eligible registered node and records placement. | SRS-03, SRS-NFR-01, SRS-NFR-02 |
| Surface Registered Placement Through Machine Commands | Canonical machine list/status/monitor/stop show selected node and stale-registration detail. | SRS-04, SRS-NFR-02 |
| Publish Registered Hosted Machine Workflow | Docs, help text, and proof show how registered hosted machine placement works and what still remains follow-on work. | SRS-05, SRS-NFR-03 |
