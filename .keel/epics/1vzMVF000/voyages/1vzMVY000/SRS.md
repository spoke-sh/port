# Streamed Guest Control Transport - Software Requirements Specification

> Deliver streamed guest shell and logs workflows plus real hosted copy and forward transport through the canonical Port surfaces.

**Epic:** [1vzMVF000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- streamed PTY and `logs --follow` behavior over the shared guest protocol
- hosted control-plane and node-agent transport for copy and forward
- CLI, SDK, and docs updates for the streamed guest workflow

### Out of Scope

- scheduler or host-group policy changes
- service execution or teardown beyond guest transport
- Cloud Hypervisor delivery or other new substrate programs

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Existing guest protocol framing can add stream lifecycle messages without breaking local Linux, hosted PVM, or AVF request paths | assumption | The voyage would need a protocol fork and broader migration plan |
| Hosted control-plane and node-agent servers can expose or relay streamed guest transport using the current auth and route model | dependency | Hosted transport would need a new daemon or routing layer |
| CLI raw-terminal behavior can be introduced incrementally while preserving non-interactive execution paths | assumption | `guest pty` might need a larger CLI/UI redesign |

## Constraints

- The canonical operator vocabulary remains `port guest exec|copy|pty|logs|forward`.
- Hosted auth, control-plane identity, and node-agent token boundaries must stay
  explicit.
- No silent fallback is allowed from hosted streaming to repo-local or
  node-path-only behavior.
- Existing local Firecracker, hosted PVM, and AVF proofs must stay green.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The shared guest protocol must define streamed PTY and log-follow session semantics, including attach, payload, EOF, exit, and failure framing, without replacing the existing guest command vocabulary. | SCOPE-01 | FR-01 | automated test + protocol proof |
| SRS-02 | `port guest pty` and `port guest logs --follow` must provide streamed behavior through the canonical CLI and SDK for local and AVF-backed runtimes using the shared guest protocol. | SCOPE-01, SCOPE-03 | FR-01 | automated test + CLI proof |
| SRS-03 | Hosted `port guest copy` must transfer bytes through the hosted control-plane and node-agent path without assuming the referenced host paths are directly visible on the node host. | SCOPE-02 | FR-02 | automated test + hosted demo |
| SRS-04 | Hosted `port guest forward` must use real hosted transport ownership instead of the current repo-local listener lifecycle while keeping the same canonical command family. | SCOPE-02 | FR-02 | automated test + hosted demo |
| SRS-05 | CLI help, README, `docs/hosted.md`, and `docs/sdk.md` must publish the streamed guest-session and hosted-transport workflow plus its explicit boundaries. | SCOPE-03 | FR-03 | command proof + inspection |
| SRS-06 | The streamed guest transport rollout must preserve the existing Firecracker standard, hosted PVM, and AVF workflows while it lands. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-03 | automated test + CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Streamed guest sessions must terminate deterministically with explicit EOF, exit, and error semantics visible to the CLI and SDK. | SCOPE-01, SCOPE-02 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | Hosted transport must keep explicit auth, route, and ownership detail in success and failure paths. | SCOPE-02, SCOPE-03 | NFR-01 | automated test + inspection |
| SRS-NFR-03 | No local, hosted, or substrate lane may silently fall back to older request/response or repo-local transport behavior once the new streamed path is selected. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
