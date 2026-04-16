# Guest-Agent Heartbeat And Age Surface - SRS

## Summary

Epic: VGzxKV9OX
Goal: Introduce a guest-agent heartbeat probe and per-machine guest_refresh_age_seconds so the hosted control plane can tell apart a guest-side wedge (node-agent healthy, guest-agent silent) from a node-side wedge.

## Scope

### In Scope

- [SCOPE-01] Add a `Ping` request frame and matching `Pong` response to `port-agent-protocol` and implement the guest-agent `Ping` handler so a healthy guest-agent responds within a bounded budget; failure (timeout, decode, transport close) is treated as a failed probe.
- [SCOPE-02] Drive a periodic probe loop in the node-agent that stamps `guest_agent_last_heartbeat` per machine in the node-agent's in-memory state on every successful pong, independent of any in-flight guest operation.
- [SCOPE-03] Surface `guest_refresh_age_seconds: Option<u64>` on the existing hosted cluster status contract, computed monotonically per machine and flowing through the node-agent → control-plane response path the same way `refresh_age_seconds` already does.
- [SCOPE-05] Cover the new probe, timestamp update, and surfaced field with unit and integration tests; ensure tests do not regress existing guest-operation flows.

### Out of Scope

- [SCOPE-04] The wedge detector that consumes this heartbeat (voyage VGzxlScKS).
- [SCOPE-06] Any recovery action (tier-1/2/3) — owned by epic VGzxMc4G4.
- [SCOPE-07] Cross-cluster aggregation, alerting, or UI surfaces.
- [SCOPE-08] Guest-side kernel watchdog or OS-level liveness; this voyage owns only the agent-layer heartbeat.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port-agent-protocol` defines `StreamRequestFrame::Ping` and `StreamResponseFrame::Pong` (or equivalent request/response envelope pair) with round-trip serde coverage; the guest-agent responds to `Ping` with `Pong` on the existing control stream with a documented, observable response budget. | SCOPE-01 | FR-01 | unit |
| SRS-02 | The node-agent runs a periodic probe loop per registered machine that issues `Ping`, records `guest_agent_last_heartbeat` on pong, and leaves the timestamp unchanged on failure. | SCOPE-02 | FR-01 | unit |
| SRS-03 | `port cluster status --format json` and the machine status contract expose `guest_refresh_age_seconds: Option<u64>` per machine, `None` until the first successful probe, populated via a monotonic `Instant` sidecar on the node-agent and wall-clock-immune across reads. | SCOPE-03 | FR-01 | integration |
| SRS-04 | Existing guest-operation flows (`exec`, `copy`, `pty`, `logs`, `forward`) continue to pass their suites with the probe loop running, including the hosted-control-plane round-trip tests. | SCOPE-05 | NFR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The probe loop must not hold the guest-operation lock or serialize behind in-flight guest ops; a long `Exec` or `Pty` must not mask a wedged agent. | SCOPE-02 | NFR-01 | unit |
| SRS-NFR-02 | `guest_refresh_age_seconds` computation must use a monotonic clock source so wall-clock jumps on the node-agent host do not produce negative or wildly inflated ages. | SCOPE-03 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
