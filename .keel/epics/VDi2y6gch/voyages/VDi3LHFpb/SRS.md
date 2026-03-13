# Hosted HTTP App Curl Proof - SRS

> Make the repo-level proof surface host one minimal HTTP app through Port,
> curl it from the host, and record a human-reviewable artifact.

**Epic:** [VDi2y6gch](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Launch one minimal hosted HTTP application through the canonical
  hosted service path in a repo-local hosted control-plane and node-agent
  environment.
- [SCOPE-02] Expose that hosted app through the canonical Port guest-forward
  surface and prove success with a host-side `curl`.
- [SCOPE-03] Surface the workflow and artifact through the current repo-level
  proof entrypoint so maintainers can review it from one place, and publish
  operator-facing proof contract and boundary guidance for the first
  app-hosting slice.

### Out of Scope

- [SCOPE-90] Renaming the shipped repo-level proof entrypoint to `screen`
  before upstream `keel screen` exists.
- [SCOPE-91] Migrating the proof recorder to `atxt`.
- [SCOPE-92] Multi-service orchestration, ingress, public exposure,
  autoscaling, or broader production-hosting claims.
- [SCOPE-93] Hosted tenancy, auth hardening, or external publishing systems.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted `port service apply|list|status|stop` already executes real service lifecycle through the live hosted route. | dependency | The voyage would fall back into infrastructure delivery instead of app-hosting proof work. |
| Hosted `port guest forward` already exposes node-owned listeners through the live control-plane and node-agent path. | dependency | The voyage could not produce an honest host-side curl proof. |
| The current repository proof system can continue to publish human-reviewable artifacts through renderer-backed GIF/cast outputs. | dependency | The voyage would block on recorder tooling rather than shipping the first proof path. |
| One minimal HTTP service is enough to prove the first app-hosting outcome. | assumption | The voyage would over-expand into broader service-platform work too early. |

## Constraints

- Keep one canonical operator vocabulary: use `port service`, `port guest
  forward`, and the current repo-level proof surface.
- Keep the first slice narrow: one minimal hosted HTTP application and one
  successful host-side curl.
- Use repository-local verification techniques aligned to Keel recommendations:
  Rust tests, command proofs, and a recording-backed artifact.
- Treat `screen` naming and `atxt` recorder adoption as follow-on work, not
  blockers for this voyage.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must provide one canonical workflow that starts a repo-local hosted control plane and node agent, applies one minimal hosted HTTP service through `port service apply`, and keeps machine, host-group, and hosted-route context explicit. | SCOPE-01 | FR-01 | command proof + automated regression test |
| SRS-02 | Port must expose that hosted HTTP service through the canonical `port guest forward` surface and prove success with a host-side `curl` that returns the expected application payload. | SCOPE-02 | FR-01 | command proof + automated regression test |
| SRS-03 | The current repo-level proof entrypoint must surface the canonical hosted app proof workflow, including the runnable proof path and the recorded artifact, as the primary operator-facing evidence for this slice. | SCOPE-03 | FR-02 | command proof + inspection |
| SRS-04 | Port must publish the app-hosting proof contract and boundaries, including the current `mission` name, the planned future `screen` cutover, the current recorder path, and the deferred `atxt` migration. | SCOPE-03 | FR-03 | inspection + search proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Verification for this voyage must use repo-local techniques already recommended by Keel for this repository: Rust tests, command proofs, and a recording-backed human-reviewable proof artifact. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | board review + command proof + recording |
| SRS-NFR-02 | The voyage must preserve existing hosted service, hosted guest-forward, and repo-level mission proof behavior outside the newly added canonical app-hosting proof path. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated regression test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VDi3O7KjN](../../../../stories/VDi3O7KjN/README.md) Implement Hosted HTTP App Proof Workflow | SRS-01, SRS-02, SRS-NFR-02 |
| [VDi3O5dlc](../../../../stories/VDi3O5dlc/README.md) Wire Repo-Level Screen Surface To App Hosting Proof | SRS-03, SRS-NFR-01 |
| [VDi3O6vld](../../../../stories/VDi3O6vld/README.md) Publish App Hosting Proof Contract And Boundaries | SRS-04, SRS-NFR-01 |
