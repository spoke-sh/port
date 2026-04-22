# Recover Hosted Placement Truth Without Read-Path Stall - SRS

## Summary

Epic: VHXXs1f1f
Goal: Return live, partial hosted cluster truth when stored placement drifts or disappears, keep request paths non-blocking, and expose enough status fidelity for rollout and auto-recovery to trust the control plane.

## Scope

### In Scope

- [SCOPE-01] Make hosted machine list/status and hosted service status live-first: when stored placement is stale or missing but a live node-agent can still answer, Port returns runtime truth or explicit degraded detail instead of `malformed`.
- [SCOPE-02] Remove write-on-read placement repair from hosted request handlers and add a background/control-plane reconcile path that repairs placement from launch, registration, stop, and startup events while persisting canonical node names.
- [SCOPE-03] Split hosted cluster readiness into machine/runtime visibility, API visibility, node visibility, and kubeconfig handoff so `cluster status` can return degraded partial truth without blocking on kubeconfig guest-exec.
- [SCOPE-04] Add observability and regression coverage for placement repair, alias canonicalization, machine-list timeout isolation, and degraded cluster-status reporting.

### Out of Scope

- [SCOPE-05] New scheduler algorithms, host-group policy changes, or general hosted placement rebalancing beyond repairing existing machine placement truth.
- [SCOPE-06] Downstream `infra` Flux/bootstrap changes or non-Port rollout orchestration.
- [SCOPE-07] Hosted auth redesign, alternate transport protocols, or a second operator/debug API.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `list_machines` and `machine_status` must return live runtime truth when stored placement is stale or absent but a live node-agent route can still resolve the machine; these paths must avoid `malformed` responses caused only by missing stored placement. | SCOPE-01 | FR-01 | integration |
| SRS-02 | Hosted service-status and guest-route resolution must use live placement or candidate-node resolution before failing on missing stored placement, returning explicit degraded detail when live truth cannot be refreshed. | SCOPE-01 | FR-01 | integration |
| SRS-03 | Hosted request handlers must stop mutating placement state on read paths; placement repair happens through explicit reconcile hooks triggered by control-plane startup, node registration, and lifecycle events. | SCOPE-02 | FR-02 | unit |
| SRS-04 | The placement reconciler must canonicalize legacy node aliases to the configured node identity and persist repaired machine placements without requiring a user read path to touch the cache. | SCOPE-02 | FR-02 | unit |
| SRS-05 | `cluster status` must report machine/runtime visibility, API visibility, node visibility, and kubeconfig availability separately so a healthy API with broken kubeconfig handoff returns degraded partial truth instead of blocking or collapsing to one opaque failure. | SCOPE-03 | FR-03 | integration |
| SRS-06 | `cluster kubeconfig` must depend on the split readiness model: it may fail on kubeconfig handoff, but it must not hide already-established machine/API readiness behind the same generic failure mode as `cluster status`. | SCOPE-03 | FR-03 | integration |
| SRS-07 | Port must emit placement-repair and stall observability plus regression tests covering missing placement fallback, alias repair, machine-list timeout isolation, and degraded cluster status. | SCOPE-04 | FR-04 | unit |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted machine list/status fan-out must use bounded per-machine deadlines and partial aggregation so one bad machine route cannot wedge the entire fleet response. | SCOPE-01, SCOPE-03 | NFR-01 | integration |
| SRS-NFR-02 | Hosted control-plane request handlers must not call back into the hosted control plane or perform synchronous placement persistence on read paths. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-02 | unit |
| SRS-NFR-03 | New readiness or observability fidelity must remain additive on the canonical CLI surfaces; operators keep using the same `machine`, `service`, and `cluster` verbs. | SCOPE-03, SCOPE-04 | NFR-03 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
