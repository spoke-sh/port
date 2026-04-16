# Recovery Exhaustion Reset And End-To-End Proof - SRS

## Summary

Epic: VGzxMc4G4
Goal: Deliver the sticky recovery_exhausted terminal state, the port machine unfence reset path, and auto-clear on a successful operator-driven launch that produces a Live guest-agent heartbeat. Cover the end-to-end ladder with an integration test that converges a simulated wedge under tier-1 and another under tier-3.

## Scope

### In Scope

- [SCOPE-06] Introduce `recovery_state = "exhausted"` as a terminal, sticky state: once set, it survives `window_seconds` rollovers and the ladder stops attempting further actions on the affected machine.
- [SCOPE-06] Add `port machine unfence --machine X` CLI command that clears `recovery_exhausted`, resets `recovery_attempts.*` to zero, and emits a `recovery_unfenced` event. Requires no other machine state change; explicitly not an alias for `launch`.
- [SCOPE-06] Auto-clear `recovery_exhausted` when a successful operator-driven `port machine launch` produces a Live guest-agent heartbeat — this avoids forcing a second manual step when the operator already rebooted the machine.
- [SCOPE-09] End-to-end integration tests covering the full ladder: (a) simulated guest-side wedge converges under tier-1 alone; (b) simulated node-side wedge on a single-tenant host converges under tier-3; (c) ladder exhausts without convergence, `recovery_state = "exhausted"` is set and persists across a window rollover, and `port machine unfence` resets cleanly.

### Out of Scope

- [SCOPE-10] Cross-cell rebalancing after exhaustion.
- [SCOPE-13] Alerting, dashboards, UI for exhaustion.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | When a machine's ladder reaches tier-3 without convergence (or tier-3 is suppressed and no further tiers remain), the runner sets `recovery_state = "exhausted"`, stops attempting further actions, and persists the state across control-plane restarts. | SCOPE-06 | FR-01 | integration |
| SRS-02 | `recovery_state = "exhausted"` survives `window_seconds` rollovers: attempt counters decaying does not re-arm the ladder; a test rolls the window and asserts no tier-1 fires on the exhausted machine. | SCOPE-06 | FR-01 | unit |
| SRS-03 | `port machine unfence --machine X` is a new CLI command that clears `recovery_exhausted`, resets `recovery_attempts.tier_1/2/3` to zero, and emits a `recovery_unfenced` event; subsequent detector ticks re-arm the ladder normally. | SCOPE-06 | FR-01 | integration |
| SRS-04 | A successful operator-driven `port machine launch` that produces a Live guest-agent heartbeat within a documented convergence window auto-clears `recovery_exhausted`, resets counters, and emits a `recovery_unfenced_via_launch` event. Unsuccessful launches do not clear the state. | SCOPE-06 | FR-01 | integration |
| SRS-05 | An end-to-end test simulates a guest-side wedge on a local Firecracker machine and asserts tier-1 converges it without operator intervention; the test is deterministic (no `sleep`-based waits). | SCOPE-09 | FR-01 | integration |
| SRS-06 | An end-to-end test simulates a node-side wedge on a single-tenant host, asserts tier-3 fires through the fake `HostRebootClient`, and confirms `recovery_state` returns to `"ok"` on re-registration + heartbeat recovery. | SCOPE-09 | FR-01 | integration |
| SRS-07 | An end-to-end test drives the ladder through exhaustion, asserts sticky `recovery_exhausted` across a window rollover, then invokes `port machine unfence` and confirms the ladder re-arms cleanly. | SCOPE-09 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | `recovery_exhausted` persistence survives a control-plane restart (on-disk, same directory as registered-node state); a test crashes and restarts the control plane mid-exhaustion and asserts the state is still set. | SCOPE-06 | NFR-01 | integration |
| SRS-NFR-02 | End-to-end tests must not use wall-clock `sleep` for convergence; they drive time via injectable clocks or event hooks so CI is stable. | SCOPE-09 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
