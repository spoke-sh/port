# Artifact Mobility And Hosted Distribution - Product Requirements

## Problem Statement

Port artifact mobility is still anchored to the file-backed demo backend. The
canonical `port artifacts push|pull` verbs exist, but operators cannot publish
or fetch selected kernel and guest-image variants through the hosted product
path that Port already uses for machines, guests, and services.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship the first live remote artifact backend through Port's hosted control plane. | A sample config using `backend = "hosted-api"` can push and pull a selected artifact variant end-to-end through the canonical CLI with recorded evidence. | First voyage complete |
| GOAL-02 | Preserve one canonical artifact vocabulary across local and hosted workflows. | Operators use the same `artifacts build|validate|push|pull` commands plus architecture/substrate/protection selectors across file-backed and hosted backends. | First voyage complete |
| GOAL-03 | Publish an operator story for hosted artifact distribution that is discoverable and honest about limits. | README, artifact docs, and CLI help explain the shipped hosted backend, cache/store ownership, auth path, and the follow-on OCI boundary. | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port operator | Builds and launches machines locally or through a hosted control plane. | Publish and fetch the exact artifact variant needed by a machine without inventing a second tool or backend-specific workflow. |
| Hosted node operator | Runs the control plane or node agent on cost-sensitive cloud VMs. | Receive the right kernel and guest-image variants through Port's existing hosted transport instead of repo-local file copies. |

## Scope

### In Scope

- [SCOPE-01] A live `hosted-api` artifact backend routed through the existing
  hosted control-plane auth and transport.
- [SCOPE-02] Canonical hosted push and pull flows for selected kernel and
  guest-image variants, including deterministic cache and store paths plus
  explicit transfer metadata.
- [SCOPE-03] CLI help, docs, and executable proofs for local build plus hosted
  publish and fetch workflows.

### Out of Scope

- [SCOPE-04] OCI registry transport or registry-auth integration.
- [SCOPE-05] Artifact deduplication, garbage collection, or quota management
  beyond deterministic overwrite semantics.
- [SCOPE-06] Content-addressed CAS redesign or external package-manager
  integration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must model and validate a live hosted artifact distribution contract for the existing `hosted-api` backend without changing the canonical artifact reference and selector vocabulary. | GOAL-01, GOAL-02 | must | The backend must become executable without fragmenting the artifact model. |
| FR-02 | Port must expose hosted control-plane routes that can publish and fetch one selected artifact variant over authenticated HTTP. | GOAL-01 | must | Hosted artifact mobility needs the same product path as the rest of the hosted control plane. |
| FR-03 | `port artifacts push` and `port artifacts pull` must route to the configured hosted backend and surface deterministic local/cache/store paths plus backend ownership in CLI output. | GOAL-01, GOAL-02 | must | Operators need one coherent CLI flow they can learn once. |
| FR-04 | Port must publish a repo-local proof that builds a variant, pushes it through the hosted backend, removes the local copy, then pulls it back successfully. | GOAL-01, GOAL-03 | must | The workflow must be executable, not prose-only. |
| FR-05 | README, `docs/artifacts.md`, and CLI help must explain the shipped hosted backend and make OCI follow-on work explicit instead of implied. | GOAL-03 | should | Discoverability and honest limits are required for a first-class CLI surface. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Variant resolution, cache paths, and hosted store paths must remain deterministic across model rendering, push, and pull. | GOAL-01, GOAL-02 | must | Artifact regressions are hard to debug if selectors resolve differently per surface. |
| NFR-02 | Hosted artifact failures must include artifact reference, selector, backend, and control-plane or node path detail. | GOAL-01, GOAL-03 | must | Operators need actionable failure context rather than generic transfer errors. |
| NFR-03 | Port must fail fast when a backend is configured but unsupported; it must not silently fall back from `hosted-api` to `file-system`. | GOAL-01, GOAL-02 | must | Hard cutover keeps the product model coherent. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Hosted artifact contract | Rust unit tests on model/runtime/backend selection | Story evidence logs linked from implementation stories |
| Hosted push/pull transport | CLI proofs against repo-local control-plane and node-agent processes | Story-level command logs and voyage report |
| Operator discoverability | Help/doc review plus scripted CLI evidence | Story verification scripts and generated board artifacts |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current hosted bearer-token contract is sufficient for the first artifact slice. | The voyage would need to pull auth redesign work forward. | Validate during control-plane route planning. |
| A control-plane-owned filesystem store is acceptable for the first live hosted backend. | The design may need a separate artifact service or object store sooner. | Keep storage ownership explicit in docs and proofs. |
| File-backed local distribution remains useful as the baseline compatibility path. | The epic may need a migration story instead of a new hosted backend slice. | Preserve file-system backend behavior and compare outputs in tests. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| When Port grows beyond the first hosted backend, should control-plane-owned storage remain the canonical store or give way to a dedicated artifact service? | Epic owner | Open |
| Large artifact streaming may expose timeout or buffering issues in the current hosted HTTP stack. | Epic owner | Open |
| OCI support is strategically important, but shipping it together with the first hosted backend risks two partial backends instead of one real one. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The sample model can be switched from `file-system` to `hosted-api` for at least one artifact and still complete `port artifacts push` and `pull` end-to-end through the hosted product path.
- [ ] Hosted artifact mobility preserves the existing artifact reference plus selector model and does not add a second artifact CLI family.
- [ ] Operator docs and help text explain what the hosted backend stores, how it authenticates, and what remains follow-on work.
<!-- END SUCCESS_CRITERIA -->
