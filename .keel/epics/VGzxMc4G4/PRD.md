# Hosted Fleet Recovery Ladder And Host Recycle - Product Requirements

## Problem Statement

Once Port can detect a wedged microVM, it still has no automatic recovery path — operators must SSH to the host and intervene manually, which is slow and scales poorly. Port owns the runtime and the process graph that lifecycle actions touch, so recovery must live inside Port. Deliver a per-cluster opt-in recovery ladder (tier-1 guest restart, tier-2 overlay recreate, tier-3 host recycle with a single-tenant gate), a per-provider host_reboot integration reused by tier-3, serialization against in-flight human lifecycle operations, a sticky recovery_exhausted terminal state, and an explicit port machine unfence reset path. Keep enabled=false by default so production has to opt in per cluster.

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
- [SCOPE-04] Tier-3 host recycle action: restart the host's Firecracker and node-agent process graph via a host-provider reboot, gated on `host.single_tenant_host == true` or exactly one machine currently placed on the host; otherwise `recovery_state = "suppressed_multi_tenant"`.
- [SCOPE-05] Per-provider `host_reboot` integration: AWS EC2 `RebootInstances` for `provider = aws` hosts; SSH `systemctl restart port-node-agent` for `provider = ssh` hosts. One reusable interface consumed by tier-3.
- [SCOPE-06] Sticky `recovery_exhausted` terminal state that survives `window_seconds` rollovers; cleared only via `port machine unfence --machine X` or a successful operator-driven `port machine launch` that produces a Live guest-agent heartbeat.
- [SCOPE-07] Structured event log for each tier transition (start, success, failure, skip) with machine, tier, timestamp, and outcome, persisted alongside the existing recovery surface so operators can correlate with Kubernetes node-ready transitions.
- [SCOPE-08] Serialization against in-flight human lifecycle operations: if `port machine stop/launch/up` is in-flight on the target machine, recovery skips that tick and re-evaluates on the next settling interval instead of cancelling the human's op.
- [SCOPE-09] End-to-end integration tests: a simulated guest-side wedge converges under tier-1 without operator intervention, and a simulated node-side wedge on a single-tenant host converges under tier-3.

### Out of Scope

- [SCOPE-10] Cross-cell rebalancing if a host is permanently unhealthy — different problem class.
- [SCOPE-11] Guest-side kernel or userspace watchdog — belongs in the Spoke guest image, not Port.
- [SCOPE-12] Host providers beyond `aws` and `ssh` — add as demand materialises.
- [SCOPE-13] Alerting, dashboards, or UI for recovery events — consumers read the JSON surface and structured events today.

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
