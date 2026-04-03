# Export And Prove AWS PVM Host Kit Module - SRS

## Summary

Epic: VFhLhfrqk
Goal: Publish a downstream-consumable Nix module and supporting package metadata
for the AWS x86_64 PVM host kit, then document and verify the downstream AMI
build path.

## Scope

### In Scope

- [SCOPE-01] Export a first-class `port.nixosModules.aws-pvm-host` surface for
  downstream `nixosSystem` consumers.
- [SCOPE-02] Export a companion `firecracker-pvm-host-kit` package/metadata
  surface that reflects the canonical AWS x86_64 PVM host-kit identity already
  modeled by Port.
- [SCOPE-03] Make the exported module encode the AWS PVM host contract Port
  expects for doctor/readiness and document the downstream AMI build handoff so
  repos like `infra` can point at Port's exported module/package surface
  instead of inventing a local host-kit module.

### Out of Scope

- [SCOPE-04] AWS VM Import/Export automation, S3 upload/import orchestration,
  and AMI publication policy.
- [SCOPE-05] Downstream bootstrap, cluster provisioning, IAM, DNS, GitOps, or
  scheduler work.
- [SCOPE-06] arm64 PVM, GCP/Azure host-kit exports, or a generalized
  multi-provider confidential-host surface.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The flake exports `nixosModules.aws-pvm-host` and a downstream `nixosSystem` can evaluate with that module in `modules = [ ... ]`. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Port exports a companion `firecracker-pvm-host-kit` package/metadata surface whose package identity, host-kernel release, Firecracker build, binary name/env, and boot args match the canonical AWS x86_64 PVM host-kit contract. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The exported module configures the host so Port's canonical local PVM doctor checks are satisfiable for the AWS x86_64 host contract: host platform, host architecture, boot-line, and firecracker-binary surface. | SCOPE-03 | FR-03 | manual |
| SRS-04 | Port docs show the supported downstream AMI-build handoff using the exported module/package surface, not a downstream repo-local host-kit module. | SCOPE-03 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The exported Nix surface must derive from or stay mechanically aligned with Port-owned canonical host-kit data so the Nix contract cannot silently drift from `prepare-pvm-node`. | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | The implementation and docs must keep scope truthful: host-kit definition is in Port, but AMI import/export and downstream orchestration remain outside Port. | SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
