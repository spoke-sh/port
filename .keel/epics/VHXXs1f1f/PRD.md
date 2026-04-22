# Repair Hosted Control-Plane Placement Resolution - Product Requirements

## Problem Statement

Hosted control-plane read paths still stall or return malformed state when stored placement drifts or disappears, even while live node-agent and K3s truth remain healthy. That blocks cluster status, kubeconfig handoff, and auto-recovery.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Return truthful hosted machine and service state even when stored placement is stale, missing, or temporarily inconsistent. | Hosted `machine status`, `machine list`, and service-status surfaces use live node-agent truth or explicit degraded status instead of `malformed` when the guest/runtime is still live. | First voyage |
| GOAL-02 | Keep hosted cluster readiness and kubeconfig handoff observable and non-blocking under placement drift. | `cluster status` returns bounded, degraded readiness detail while `cluster kubeconfig` fails only on the kubeconfig handoff boundary. | First voyage |
| GOAL-03 | Make placement/inventory stalls diagnosable enough for rollout and auto-recovery operators to trust the control plane again. | Metrics, logs, and regression tests expose placement repair, alias canonicalization, and timeout isolation behavior. | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs hosted Port clusters and debugs recovery or rollout failures. | Truthful status surfaces that distinguish healthy runtime from control-plane cache drift. |
| Hosted Control-Plane Maintainer | Owns Port's hosted routing, inventory, and recovery implementation. | A design that keeps placement repair out of fragile synchronous request paths. |
| Downstream Rollout Automation | Calls `port cluster status`, `port cluster kubeconfig`, and machine/service routes during bootstrap or recovery. | Bounded, machine-readable failure modes that degrade cleanly instead of wedging. |

## Scope

### In Scope

- [SCOPE-01] Live-first hosted machine and service resolution that prefers live node-agent truth when stored placement is stale or absent.
- [SCOPE-02] Background placement reconciliation and canonicalization of stored placement records outside synchronous read paths.
- [SCOPE-03] A split hosted cluster-readiness model that separates machine/API visibility from kubeconfig handoff.
- [SCOPE-04] Logging, metrics, and regression coverage for placement stalls, repair behavior, and timeout isolation.

### Out of Scope

- [SCOPE-05] New scheduler policy, host-group placement strategy, or storage semantics beyond current hosted placement repair.
- [SCOPE-06] Downstream `infra` manifest, Flux, or post-handoff reconciliation changes.
- [SCOPE-07] Hosted auth-model redesign or a second operator/debug API surface.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Hosted machine, service, and guest-route status paths must use live node-agent truth ahead of stale or missing stored placement, returning explicit degraded detail instead of `malformed` when the runtime is still reachable. | GOAL-01, GOAL-03 | must | Read paths should report runtime truth, not cache accidents. |
| FR-02 | Placement repair must move out of synchronous read handlers into an explicit reconcile path triggered by startup, node registration, and lifecycle events, with canonical node-name persistence. | GOAL-01, GOAL-03 | must | Write-on-read repair makes stalls and drift harder to reason about. |
| FR-03 | Hosted cluster readiness must report machine/API visibility separately from kubeconfig handoff so `cluster status` can degrade without blocking on kubeconfig guest-exec. | GOAL-02 | must | Rollout needs bounded, truthful readiness rather than one opaque failure gate. |
| FR-04 | Port must emit actionable observability and regression coverage for placement drift, alias repair, and control-plane timeout isolation. | GOAL-03 | must | This class of failure needs direct instrumentation and tests to stay fixed. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Hosted control-plane request paths must stay bounded: one bad machine or service probe cannot wedge `list`, `status`, or cluster readiness for the whole fleet. | GOAL-01, GOAL-02, GOAL-03 | must | Fleet truth surfaces are only useful if they fail partially instead of globally. |
| NFR-02 | The design must not reintroduce hosted control-plane self-recursion or new synchronous placement writes inside read handlers. | GOAL-01, GOAL-03 | must | The previous production failure mode came directly from that coupling. |
| NFR-03 | Existing CLI surfaces remain canonical; new fidelity is additive through richer status detail, not a second operator workflow. | GOAL-02, GOAL-03 | must | Port should fix the existing product seam, not escape it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Live placement fallback | Targeted hosted-control-plane and runtime tests | Machine/service status tests proving live-first fallback under missing or stale placement |
| Readiness split | Hosted cluster-status and kubeconfig tests | Tests and CLI proofs showing degraded `cluster status` and isolated kubeconfig failures |
| Observability and stall isolation | Regression tests plus log/metric inspection | Tests for timeout isolation, alias repair, and emitted repair/stall signals |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Node-agent and runtime truth are available often enough to act as the live recovery authority when stored placement drifts. | Live-first routing could still fail too often and need a larger inventory redesign. | Validate with hosted-control-plane route tests and production repros. |
| Splitting readiness from kubeconfig handoff is compatible with current downstream callers as long as the CLI surface stays the same. | Consumers may require a coordinated field-level rollout. | Keep the surface additive and validate against current rollout flows. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much live candidate-node probing is acceptable before `list_machines` becomes too expensive under larger fleets? | Epic owner | Open |
| Some stale placements may point at nodes that are both alias-drifted and temporarily unreachable. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted machine and service status no longer report `malformed` solely because stored placement drifted while live node-agent/runtime truth is healthy.
- [ ] `port cluster status --format json` returns bounded degraded readiness detail when kubeconfig handoff fails, instead of hanging or collapsing the whole status surface.
- [ ] Placement repair, alias canonicalization, and timeout isolation are covered by logs/metrics and regression tests.
- [ ] The board contains executable stories with authored requirements and no placeholder planning content.
<!-- END SUCCESS_CRITERIA -->
