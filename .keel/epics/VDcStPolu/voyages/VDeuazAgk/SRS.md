# SSH-First Hybrid Execution Foundations - SRS

> Define and deliver the first SSH-first remote Linux lane with explicit
> ownership, remote readiness, lifecycle routing, and operator proof while
> preserving local and hosted execution semantics.

**Epic:** [VDcStPolu](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Extend the Port execution model with an explicit SSH-managed host
  connection and corresponding route or ownership semantics alongside the
  existing local and hosted lanes.
- [SCOPE-02] Surface hybrid execution readiness, auth, and bootstrap guidance
  through `port doctor`, help text, and CLI-visible route context.
- [SCOPE-03] Deliver the first SSH-first remote Linux machine lifecycle slice
  through canonical `machine launch`, `status`, and `stop` verbs.
- [SCOPE-04] Publish the hybrid execution contract and the first SSH-first
  operator workflow in docs and proof artifacts.

### Out of Scope

- [SCOPE-90] Guest-operation parity beyond lifecycle (`guest exec`, `copy`,
  `pty`, `logs`, `forward`) for the SSH lane.
- [SCOPE-91] Hosted scheduler expansion, multi-node fleet management, or
  broader placement policy.
- [SCOPE-92] Cloud-provider credential automation, image realization systems,
  or provider-specific network provisioning.
- [SCOPE-93] New remote-only commands or compatibility shims that preserve
  multiple competing route vocabularies.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The existing local and hosted command model remains the canonical surface for introducing SSH-first remote execution. | assumption | The voyage would need a broader CLI redesign instead of a bounded route extension. |
| One remote Linux host reachable over SSH is enough to prove the first lifecycle slice before fleet or scheduler work lands. | assumption | The voyage would expand into a larger placement or inventory effort. |
| Route and ownership context currently exposed through `port-model`, `port-cli`, and docs can be extended cleanly with an SSH lane. | dependency | The voyage may need broader cross-cutting refactors than planned. |
| Human-reviewable proof should use the repo's recording-capable proof system rather than ad hoc screenshots. | dependency | The voyage would need a different proof strategy and mission review surface. |

## Constraints

- Preserve one canonical `port` CLI vocabulary for machine lifecycle across
  local, hosted, and SSH lanes.
- Fail fast when a machine or host cannot satisfy the requested lane; do not
  fall back silently to local runtime ownership.
- Keep local Linux and hosted control-plane workflows valid while adding the
  SSH lane.
- Use repo-local verification techniques aligned to config detection and
  recommendation: Rust tests, command proofs, and at least one recording path.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must add an explicit SSH-managed host connection contract and expose corresponding hybrid route or ownership fields without replacing the current `local` and `hosted-control-plane` paths. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-02 | `port doctor` and related help surfaces must distinguish SSH remote-host readiness, auth material, and bootstrap requirements from local-host and hosted-control-plane prerequisites. | SCOPE-02 | FR-03 | automated test + CLI proof |
| SRS-03 | Port must route canonical `machine launch`, `status`, and `stop` against an SSH-managed remote Linux host through one bounded lifecycle path. | SCOPE-03 | FR-02 | automated test + command proof |
| SRS-04 | CLI output and failure paths for SSH-targeted machines must keep machine, host, provider, route, and ownership context explicit. | SCOPE-02, SCOPE-03 | FR-04 | automated test + command proof |
| SRS-05 | The voyage must publish the first hybrid execution operator workflow, including at least one human-reviewable proof artifact recorded through the proof system. | SCOPE-04 | FR-05 | inspection + recording |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | SSH-lane failures must be explicit and actionable, including why the target cannot use SSH ownership and what prerequisite or configuration is missing. | SCOPE-02, SCOPE-03 | NFR-01 | automated test + CLI proof |
| SRS-NFR-02 | The first SSH-first slice must preserve the shipped local Linux and hosted control-plane workflows. | SCOPE-01, SCOPE-03 | NFR-02 | automated regression test |
| SRS-NFR-03 | Verification for this voyage must use repo-local techniques recommended by Keel for this repository: Rust tests, command proofs, and a recording-backed human proof path. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | board review + command proof + recording |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VDeuzcDcL](../../../../stories/VDeuzcDcL/README.md) Introduce SSH Hybrid Route Contract | SRS-01, SRS-NFR-02 |
| [VDeuzX5cO](../../../../stories/VDeuzX5cO/README.md) Add SSH Remote Doctor Guidance | SRS-02, SRS-NFR-01 |
| [VDeuzYscv](../../../../stories/VDeuzYscv/README.md) Implement SSH Machine Lifecycle Routing | SRS-03, SRS-04 |
| [VDeuzbve3](../../../../stories/VDeuzbve3/README.md) Publish Hybrid Execution Operator Proof | SRS-05, SRS-NFR-03 |
