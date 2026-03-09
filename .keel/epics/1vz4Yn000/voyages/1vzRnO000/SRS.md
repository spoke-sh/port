# Execute Hosted Services And Sandboxes - Software Requirements Specification

> Run hosted service and sandbox commands through the live control plane and node agent instead of only storing desired state

**Epic:** [1vz4Yn000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:

- [SCOPE-01] Execute hosted `port service apply|list|status|stop` through the
  live control-plane and node-agent path instead of only storing desired state.
- [SCOPE-02] Keep `--kind sandbox` on the same canonical `service` surface and
  make sandbox execution observable through the same status model.
- [SCOPE-03] Materialize stored machine secrets into launched guest processes
  without echoing secret values back through CLI, SDK, or runtime status.
- [SCOPE-04] Persist node-owned hosted service runtime state, including live
  state, exit status, and log-path metadata, under the selected runtime owner.
- [SCOPE-05] Publish help, docs, and proof that show what hosted service and
  sandbox execution now ships versus which higher-level behaviors remain
  follow-on work.

Out of scope:

- [SCOPE-06] Scheduler or host-group policy changes beyond the current placement model.
- [SCOPE-07] Secret-backend hardening, encryption, or external secret stores.
- [SCOPE-08] Restart policies, health checks, auto-restarts, or multi-instance services.
- [SCOPE-09] Dedicated `service logs` or `service exec` verbs.
- [SCOPE-10] Quota, billing, RBAC, or broader multi-tenant auth changes.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The hosted control plane and node agent remain the routing and runtime owners for hosted service actions. | dependency | Hosted service execution would need a different product boundary. |
| The guest agent can grow a managed-process contract without breaking existing guest `exec|copy|pty|logs|forward` flows. | assumption | Service execution would require an incompatible second guest-control channel. |
| Stored machine secrets remain the bootstrap source of service env injection for this slice. | assumption | Secret materialization would need a broader design before execution can ship. |

## Constraints

- Keep one canonical `port service` CLI and hosted API surface; do not add
  hosted-only aliases.
- Preserve the existing machine, guest, and service ownership model: CLI/SDK ->
  control plane -> node agent -> guest agent.
- Treat secret values as write-only at the operator surface; status and logs may
  refer to secret names and env bindings but not their raw values.
- Make automated proofs reliable under `keel verify` and `keel story record`,
  including nested `nix develop` execution.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a managed guest-process execution contract for services and sandboxes that preserves the canonical `service` CLI/API surface and routes through the existing hosted control-plane and node-agent path. | SCOPE-01, SCOPE-02 | FR-05 | rust tests + CLI proof |
| SRS-02 | The hosted runtime must materialize stored machine secrets into launched guest processes and persist node-owned runtime state, including live state, exit metadata, and operator-visible log paths, without surfacing raw secret values. | SCOPE-03, SCOPE-04 | FR-05 | rust tests + runtime inspection |
| SRS-03 | Hosted `port service apply`, `list`, `status`, and `stop` must execute real service and sandbox lifecycles through the canonical CLI, model, SDK, and live hosted control path instead of only mutating stored desired state. | SCOPE-01, SCOPE-02, SCOPE-04 | FR-05 | rust tests + hosted CLI demo |
| SRS-04 | Help text, README, hosted docs, and SDK docs must publish the hosted service and sandbox execution workflow and make remaining limits explicit. | SCOPE-05 | NFR-02 | help/doc proof + demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Port must keep one canonical `service` vocabulary across local and hosted lanes; hosted execution work may not introduce a second runtime model or hosted-only verb family. | SCOPE-01, SCOPE-02 | NFR-01 | design review + CLI inspection |
| SRS-NFR-02 | Hosted service execution proofs must be deterministic under `keel verify`, including nested Nix-shell execution and Unix-socket path limits. | SCOPE-04, SCOPE-05 | NFR-02 | automated script proof |
| SRS-NFR-03 | Operator-facing output must distinguish shipped hosted execution from still-planned work such as restart policy, health checks, hardened secret backends, and scheduler policy. | SCOPE-05 | NFR-02 | doc/help proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Managed Service Execution Contract | Shared service runtime contract, state model, and proof boundary are explicit before implementation. | SRS-01, SRS-02, SRS-NFR-01 |
| Implement Guest-Agent Managed Process Supervisor | The guest agent can start, inspect, and stop managed service/sandbox processes with log capture and secret env injection. | SRS-01, SRS-02, SRS-NFR-02 |
| Route Hosted Service Lifecycle Through Live Runtime | Canonical hosted `port service apply|list|status|stop` reaches real execution and live state through control plane and node agent. | SRS-02, SRS-03, SRS-NFR-01 |
| Publish Hosted Service And Sandbox Workflow | Help text, docs, SDK docs, and CLI proof show the shipped workflow and remaining limits clearly. | SRS-04, SRS-NFR-02, SRS-NFR-03 |
