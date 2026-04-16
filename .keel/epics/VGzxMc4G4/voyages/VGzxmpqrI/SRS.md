# Tier-1 Guest Restart And Attempt Accounting - SRS

## Summary

Epic: VGzxMc4G4
Goal: Deliver tier-1 guest restart action and attempt accounting: when guest-side wedge trigger fires on an opted-in cluster, serialize against human lifecycle ops, stop-then-launch the machine, increment recovery_attempts.tier_1, and emit structured events. Default off.

## Scope

### In Scope

- [SCOPE-01] Extend the `[clusters.<name>.recovery]` config block with `enabled: bool` (default `false`), `settle_seconds`, and the tier-1 attempt accounting fields consumed by this voyage.
- [SCOPE-02] When the detector writes a guest-side wedge on an opted-in cluster, the node-agent executes tier-1 recovery: `port machine stop` followed by `port machine launch` against the same runtime root.
- [SCOPE-07] Emit a structured recovery event (JSON) on each tier-1 attempt with `machine`, `tier`, `timestamp_unix_s`, and `outcome`; persisted so operators can correlate with Kubernetes transitions.
- [SCOPE-08] Serialize tier-1 against any in-flight human lifecycle operation on the same machine: if a human `stop/launch/up` is in progress, skip this tick and re-evaluate next interval.

### Out of Scope

- [SCOPE-03] Tier-2 overlay recreate — owned by voyage VGzxnR97R.
- [SCOPE-04] Tier-3 host recycle — owned by voyage VGzxnR97R.
- [SCOPE-06] Sticky `recovery_exhausted` state and `port machine unfence` reset — owned by voyage VGzxoN8WF.
- [SCOPE-13] Alerting, dashboards, UI.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port.toml` grows a `[clusters.<name>.recovery]` block with `enabled: bool` (default `false`), `settle_seconds: u64`, and the tier-1 attempt-counter fields (`recovery_attempts.tier_1`, `last_recovery_action`, `recovery_state`) exposed through the existing machine status contract. | SCOPE-01 | FR-01 | unit |
| SRS-02 | When `recovery.enabled = true` on a cluster and the detector has set `wedged_since` with `wedge_class = "guest"` on a machine, the node-agent responsible for that machine executes `port machine stop` followed by `port machine launch` against the same runtime root, then re-evaluates the wedge state after `settle_seconds`. | SCOPE-02 | FR-01 | integration |
| SRS-03 | A successful tier-1 attempt increments `recovery_attempts.tier_1` and stamps `last_recovery_action = { tier: 1, timestamp_unix_s, outcome }`; on convergence (guest heartbeat resumes and `wedged_since` clears), `recovery_state` returns to `"ok"`. | SCOPE-02 | FR-01 | integration |
| SRS-04 | Each tier-1 attempt emits a structured event with fields `machine`, `tier`, `timestamp_unix_s`, `outcome` ∈ `{started, succeeded, failed, skipped_busy}` to the configured event sink. | SCOPE-07 | FR-01 | unit |
| SRS-05 | If a human lifecycle operation (`port machine stop/launch/up`) holds the per-machine lifecycle lock when tier-1 would fire, recovery emits a `skipped_busy` event, does not increment the counter, and re-evaluates on the next settling interval. | SCOPE-08 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Recovery defaults to `enabled = false` for every cluster; a cluster with no `[recovery]` block behaves exactly as one with `enabled = false` — no action, no state change. | SCOPE-01 | NFR-01 | unit |
| SRS-NFR-02 | Tier-1 must not take action when the detector has cleared `wedged_since` between the trigger and the action (race guard); a unit test proves the recovery path re-reads the wedge state immediately before executing. | SCOPE-02 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
