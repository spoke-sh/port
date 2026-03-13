# Canonical App Hosting Screen Surface - Product Requirements

## Problem Statement

Port can already run hosted services and expose guest traffic, but it still
lacks one canonical operator proof surface that launches a real HTTP app,
curls it from the host, and records human-reviewable evidence.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Give maintainers one canonical answer to "can Port host an app?" | One repo-level proof path launches a minimal hosted HTTP app, curls it successfully from the host, and records a reviewable artifact | First voyage complete |
| GOAL-02 | Reuse the existing hosted service and guest-forward operator model instead of inventing a demo-only lane | The proof path runs through canonical `port service` and `port guest forward` surfaces | First voyage complete |
| GOAL-03 | Keep the proof surface narrow and ready for future naming and recorder upgrades without blocking the first shipped result | The first slice ships on the current repo-level proof surface with current recording tooling and documents future `screen` and `atxt` work as follow-on | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Maintainer / Operator | A repo maintainer or evaluator deciding whether Port can host real applications yet | One fast, reviewable proof that exercises real hosted app lifecycle and exposure surfaces end to end |
| Contributor | A developer extending Port's hosted experience | One canonical proof contract and boundary statement to evolve without creating parallel demo surfaces |

## Scope

### In Scope

- [SCOPE-01] One narrow hosted HTTP application proof workflow using canonical
  `port service` plus `port guest forward` surfaces.
- [SCOPE-02] One repo-level proof surface that highlights this workflow and its
  human-reviewable artifact.
- [SCOPE-03] Operator-facing documentation and boundaries for the canonical
  proof path.

### Out of Scope

- [SCOPE-90] Renaming the live repo-level proof surface from `mission` to
  `screen` before upstream `keel screen` exists.
- [SCOPE-91] Migrating the recording path to `atxt` before the tool is ready in
  this repository environment.
- [SCOPE-92] General app platform claims such as ingress, autoscaling, public
  networking, or multi-service orchestration.
- [SCOPE-93] Hosted control-plane hardening, tenancy, or external publishing
  infrastructure beyond the repo-local proof lane.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must provide one canonical hosted HTTP app proof workflow that launches a minimal service through the existing hosted control-plane and node-agent path, exposes it through the canonical Port transport, and proves success with a host-side `curl`. | GOAL-01, GOAL-02 | must | This is the shortest honest proof that Port can host an app rather than only machines or desired state. |
| FR-02 | Port must surface that workflow through one repo-level proof entrypoint that keeps command path, artifact path, and mission evidence legible from one place. | GOAL-01, GOAL-03 | must | Maintainers need one obvious operator proof surface instead of reading multiple docs and scripts by hand. |
| FR-03 | Port must document the canonical app-hosting proof contract, prerequisites, and current boundaries. | GOAL-02, GOAL-03 | must | Without explicit boundaries, the proof surface would imply broader hosted app support than the product currently ships. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Verification for this epic must use repository-local techniques already recommended for this project: Rust tests, command proofs, and a recording-backed human-reviewable artifact. | GOAL-01, GOAL-03 | must | The proof surface only matters if it is repeatable and reviewable in the current repo environment. |
| NFR-02 | The first app-hosting proof slice must preserve the existing hosted service, hosted guest-forward, and repo-level mission proof surfaces without silent behavioral regressions. | GOAL-02, GOAL-03 | must | This epic should harden and focus the product surface, not destabilize already-shipped hosted lanes. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Hosted app lifecycle | Rust tests + command proof | Story-level logs proving hosted service apply, exposure, and curl success |
| Repo-level proof surface | Command proof + inspection | `just mission` output until `screen` lands, plus mission-linked artifact gallery |
| Human review artifact | Recording-backed proof | GIF, cast, or equivalent artifact linked through the proof system |
| Boundaries and docs | Search proof + inspection | Updated README/docs plus story evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing hosted `service apply` and hosted `guest forward` paths are strong enough to anchor the first app-hosting proof without new substrate work. | The epic would collapse back into infrastructure work instead of shipping a canonical proof path. | Validate in the first voyage through tests and command proofs. |
| The current repository proof path can continue using recording-backed artifacts even while direct `vhs` capture remains environment-sensitive. | The epic would block on recorder tooling instead of shipping the first operator proof. | Keep artifact generation renderer-backed and leave `atxt` to the scheduled routine. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the repo-level proof surface continue to be named `mission` until upstream `keel screen` exists? | Epic owner | Resolved for first slice: yes |
| Could the hosted proof imply broader production app-hosting guarantees than Port currently supports? | Epic owner | Mitigated through explicit boundary docs |
| Could recorder/tooling instability distract from the first proof slice? | Epic owner | Mitigated by keeping current recorder path and tracking `atxt` separately via routine |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A maintainer can run one repo-level proof path and review a real hosted HTTP app workflow from service launch through successful host-side curl.
- [ ] The proof surface reuses canonical Port service and guest-forward surfaces instead of a bespoke demo-only lane.
- [ ] The first slice ships with explicit docs and boundaries, while future `screen` and `atxt` upgrades stay clearly separated as follow-on work.
<!-- END SUCCESS_CRITERIA -->
