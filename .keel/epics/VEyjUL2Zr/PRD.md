# Canonical External Project Deployment Workflow - Product Requirements

## Problem Statement

Port can prove one bounded hosted app path, but it still lacks a canonical operator workflow that stages and runs a real external project through current Port primitives instead of injecting files inline inside the guest.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Give maintainers one canonical answer to "can Port deploy an external project?" | One repo-level proof path stages an external static-site project snapshot into hosted compute, serves it, curls the expected payload from the host, and records a reviewable artifact | First voyage complete |
| GOAL-02 | Reuse the existing hosted guest-copy, service, and guest-forward model instead of inventing app-bundle semantics early. | The proof path runs through canonical `port guest copy`, `port service`, and `port guest forward` surfaces | First voyage complete |
| GOAL-03 | Keep the first deployment slice honest and future-facing. | The first slice ships with explicit boundaries that leave app-bundle artifact and runtime work as follow-on missions | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Maintainer / Operator | A repo maintainer or evaluator deciding whether Port can move beyond one inline demo and deploy outside project content yet. | One fast, reviewable proof that stages and runs a real external project through Port-managed compute. |
| Contributor | A developer extending Port's app-hosting experience. | One canonical deployment proof contract that uses shipped primitives today without blurring into future app-bundle scope. |

## Scope

### In Scope

- [SCOPE-01] One narrow hosted external-project proof using canonical
  `port guest copy`, optional `port guest exec`, `port service`, and
  `port guest forward`.
- [SCOPE-02] One repo-level proof surface that highlights this workflow and its
  human-reviewable artifact.
- [SCOPE-03] Operator-facing documentation and boundaries for the
  current-primitives external-project deployment slice.

### Out of Scope

- [SCOPE-90] App bundle artifact contracts or packaging formats for deployment.
- [SCOPE-91] App bundle service runtimes or container-like execution contracts.
- [SCOPE-92] Language-specific runtime bootstrap, buildpack/container
  packaging, or projects that require more than the shipped BusyBox-compatible
  guest environment.
- [SCOPE-93] Renaming the repo-level proof surface to `screen` or migrating the
  recorder path to `atxt`.
- [SCOPE-94] Ingress, public exposure, autoscaling, multi-service
  orchestration, tenancy, or broader production-hosting guarantees.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must provide one canonical external-project deployment proof workflow that stages real external-project bytes into hosted compute through shipped hosted guest operations, starts the project through `port service apply`, exposes it through the canonical Port transport, and proves success with a host-side `curl`. | GOAL-01, GOAL-02 | must | This is the smallest honest proof that Port can deploy project content from outside the repo instead of only running an inline demo payload. |
| FR-02 | Port must surface that workflow through one repo-level proof entrypoint that keeps the runnable command path, artifact path, and mission evidence legible from one place. | GOAL-01, GOAL-03 | must | Maintainers need one obvious review surface instead of reconstructing the workflow from multiple scripts and docs. |
| FR-03 | Port must document the external-project deployment contract and its current limits relative to future app-bundle work. | GOAL-02, GOAL-03 | must | Without explicit boundaries, the proof would overclaim a container or app-bundle developer experience that is not shipped yet. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Verification for this epic must use repository-local techniques already recommended for this project: Rust tests, command proofs, and a recording-backed human-reviewable artifact. | GOAL-01, GOAL-03 | must | The deployment claim only matters if the proof is repeatable and reviewable in the current repo environment. |
| NFR-02 | The first external-project deployment slice must preserve existing hosted `guest copy`, hosted service, hosted guest-forward, and repo-level mission proof behavior without silent regressions. | GOAL-02, GOAL-03 | must | This epic should extend the existing hosted surface, not destabilize it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| External-project staging and runtime | Rust tests + command proof | Story-level logs proving guest copy/setup, service launch, exposure, and curl success |
| Repo-level proof surface | Command proof + inspection | `just mission` output and mission-linked artifact gallery for the active mission |
| Human review artifact | Recording-backed proof | GIF, cast, or equivalent artifact linked through the proof system |
| Boundaries and docs | Search proof + inspection | Updated README/docs plus story evidence that distinguishes this slice from app-bundle follow-on work |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Hosted `guest copy`, hosted `service apply`, and hosted `guest forward` are already strong enough to anchor the first external-project proof without new substrate work. | The epic would collapse back into infrastructure delivery instead of proving deployment of outside project content. | Validate in the first voyage through tests and command proofs. |
| One external static-site project snapshot is enough to prove the first external deployment outcome. | The epic would expand into a broader runtime/platform mission too early. | Keep the first voyage narrow and validate the proof contract with docs and artifacts. |
| A vendored or captured external-project snapshot is acceptable for repo-local reproducibility. | The proof would depend on live network fetches or unstable upstream state. | Keep the first workflow deterministic and reviewable inside the repository. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Could this proof be mistaken for a shipped app-bundle or container contract? | Epic owner | Mitigated through explicit scope and boundary docs |
| Could the chosen sample project require runtime support beyond the shipped guest image? | Epic owner | Mitigated by choosing a BusyBox-compatible static site snapshot for the first slice |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A maintainer can run one repo-level proof path and review a real external-project deployment from staged project bytes through successful host-side curl.
- [ ] The proof surface reuses canonical hosted `guest copy`, `service`, and `guest forward` surfaces instead of inventing an app-bundle path early.
- [ ] The first slice ships with explicit docs and boundaries while app-bundle artifact and runtime work stays clearly separated as follow-on missions.
<!-- END SUCCESS_CRITERIA -->
