# Persistent Registration And Inventory Sync - Software Requirements Specification

> Define and deliver persistent node registration, heartbeat freshness, and
> imported fleet inventory contracts for the hosted control plane.

**Epic:** [1vzUnI000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Persist hosted node registration and freshness state under the
  control plane.
- [SCOPE-02] Refresh persisted registration from `port node-agent serve`
  without introducing a second hosted auth or transport path.
- [SCOPE-03] Materialize a first imported fleet inventory contract into the
  hosted control plane.
- [SCOPE-04] Surface persisted registration, freshness, and imported inventory
  through canonical operator-visible fleet or machine output and docs.

### Out of Scope

- [SCOPE-05] Autoscaling, rebalancing, or new scheduler policies beyond the
  current deterministic-first-fit contract.
- [SCOPE-06] Live cloud-provider discovery APIs for every provider in one
  voyage.
- [SCOPE-07] Hosted-only command families that replace `port machine`,
  `port node-agent`, or `port control-plane`.
- [SCOPE-08] Multi-tenant auth, quotas, billing, or control-plane HA.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The existing hosted control-plane and node-agent HTTP routes remain the canonical path for registration refresh. | dependency | The voyage would need a deeper hosted transport redesign before implementation could proceed safely. |
| Imported inventory can begin as a file-backed or model-backed contract before Port integrates live cloud discovery. | assumption | The voyage would need to broaden into provider-specific APIs immediately, increasing scope significantly. |
| Current hosted placement and service scheduling can continue to consume deterministic-first-fit inputs after durable inventory lands. | assumption | The voyage would need to pull policy redesign into the same slice. |

## Constraints

- Keep one canonical hosted auth model; do not add a second secret/token path
  for imported inventory or heartbeat refresh.
- Treat stale persisted registrations explicitly; never silently route through
  a node that has exceeded the freshness window.
- Keep canonical CLI vocabulary coherent; prefer extending `machine`,
  `control-plane`, or `node-agent` output over inventing a hosted-only fleet
  dialect.
- Preserve repository-local proofability through Rust tests and CLI proofs run
  under `keel verify`.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must persist hosted node registration and freshness state so the control plane can recover the fleet view after restart. | SCOPE-01 | FR-01 | rust tests + CLI proof |
| SRS-02 | `port node-agent serve` must refresh persisted registration and heartbeat freshness through the existing hosted transport and auth contract. | SCOPE-02 | FR-02 | rust tests + hosted runtime proof |
| SRS-03 | Port must define and materialize an imported fleet inventory contract that the hosted control plane can merge with configured node inventory. | SCOPE-03 | FR-03 | rust tests + CLI proof |
| SRS-04 | Canonical hosted inspection output must surface persisted registration state, freshness, and imported inventory provenance instead of hiding them behind generic hosted status. | SCOPE-04 | FR-04 | rust tests + CLI proof |
| SRS-05 | CLI help, README, and hosted docs must publish the durable registration and imported inventory workflow together with the limits that remain after this slice. | SCOPE-04 | FR-05 | doc/help proof + demo |
| SRS-06 | Shared `port-model` and `port-hosted-protocol` contracts must represent durable hosted registry and imported inventory state through one canonical node identity schema. | SCOPE-01, SCOPE-03 | FR-06 | rust tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Persisted registration recovery and freshness expiry must behave deterministically for the same stored state and current time inputs. | SCOPE-01, SCOPE-02 | NFR-01 | rust tests |
| SRS-NFR-02 | Restart, staleness, and import mismatch failures must include explicit control-plane context and affected node detail. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | rust tests + CLI proof |
| SRS-NFR-03 | Planning and implementation must end with executable stories and board evidence, not a second planning-only hosted backlog. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | board review + story verification |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Durable Hosted Registry Contract | Shared model and hosted protocol can represent persisted node records, freshness, and imported inventory provenance. | SRS-06, SRS-NFR-02 |
| Persist Hosted Registration And Freshness | Control plane and node agent store, reload, and refresh persisted registration across restart. | SRS-01, SRS-02, SRS-NFR-01, SRS-NFR-02 |
| Materialize Imported Fleet Inventory | Operators can load imported node membership into the hosted control-plane state and inspect merge eligibility. | SRS-03, SRS-NFR-02 |
| Surface Durable Hosted Fleet State | Canonical hosted inspection output surfaces persisted registration, freshness, and imported provenance explicitly. | SRS-04, SRS-NFR-02 |
| Publish Durable Hosted Fleet Workflow | CLI/docs/help and proof publish the durable registration and imported inventory workflow with current limits. | SRS-05, SRS-NFR-03 |
