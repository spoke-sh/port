# Hosted External Project Deployment Proof - SRS

> Prove Port can stage and run one real external project through shipped hosted
> primitives without claiming an app-bundle contract yet.

**Epic:** [VEyjUL2Zr](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Stage one external static-site project snapshot into repo-local
  hosted compute through hosted `port guest copy` and any minimal setup step
  needed to make the project runnable.
- [SCOPE-02] Start that staged project through canonical hosted
  `port service apply`, expose it through `port guest forward`, and prove
  success with a host-side `curl`.
- [SCOPE-03] Surface the workflow and artifact through the current repo-level
  proof entrypoint and publish operator-facing contract and boundary guidance
  for the first external-project deployment slice.

### Out of Scope

- [SCOPE-90] App bundle artifact contracts, image formats, or deployment
  packaging standards.
- [SCOPE-91] App bundle service runtimes, container-like lifecycle semantics,
  or projects that need more than the current BusyBox-compatible guest
  environment.
- [SCOPE-92] Ingress, public networking, multi-service orchestration,
  autoscaling, tenancy, or broader production-hosting claims.
- [SCOPE-93] Renaming the shipped repo-level proof entrypoint to `screen` or
  migrating the recorder path to `atxt`.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hosted `port guest copy` already streams bytes through the live control-plane and node-agent path. | dependency | The voyage could not honestly prove staging a project from outside the repo into hosted compute. |
| Hosted `port service apply|list|status|stop` already executes real service lifecycle through the live hosted route. | dependency | The voyage would fall back into runtime delivery instead of external-project deployment proof work. |
| Hosted `port guest forward` already exposes node-owned listeners through the live control-plane and node-agent path. | dependency | The voyage could not produce an honest host-side curl proof. |
| The current repository proof system can continue to publish human-reviewable artifacts through renderer-backed GIF/cast outputs. | dependency | The voyage would block on recorder tooling rather than shipping the first deployment proof path. |
| One external static-site project snapshot is enough to prove the first external deployment outcome. | assumption | The voyage would over-expand into app-bundle or runtime-platform work too early. |

## Constraints

- Keep one canonical operator vocabulary: use `port guest copy`, optional
  `port guest exec` for setup, `port service`, `port guest forward`, and the
  current repo-level mission proof surface.
- Keep the first slice narrow: one external static-site project snapshot, one
  staging path, one service process, and one successful host-side `curl`.
- Keep the sample project repo-local and reproducible; do not depend on live
  network fetches or mutable upstream HEAD state during proof verification.
- Treat app-bundle contracts and app-bundle runtimes as explicit follow-on
  missions, not blockers for this voyage.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must provide one canonical workflow that starts a repo-local hosted control plane and node agent, stages one external static-site project snapshot into hosted compute through `port guest copy` plus any minimal setup step, and keeps machine, host-group, and hosted-route context explicit. | SCOPE-01 | FR-01 | command proof + automated regression test |
| SRS-02 | Port must start that staged project through canonical `port service apply`, expose it through `port guest forward`, and prove success with a host-side `curl` that returns a payload sourced from the staged project bytes. | SCOPE-02 | FR-01 | command proof + automated regression test |
| SRS-03 | The current repo-level proof entrypoint must surface the canonical external-project deployment workflow, including the runnable proof path and the recorded artifact, as the primary operator-facing evidence for this slice. | SCOPE-03 | FR-02 | command proof + inspection |
| SRS-04 | Port must publish the external-project deployment contract and boundaries, including that this slice uses shipped hosted primitives today while app-bundle artifact and runtime work remains deferred. | SCOPE-03 | FR-03 | inspection + search proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Verification for this voyage must use repo-local techniques already recommended by Keel for this repository: Rust tests, command proofs, and a recording-backed human-reviewable proof artifact. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | board review + command proof + recording |
| SRS-NFR-02 | The voyage must preserve existing hosted `guest copy`, hosted service, hosted guest-forward, and repo-level mission proof behavior outside the newly added canonical external-project proof path. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated regression test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VEyjdNRno](../../../../stories/VEyjdNRno/README.md) Implement Hosted External Project Deployment Workflow | SRS-01, SRS-02, SRS-NFR-02 |
| [VEyjdN0nf](../../../../stories/VEyjdN0nf/README.md) Wire Repo-Level Mission Surface To External Project Deployment Proof | SRS-03, SRS-NFR-01 |
| [VEyjdJhne](../../../../stories/VEyjdJhne/README.md) Publish External Project Deployment Contract And Boundaries | SRS-04 |
