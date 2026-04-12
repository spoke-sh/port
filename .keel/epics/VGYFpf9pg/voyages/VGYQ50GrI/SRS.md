# Prove Runtime Class Identity And Guard Rails - SRS

> Deliver the promotion-runner execution contract as a clean-room runtime class
> that stays inspectably distinct from scratch authoring.

**Epic:** [VGYFpf9pg](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Defining the canonical
  `blessed-closure-promotion-runner` runtime-class contract in Port.
- [SCOPE-02] Enforcing clean-room input, state, and trust boundaries distinct
  from `workspace-scratch-builder`.
- [SCOPE-03] Surfacing promotion-runner execution identity and guard-rail
  posture through Port-authored machine contracts and proof surfaces.

### Out of Scope

- [SCOPE-04] Publication manifests, signing keys, cache promotion policy, or
  rollback pointer ownership.
- [SCOPE-05] Scratch-builder writable-state implementation beyond the shared
  contract surface already established in the builder epic.
- [SCOPE-06] Spoke approval UX or lineage authority.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The shared runtime-class vocabulary from the builder epic lands first. | dependency | Promotion work would risk inventing a second incompatible contract. |
| Promotion runner execution can be represented as a constrained machine lane in Port. | assumption | Port would need a broader execution abstraction than this mission currently scopes. |
| Downstream `infra` owns evidence, publication, and rollback state even when Port exposes execution proof. | dependency | Port could overreach into publication-system ownership. |

## Constraints

- Promotion runner trust must never be represented as an upgrade bit on a
  scratch builder.
- Port must expose execution and proof only; publication policy and signing
  stay out of scope.
- Verification must include negative-path proof that scratch and promotion
  remain distinct in identity, writable state, and trust material.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define `blessed-closure-promotion-runner` as a dedicated runtime class rather than as a trust flag on `workspace-scratch-builder`. | SCOPE-01, SCOPE-02 | FR-01 | automated Rust tests + config round-trip |
| SRS-02 | The promotion-runner contract must declare immutable-input and clean-room posture explicitly, including the absence of inherited scratch writable state. | SCOPE-01, SCOPE-02 | FR-02 | automated Rust tests + inspection proof |
| SRS-03 | Port machine-facing contracts must surface promotion-runner identity, declared-input posture, and trust-material expectations so downstream publication tooling can link runtime proof to what ran. | SCOPE-03 | FR-03 | automated Rust tests + CLI proof |
| SRS-04 | Port validation must reject configurations that attempt to reuse scratch writable state or creator credentials for the promotion runner. | SCOPE-02, SCOPE-03 | FR-02 | automated Rust tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Promotion execution must remain distinct from scratch execution in identity, writable state, and proof artifacts across both local and AWS lanes. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | code inspection + automated Rust tests |
| SRS-NFR-02 | The promotion-runner slice must not absorb signing, publication, or rollback policy ownership into Port while adding the runtime class. | SCOPE-01, SCOPE-03 | NFR-02 | planning review + code inspection |
| SRS-NFR-03 | Verification for this voyage must include negative-path proof for collapsed trust boundaries, not only happy-path serialization checks. | SCOPE-02, SCOPE-03 | NFR-01 | story evidence + targeted tests |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| Promotion-runner runtime-class contract and validation slice | SRS-01, SRS-02, SRS-04 |
| Promotion-runner inspection and proof surfaces | SRS-03, SRS-NFR-01, SRS-NFR-02, SRS-NFR-03 |
