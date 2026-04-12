# Define Builder And Promotion Runtime Class Contracts - SRS

> Establish the shared Port runtime-class contract that makes
> `workspace-scratch-builder` an explicit, inspectable lane without smuggling
> trust or publication policy into Port.

**Epic:** [VGYFpewpf](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Extending the shared Port model with explicit runtime-class
  metadata for machine-backed execution lanes so
  `workspace-scratch-builder` is a canonical contract instead of an ad hoc
  convention.
- [SCOPE-02] Defining the canonical `workspace-scratch-builder` contract in
  Port, including trust posture, workspace binding, writable-state
  expectations, and machine-facing proof surfaces.
- [SCOPE-03] Keeping the runtime-class contract environment-agnostic so the
  same builder vocabulary applies across local and AWS lanes.

### Out of Scope

- [SCOPE-04] Trusted publication or promotion-runner behavior.
- [SCOPE-05] Spoke admission, lineage, or creator-facing workspace UX.
- [SCOPE-06] Cache signing, publication ownership, or blessed-cache policy.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Port's current machine model is still the right execution seam for builder lanes. | assumption | The mission would need a new top-level runtime object instead of a constrained machine contract. |
| Downstream `infra` and `spoke` work can consume explicit runtime-class identity without Port owning their policy state. | dependency | The shared contract could drift or reintroduce duplicated lifecycle state downstream. |
| Builder writable state can be modeled explicitly before every environment-specific storage backend is implemented. | assumption | The first slice would need to overreach into storage provisioning instead of locking the contract first. |

## Constraints

- Keep the ownership split explicit: Port owns runtime class, execution
  identity, writable-state contract, and proof surfaces, but not promotion or
  signing policy.
- `workspace-scratch-builder` and
  `blessed-closure-promotion-runner` must remain distinct names with distinct
  trust posture; no mode-bit upgrade path is allowed.
- The first execution slice must improve model and inspection truth first
  instead of hiding the contract in docs or shell scripts.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model runtime classes explicitly in the shared configuration and machine contract surface rather than relying on ad hoc machine naming or comments. | SCOPE-01 | FR-01 | automated Rust tests + config round-trip |
| SRS-02 | Port must define a canonical `workspace-scratch-builder` runtime class whose contract names the workspace-bound writable-state categories and bounded trust posture expected by the lane. | SCOPE-02 | FR-02 | automated Rust tests + inspection proof |
| SRS-03 | `port machine status`, machine launch metadata, and related machine-facing contracts must surface runtime-class identity and posture so downstream operators can inspect what ran and why. | SCOPE-02 | FR-04 | automated Rust tests + CLI proof |
| SRS-04 | Port validation must reject impossible or unsafe runtime-class declarations, including attempts to mark a scratch builder as publish-trusted or to omit its writable-state contract. | SCOPE-02 | FR-03 | automated Rust tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The shared runtime-class contract must stay environment-agnostic so the same builder vocabulary applies to local and AWS lanes without changing its meaning. | SCOPE-03 | NFR-01 | code inspection + automated Rust tests |
| SRS-NFR-02 | The builder lane must remain explicitly untrusted by contract; Port must not infer publish rights, signing rights, or cluster-admin reachability from the runtime class. | SCOPE-02 | NFR-02 | automated Rust tests + CLI inspection proof |
| SRS-NFR-03 | Verification for this voyage must prove the shared model, validation failures, and operator-visible inspection surfaces rather than relying on prose-only documentation. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | story evidence + targeted command proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| Runtime-class model and validation slice | SRS-01, SRS-02, SRS-04, SRS-NFR-01 |
| Machine-facing inspection surfaces for runtime classes | SRS-03, SRS-NFR-02, SRS-NFR-03 |
