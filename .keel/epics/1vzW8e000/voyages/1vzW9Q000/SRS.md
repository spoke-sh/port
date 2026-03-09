# Hosted Artifact Push And Pull - Software Requirements Specification

> Implement the first live hosted-api artifact backend so canonical Port push and pull flows work end-to-end through the hosted control plane.

**Epic:** [1vzW8e000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:

- [SCOPE-01] Ship a live `hosted-api` artifact backend through the existing
  hosted control-plane auth and transport path.
- [SCOPE-02] Support canonical hosted push and pull flows for selected kernel
  and guest-image variants, including deterministic cache and store paths plus
  explicit transfer metadata.
- [SCOPE-03] Publish CLI help, docs, and executable proof for local build plus
  hosted publish and fetch workflows.

Out of scope:

- [SCOPE-04] OCI registry transport or registry-auth integration.
- [SCOPE-05] Artifact deduplication, garbage collection, or quota management
  beyond deterministic overwrite semantics.
- [SCOPE-06] Content-addressed CAS redesign or external package-manager
  integration.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The hosted bearer-token contract in the sample config remains the canonical auth path for the first artifact slice. | dependency | Hosted artifact transport would need auth redesign before it can ship safely. |
| `ArtifactStore::HostedApi { endpoint }` remains the canonical model token for hosted distribution. | assumption | The voyage would have to rework model and docs before implementation. |
| Control-plane-owned filesystem storage under `.port/hosted/<control-plane>/...` is acceptable for the first executable backend. | assumption | The voyage would need a dedicated storage service instead of a control-plane-owned store. |
| Existing local build and cache-path behavior remain the compatibility baseline. | dependency | Hosted mobility could regress the shipped file-backed backend or selector model. |

## Constraints

- No silent fallback from `hosted-api` to `file-system`.
- CLI surface stays `artifacts build|validate|push|pull`; no hosted-only artifact command family.
- The first slice must remain selector-aware across `architecture`, `substrate`,
  and `protection_mode`.
- Verification should use repo-local control-plane proof plus automated Rust
  tests; OCI and external registries are out of scope.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define a hosted artifact backend contract and resolve it deterministically from the existing artifact model, including explicit hosted endpoint, selector, cache path, and hosted store path detail. | SCOPE-01 | FR-01 | automated test |
| SRS-02 | The hosted control plane must expose authenticated artifact push and pull routes that stream one selected variant into and out of a control-plane-owned store without changing the artifact reference vocabulary. | SCOPE-01, SCOPE-02 | FR-02 | automated test + CLI proof |
| SRS-03 | `port artifacts push` and `port artifacts pull` must route to the configured hosted backend and print backend, selector, cache path, and hosted store path information through the canonical CLI output. | SCOPE-02 | FR-03 | automated test + command proof |
| SRS-04 | Repo-local proof must show a selected artifact variant built locally, pushed to the hosted backend, deleted from the local output path, then pulled back successfully through the same CLI. | SCOPE-02, SCOPE-03 | FR-04 | command proof |
| SRS-05 | CLI help, README, and artifact docs must publish the hosted artifact workflow and explicitly state that OCI remains follow-on work. | SCOPE-03 | FR-05 | doc/help proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted artifact paths and selector rendering must remain deterministic across model tests, runtime transfer metadata, and CLI output. | SCOPE-01, SCOPE-02 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | Hosted artifact failures must include artifact reference, selector, backend, and control-plane store-path or endpoint detail. | SCOPE-01, SCOPE-02 | NFR-02 | automated test |
| SRS-NFR-03 | The voyage must close with executable proof and board evidence for the shipped hosted backend instead of leaving `hosted-api` as a modeled-only placeholder. | SCOPE-03 | NFR-03 | board evidence + command proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Hosted Artifact Backend Contract | Shared model, runtime, and hosted protocol deterministically resolve `hosted-api` artifact metadata and failure context. | SRS-01, SRS-NFR-01, SRS-NFR-02 |
| Implement Hosted Artifact Control Plane Routes | Hosted control-plane upload and download routes persist selected artifact variants under the deterministic store path. | SRS-02, SRS-NFR-01, SRS-NFR-02 |
| Route Artifact Push And Pull Through Hosted Backend | Canonical `port artifacts push|pull` uses the hosted backend and prints backend plus path detail. | SRS-03, SRS-NFR-01, SRS-NFR-02 |
| Publish Hosted Artifact Mobility Workflow | Repo-local proof and docs publish the hosted workflow and explicit OCI boundary. | SRS-04, SRS-05, SRS-NFR-03 |
