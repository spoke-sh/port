# OCI Artifact Registry Mobility - Software Requirements Specification

> Ship the reserved oci-registry backend through canonical artifact push and pull workflows with explicit auth, path, and failure contracts.

**Epic:** [1vzW8e000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:
- [SCOPE-04] Deliver a real `oci-registry` backend for `port artifacts
  push|pull` without changing the canonical artifact reference, selector, or
  CLI vocabulary.
- [SCOPE-05] Define the registry transport contract, auth-source contract, and
  repo-local proof workflow for the first OCI slice.

Out of scope:
- [SCOPE-06] Artifact deduplication, garbage collection, or quota management
  beyond deterministic overwrite semantics.
- [SCOPE-08] Provenance signing, SBOM upload, and manifest-list assembly.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| `oras` is an acceptable first-slice registry client dependency for Port's OCI backend. | tooling | The voyage would need an in-process registry client instead of a thin runtime adapter. |
| A repo-local OCI registry process is sufficient proof for the first shipped workflow. | environment | The proof strategy would need external registry credentials or an embedded registry test service. |
| A single selected artifact variant can be represented as one OCI artifact reference derived from the canonical artifact reference plus selector. | design | The voyage would need a multi-manifest design instead of a deterministic single-reference mapping. |

## Constraints

- Hard cutover: replace the reserved OCI stub with one canonical runtime path;
  do not add compatibility fallbacks to `file-system` or `hosted-api`.
- Keep `port artifacts build|validate|push|pull` as the only artifact CLI
  family.
- Local/cache paths remain deterministic and backend-independent; only the
  remote storage path changes with the OCI backend.
- The first slice may depend on external registry tooling, but Port must expose
  that dependency through doctor and explicit error messages.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define and validate a canonical `oci-registry` artifact-store contract with deterministic remote-reference derivation, explicit auth sourcing, and explicit transport policy for selected variants. | SCOPE-04 | FR-06 | automated test + doctor proof |
| SRS-02 | `port artifacts push` must publish the selected kernel or guest-image variant through the `oci-registry` backend without changing the existing artifact reference and selector vocabulary. | SCOPE-04 | FR-07 | automated test + CLI proof |
| SRS-03 | `port artifacts pull` must fetch the selected kernel or guest-image variant from the `oci-registry` backend into the canonical cache and local artifact paths without adding a second retrieval workflow. | SCOPE-04 | FR-07 | automated test + CLI proof |
| SRS-04 | OCI transfer metadata and failures must surface the resolved remote reference, selected variant, auth source, local/cache ownership, and missing dependency detail. | SCOPE-04 | FR-08 | automated test + inspection |
| SRS-05 | README, `docs/artifacts.md`, CLI help, sample config guidance, and helper tasks must publish an executable local OCI registry workflow with recorded evidence. | SCOPE-05 | FR-10 | demo + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Local artifact paths and cache paths must remain deterministic across file-system, hosted-api, and oci-registry backends for the same artifact reference and selector. | SCOPE-04 | NFR-01 | automated test |
| SRS-NFR-02 | The repo-local OCI proof must not depend on a public registry or external internet access once the required tools are present locally. | SCOPE-05 | FR-09 | demo |
| SRS-NFR-03 | Port must fail fast when the OCI backend is configured but the required registry client or auth variables are missing. | SCOPE-04 | NFR-04 | automated test + doctor proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
