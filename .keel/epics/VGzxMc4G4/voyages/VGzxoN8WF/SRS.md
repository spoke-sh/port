# Tier-3 Signal Persistence, Unfence Reset, And End-To-End Proof - SRS

## Summary

Epic: VGzxMc4G4
Goal: Deliver the durability and operator-reset half of the recovery ladder — persist `awaiting_tier_3_host_recycle` across control-plane restarts, land `port machine unfence` as the manual clear path, auto-clear on a successful operator-driven launch, and cover the full ladder with deterministic end-to-end tests.

## Scope

### In Scope

- [SCOPE-06] Persist `recovery_state` and `recovery_attempts` on disk so a control-plane restart mid-escalation does not re-arm the ladder against a machine that is already awaiting consumer handoff.
- [SCOPE-06] `port machine unfence --machine X` CLI command that resets any non-`ok` recovery state (`awaiting_tier_3_host_recycle`, `in_progress`) and `recovery_attempts.*` back to zero, emitting a `recovery_unfenced` event. The command makes no runtime changes — it is explicitly not an alias for `launch`.
- [SCOPE-06] Auto-clear on a successful operator-driven `port machine launch`: if the machine was in `awaiting_tier_3_host_recycle` and the launch produces a Live guest-agent heartbeat within a documented window, Port transitions `recovery_state` back to `ok` and emits `recovery_unfenced_via_launch`. Unsuccessful launches do not clear the state.
- [SCOPE-08] End-to-end integration tests covering the full ladder: (a) simulated guest-side wedge converges under tier-1 alone; (b) simulated node-side wedge escalates to `awaiting_tier_3_host_recycle` with the `tier_3_escalation` event emitted, and returns to `ok` once simulated host return (re-register + fresh guest heartbeat) is observed; (c) `port machine unfence` on a persisted escalation cleanly resets to `ok`.

### Out of Scope

- [SCOPE-11] Cross-cell rebalancing after escalation.
- [SCOPE-13] Alerting, dashboards, UI for recovery events.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `recovery_state` and `recovery_attempts` are persisted under the runtime-root (e.g. `runtime/recovery/<machine>.json`) alongside existing registered-node state; a control-plane restart with a machine in `awaiting_tier_3_host_recycle` reloads the state unchanged and does not re-arm tier-1/2. | SCOPE-06 | FR-01 | integration |
| SRS-02 | `port machine unfence --machine X` is a new CLI command routed through a new control-plane endpoint `POST /v1/machines/{machine}/recovery:unfence`. It clears any non-`ok` `recovery_state`, zeros `recovery_attempts.*`, emits a `recovery_unfenced` event, and makes no other runtime change. On a machine with `recovery_state = "ok"` it is a no-op that returns success. | SCOPE-06 | FR-01 | integration |
| SRS-03 | A successful operator-driven `port machine launch` on a machine in `awaiting_tier_3_host_recycle` auto-clears the state when a Live guest-agent heartbeat arrives within a documented convergence window; the transition emits `recovery_unfenced_via_launch`. A launch that does not produce a heartbeat within the window leaves the state unchanged. | SCOPE-06 | FR-01 | integration |
| SRS-04 | An end-to-end test simulates a guest-side wedge on a local Firecracker machine and asserts tier-1 converges it without operator intervention; captures the runner event stream via a channel-based hook and verifies every transition. | SCOPE-08 | FR-01 | integration |
| SRS-05 | An end-to-end test simulates a node-side wedge, drives the ladder until `recovery_state = "awaiting_tier_3_host_recycle"` and a `tier_3_escalation` event is captured; then simulates host return (node-agent re-registration + fresh guest heartbeat) and asserts auto-clear back to `ok` with a `tier_3_host_returned` event. The test uses no fake cloud clients — observation only. | SCOPE-08 | FR-01 | integration |
| SRS-06 | An end-to-end test drives the ladder into `awaiting_tier_3_host_recycle`, crashes and restarts the control plane, asserts the state reloads, invokes `port machine unfence` via the CLI, and confirms the ladder re-arms and a subsequent wedge fires tier-1 again. | SCOPE-08 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | End-to-end tests use injectable clocks and channel-based event hooks rather than wall-clock `sleep` for convergence, so CI is stable. A grep-based guard in the test file rejects `thread::sleep` / `tokio::time::sleep` outside explicit `#[allow]` annotations. | SCOPE-08 | NFR-01 | unit |
| SRS-NFR-02 | The unfence command does not interact with any cloud-provider API or remote shell. Unfence is a local state mutation with a local event emission — no network calls beyond the existing control-plane HTTP path. | SCOPE-06 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
