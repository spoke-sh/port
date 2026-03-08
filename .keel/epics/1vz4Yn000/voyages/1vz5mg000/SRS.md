# Hosted Runtime And Service Expansion - Software Requirements Specification

> Implement the first hosted runtime control path, then sequence forwarding,
> monitoring, secrets, services, sandboxes, and SDK surfaces on top of the
> hosted foundation contracts.

**Epic:** [1vz4Yn000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Implement the first hosted control-plane runtime path for
  `machine list|status|stop` on top of the existing hosted auth, inventory, and
  lifecycle contracts.
- [SCOPE-02] Implement the first hosted guest runtime path for canonical
  `guest exec|copy|pty|logs|forward` operations on top of the hosted guest
  attach contract.
- [SCOPE-03] Add detached forwarding and Unix-socket forwarding on top of the
  hosted guest runtime path without inventing a second forwarding command
  family.
- [SCOPE-04] Add hosted monitoring and `top` surfaces grounded in hosted node
  inventory and runtime ownership.
- [SCOPE-05] Add hosted secrets, services, and sandboxes on top of the hosted
  runtime, guest attach, and forwarding foundation.
- [SCOPE-06] Publish supported SDK and API client surfaces for hosted machine,
  guest, and service operations once the runtime interfaces stabilize.

Out of scope:

- full multi-tenant auth and RBAC
- generalized scheduler policy beyond the existing node and host-group model
- billing, quotas, and control-plane tenancy management

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted auth, inventory, lifecycle, and guest-attach contracts from `1vz4cU000` remain the canonical foundation. | dependency | This voyage would have to redefine control ownership and would fragment the hosted model. |
| The existing guest protocol remains the only canonical request/response framing for guest operations. | assumption | Hosted guest work would diverge from the local lane and require a redesign. |
| A node-agent-owned runtime remains the host-local execution authority for hosted machines. | assumption | Hosted lifecycle, monitoring, services, and forwarding order would need to change. |
| The control plane can expose machine, guest, and service APIs without becoming Firecracker-specific. | assumption | The SDK and API client scope would need a different abstraction boundary. |

## Constraints

- One canonical CLI must remain intact across local and hosted Port.
- One canonical guest protocol must remain intact across local and hosted Port.
- Hosted docs and operator surfaces must continue to distinguish shipped
  behavior, partial runtime behavior, and planned follow-on work.
- The runtime path must land before productized monitoring, secrets, services,
  sandboxes, or SDK work.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must implement the first hosted control-plane runtime path for canonical `machine list`, `status`, and `stop` operations. | SCOPE-01 | FR-03 | Rust tests + CLI proof |
| SRS-02 | Port must implement the first hosted guest runtime path for canonical `guest exec`, `copy`, `pty`, `logs`, and `forward` operations without redefining the guest protocol. | SCOPE-02 | FR-04 | Rust tests + CLI proof |
| SRS-03 | Port must add detached forwarding and Unix-socket forwarding through the canonical forwarding surface after the hosted guest runtime path exists. | SCOPE-03 | FR-04 | Rust tests + CLI proof |
| SRS-04 | Port must expose hosted monitoring and `top` surfaces grounded in hosted runtime ownership and node inventory. | SCOPE-04 | FR-05 | Rust tests + CLI proof |
| SRS-05 | Port must add hosted secrets, services, and sandboxes on top of the hosted runtime and guest foundation. | SCOPE-05 | FR-05 | design review + CLI proof |
| SRS-06 | Port must publish supported SDK and API client surfaces for hosted machine, guest, and service operations after the runtime APIs stabilize. | SCOPE-06 | FR-06 | client/API review + examples |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The voyage must preserve one canonical CLI and one canonical guest protocol across local and hosted Port. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-06 | NFR-01 | design review + CLI review |
| SRS-NFR-02 | Hosted docs and operator surfaces must distinguish shipped runtime behavior from partial or planned hosted work at every step of the sequence. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05, SCOPE-06 | NFR-02 | doc review + CLI review |
| SRS-NFR-03 | The board must preserve the implementation order `runtime -> guest runtime -> forwarding -> monitoring -> secrets/services/sandboxes -> SDK`. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05, SCOPE-06 | NFR-03 | board review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Planned Outcome | Requirements |
|-------|-----------------|--------------|
| Implement Hosted Control Plane Runtime Path | Land the first hosted machine list/status/stop runtime driver and control-plane path. | SRS-01, SRS-NFR-01, SRS-NFR-02, SRS-NFR-03 |
| Implement Hosted Guest Operations Runtime Path | Land hosted guest exec/copy/pty/logs/forward over the existing guest protocol. | SRS-02, SRS-NFR-01, SRS-NFR-02, SRS-NFR-03 |
| Add Detached And Unix-Socket Forwarding | Extend the forwarding surface with detached and Unix-socket modes after hosted guest runtime exists. | SRS-03, SRS-NFR-01, SRS-NFR-03 |
| Add Hosted Monitoring And Top | Add hosted monitoring and `top` surfaces once runtime ownership and guest brokerage exist. | SRS-04, SRS-NFR-02, SRS-NFR-03 |
| Add Hosted Secrets Services And Sandboxes | Add secrets plus service/sandbox lifecycle surfaces on top of runtime, forwarding, and monitoring foundations. | SRS-05, SRS-NFR-02, SRS-NFR-03 |
| Publish Hosted SDK And API Clients | Ship supported SDK and API client surfaces after the control-plane runtime APIs stabilize. | SRS-06, SRS-NFR-01, SRS-NFR-02, SRS-NFR-03 |
