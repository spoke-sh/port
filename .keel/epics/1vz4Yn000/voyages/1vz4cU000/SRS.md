# Hosted API And Inventory - Software Requirements Specification

> Define and begin implementing the first authenticated hosted control surface for node-aware inventory, lifecycle control, and guest bridge attachment.

**Epic:** [1vz4Yn000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Define the first hosted auth token and API identity contract.
- [SCOPE-02] Define node and host-group inventory vocabulary for hosted Port.
- [SCOPE-03] Define hosted `machine list|status|stop` contracts that preserve
  the existing CLI model.
- [SCOPE-04] Define the first hosted guest-connect bridge primitive that keeps
  the current guest protocol intact.
- [SCOPE-05] Keep README, hosted docs, operator docs, and CLI help aligned with
  the first hosted-control slice.
- Out of scope: full SDK packaging, secrets, services, sandboxes, monitoring,
  detached forwards, and Unix-socket forwarding.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Token auth is sufficient for the first hosted-control slice. | Assumption | The voyage may need to widen into broader identity design. |
| Node and host-group vocabulary can cover the first hosted placement contract. | Assumption | Placement modeling may need additional scheduler semantics early. |
| The existing guest protocol should remain canonical in hosted mode. | Constraint | Hosted guest features would fragment into substrate- or API-specific protocols. |
| Port's local `machine` vocabulary should remain the canonical operator model. | Constraint | Hosted Port would diverge into a second operator surface. |

## Constraints

- Keep one canonical CLI and one canonical guest protocol.
- Prefer additive hosted routing over inventing hosted-only command names.
- Fail fast on unimplemented hosted behavior instead of implying a hidden
  control plane exists.
- Sequence monitoring, secrets, services, sandboxes, detached forwarding, and
  SDK work after the first hosted-control foundation.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a hosted auth token and API identity contract for the first control-plane slice. | SCOPE-01, SCOPE-05 | FR-01 | doc review + contract inspection |
| SRS-02 | Port must define node and host-group inventory contracts that map hosted ownership and placement onto Port's shared machine model. | SCOPE-02, SCOPE-05 | FR-02 | doc review + contract inspection |
| SRS-03 | Port must define hosted machine list, status, and stop contracts that preserve the existing CLI verbs while routing through hosted ownership. | SCOPE-03, SCOPE-05 | FR-03 | design review + CLI/doc review |
| SRS-04 | Port must define the first hosted guest-connect bridge primitive that preserves the current guest protocol for later hosted guest operations. | SCOPE-04, SCOPE-05 | FR-04 | design review + contract inspection |
| SRS-05 | Port must produce implementation-ready stories for auth/API identity, node inventory, hosted lifecycle control, and guest bridge follow-on work. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | FR-01 | board review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The voyage must preserve one canonical CLI and one canonical guest protocol across local and hosted Port. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | design review + doc review |
| SRS-NFR-02 | Hosted docs and CLI/help text must distinguish shipped local behavior from planned or partial hosted behavior. | SCOPE-01, SCOPE-03, SCOPE-05 | NFR-02 | CLI/doc review |
| SRS-NFR-03 | The voyage must leave a coherent ordered implementation set for monitoring, secrets, services, sandboxes, detached forwarding, Unix-socket forwarding, and SDK work. | SCOPE-05 | NFR-03 | board review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Planned Outcome | Requirements |
|-------|-----------------|--------------|
| Define Hosted Auth And API Contract | Publish the first token and API identity contract, including how the CLI targets a hosted control plane without inventing new verbs. | SRS-01, SRS-NFR-01, SRS-NFR-02 |
| Define Hosted Node Inventory Model | Publish node and host-group contracts that can back placement, ownership, and later scheduling work. | SRS-02, SRS-NFR-03 |
| Define Hosted Machine Lifecycle Surface | Extend the shared machine contract so hosted list, status, and stop can be modeled explicitly and surfaced through the canonical CLI/docs. | SRS-03, SRS-NFR-01, SRS-NFR-02 |
| Define Hosted Guest Bridge Attach Contract | Publish the first hosted guest-connect route that preserves the existing guest protocol and sequences later hosted `exec|copy|pty|logs|forward` work. | SRS-04, SRS-NFR-01, SRS-NFR-03 |
| Sequence Hosted Follow-On Work | Record the ordered backlog and operator messaging for monitoring, secrets, services, sandboxes, detached forwarding, Unix-socket forwarding, and SDK work after this voyage. | SRS-05, SRS-NFR-03 |
