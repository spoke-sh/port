# Hosted Fleet Recovery Ladder And Host Recycle - Product Requirements

## Problem Statement

Once Port can detect a wedged microVM, it still has no automatic recovery path — operators must SSH to the host and intervene manually, which is slow and scales poorly. Port owns the runtime and the process graph that lifecycle actions touch, so guest-level recovery must live inside Port. Deliver a per-cluster opt-in recovery ladder where tier-1 (guest restart) and tier-2 (overlay recreate) fire inside Port against the runtime root, and tier-3 (host recycle) surfaces as a structured signal for an external consumer to act on — Port does not call cloud-provider APIs. Add serialization against in-flight human lifecycle operations, a sticky `recovery_exhausted` terminal state, and an explicit `port machine unfence` reset path. Keep `enabled = false` by default so production has to opt in per cluster.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Resolve the problem described above for the primary user. | A measurable outcome is defined for this problem | Target agreed during planning |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | The person or team most affected by the problem above. | A clearer path to the outcome this epic should improve. |

## Scope

### In Scope

- [SCOPE-01] Per-cluster recovery configuration block in `port.toml` with an `enabled` opt-in flag (default `false`), threshold knobs, and tier promotion counters.
- [SCOPE-02] Tier-1 guest restart action: when the wedge detector flags a guest-side wedge on an opted-in cluster, the node-agent executes `port machine stop` followed by `port machine launch` against the same runtime root.
- [SCOPE-03] Tier-2 guest recreate action: drop the machine's rootfs overlay and relaunch; graceful skip with a `tier_2_skipped_no_overlay` event when the machine has no overlay configured.
- [SCOPE-04] Tier-3 escalation signal: when tier-1/tier-2 exhaust, Port sets `recovery_state = "awaiting_tier_3_host_recycle"` on the wedged machine and emits a structured `tier_3_escalation` event carrying machine, host, timestamp, and the last failed tier outcome. The signal auto-clears when node-agent re-registration and a fresh guest heartbeat return — no response path from the consumer back into Port is required.
- [SCOPE-05] Sticky `recovery_exhausted` terminal state that survives `window_seconds` rollovers; cleared only via `port machine unfence --machine X` or a successful operator-driven `port machine launch` that produces a Live guest-agent heartbeat.
- [SCOPE-06] Structured event log for each tier transition (start, success, failure, skip, escalation) with machine, tier, timestamp, and outcome, persisted alongside the existing recovery surface so operators and downstream consumers can correlate with Kubernetes node-ready transitions.
- [SCOPE-07] Serialization against in-flight human lifecycle operations: if `port machine stop/launch/up` is in-flight on the target machine, recovery skips that tick and re-evaluates on the next settling interval instead of cancelling the human's op.
- [SCOPE-08] End-to-end integration tests: a simulated guest-side wedge converges under tier-1 without operator intervention; a simulated node-side wedge escalates to `awaiting_tier_3_host_recycle` with the event emitted, and returns to `ok` when simulated host return (re-registration + fresh guest heartbeat) is observed.

### Out of Scope

- [SCOPE-09] Any direct call from Port into a cloud-provider API (AWS EC2 `RebootInstances`, GCP/Azure equivalents). The tier-3 host recycle action is owned by the consumer of the escalation signal (spoke-sh/infra, operators, systemd watchers); Port only signals.
- [SCOPE-10] SSH-based `systemctl restart port-node-agent` or any other remote shell execution initiated by Port as part of recovery. Remote shell actions belong to the consumer side.
- [SCOPE-11] Cross-cell rebalancing if a host is permanently unhealthy — different problem class.
- [SCOPE-12] Guest-side kernel or userspace watchdog — belongs in the Spoke guest image, not Port.
- [SCOPE-13] Alerting, dashboards, or UI for recovery events — consumers read the JSON surface and structured events today.
- [SCOPE-14] Single-tenant-host gating inside Port. The tier-3 signal is per-machine; the consumer decides whether the blast radius of rebooting the host is acceptable before acting.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Deliver the primary user workflow for this epic end-to-end. | GOAL-01 | must | Establishes the minimum functional capability needed to achieve the epic goal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01 | must | Keeps operations stable and makes regressions detectable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
