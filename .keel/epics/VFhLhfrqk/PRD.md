# AWS PVM Host Kit Nix Surface - Product Requirements

## Problem Statement

Port owns the AWS x86_64 PVM host contract conceptually, but downstream image
pipelines still cannot consume that contract as a first-class Nix module or
package surface. That forces AMI builders to depend on out-of-band repo-local
module paths instead of a Port-owned source of truth for the host kernel, boot
args, patched `firecracker-pvm`, and readiness identity used by
`prepare-pvm-node`.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Downstream infrastructure can consume Port's AWS PVM host contract as a first-class flake export instead of an out-of-band module file invented in another repo. | A downstream `nixosSystem` evaluates with `port.nixosModules.aws-pvm-host` as an imported module. | One clean downstream module import proof |
| GOAL-02 | Port's exported Nix surface stays aligned with the canonical AWS x86_64 PVM host-kit identity already used by `prepare-pvm-node`, doctor checks, and sample inventory. | Exported module/package metadata matches the canonical host-kit contract and doctor-facing binary/boot-arg surface. | One canonical host-kit identity across Nix, model, and docs |
| GOAL-03 | Operators can follow a documented downstream AMI build path that points at Port's exported host-kit surface rather than a downstream repo-local host-kit module. | AWS/Nix docs show the supported handoff for downstream AMI builds. | One documented downstream handoff path |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Downstream Infra Maintainer | Owns Nix-authored AWS image builds, AMI publication, and AMI consumption in downstream repos such as `infra`. | A Port-owned module/package surface that can be imported or referenced without recreating the host-kit contract downstream. |
| Port Runtime Maintainer | Owns the Port-side host-kit, doctor, and hosted AWS PVM contract. | One authoritative source of truth for the AWS PVM host-kit identity and downstream guidance. |

## Scope

### In Scope

- [SCOPE-01] Flake exports for the AWS x86_64 PVM host-kit module and companion package/metadata surface.
- [SCOPE-02] Port-owned NixOS module behavior that encodes the canonical AWS PVM host contract needed for the doctor/readiness path.
- [SCOPE-03] Docs and proof showing downstream AMI-build consumption of the Port-owned surface.

### Out of Scope

- [SCOPE-04] AWS VM Import/Export automation, S3 upload/import orchestration, and AMI publication policy.
- [SCOPE-05] Downstream bootstrap, IAM, DNS, scheduler, or GitOps orchestration.
- [SCOPE-06] Generalizing the exported surface to arm64, GCP, Azure, or a broader host taxonomy.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Export `port.nixosModules.aws-pvm-host` as a downstream-consumable module that can be imported by `nixosSystem`. | GOAL-01 | must | This is the minimum first-class flake surface requested by downstream consumers. |
| FR-02 | Export a companion package/metadata surface for the AWS PVM host kit that matches the canonical Port host-kit identity used by `prepare-pvm-node`. | GOAL-02 | should | A package surface makes the contract consumable beyond a raw module import and gives docs a stable downstream handoff path. |
| FR-03 | Make the exported module own the Port-specific AWS PVM host contract: Linux/x86_64 posture, required boot args, canonical `firecracker-pvm` path/env surface, and host-kit metadata. | GOAL-02 | must | The module must express the contract Port already expects from prepared AWS PVM hosts. |
| FR-04 | Document the downstream AMI build handoff so downstream repos can point at Port's exported surface rather than inventing a repo-local host-kit module. | GOAL-03 | must | The code surface is incomplete if operators still need to rediscover or reconstruct the handoff. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The exported Nix host-kit identity must be derived from or validated against Port-owned canonical data so the module/package surface does not drift from `prepare-pvm-node`. | GOAL-02 | must | Drift between Nix exports and `prepare-pvm-node` would recreate the exact downstream problem this epic is trying to solve. |
| NFR-02 | The implementation must stay truthful about scope: Port exports the host-kit surface, but does not claim AWS import/export automation or broader downstream orchestration. | GOAL-03 | must | Clear boundaries keep docs and product posture honest. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Flake export | Nix evaluation or build proof | Story evidence showing a downstream `nixosSystem` import and the exported package/module surfaces |
| Host-kit alignment | Config inspection and contract review | Story evidence linking Nix metadata to canonical Port host-kit identity and doctor-facing checks |
| Docs handoff | Manual doc review | Story evidence citing the updated downstream AMI build guidance |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Port can ship an immediately useful first-class host-kit export even if downstream infrastructure continues to own AMI build/import orchestration. | The epic would overreach into infra automation or fail to close the actual seam. | Keep the scope constrained to module/package export plus downstream handoff docs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The repo does not currently carry a separate in-tree patched Firecracker derivation or custom host-kernel patch set. The exported Nix surface must therefore stay explicit about the contract it owns and how the binary/kernel source is surfaced. | Epic owner | Active |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Port exports a first-class AWS PVM host-kit module and companion surface that downstreams can consume without recreating the contract.
- [ ] The exported surface matches the canonical Port host-kit identity used by the hosted AWS PVM readiness path.
- [ ] Port docs show the supported downstream AMI handoff using the Port-owned export.
<!-- END SUCCESS_CRITERIA -->
