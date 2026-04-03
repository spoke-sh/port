# Foundational AWS PVM Docs Refresh - SRS

## Summary

Epic: VFhICYohO
Goal: Converge the foundational docs and public AWS narrative on one clear x86_64 hosted PVM production contract.

## Scope

### In Scope

- [SCOPE-01] Simplify the foundational doc map so root docs point to one clear AWS production narrative.
- [SCOPE-02] Rewrite the focused hosted/cloud/PVM docs around the x86_64 AWS hosted Firecracker/PVM contract.
- [SCOPE-03] Align the public AWS narrative with the same production posture and explicit boundaries.

### Out of Scope

- [SCOPE-04] Runtime, scheduler, artifact, or protocol implementation changes.
- [SCOPE-05] New AWS infrastructure automation for EC2 provisioning, IAM, networking, or downstream GitOps.
- [SCOPE-06] Extending the production claim to GCP, Azure, or arm64 PVM.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Foundational docs must direct readers to one canonical AWS production path instead of scattering the contract across repeated summaries. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The documentation set must explain the AWS x86_64 hosted Firecracker/PVM lane in operational terms, including host kit, artifact kit, `prepare-pvm-node`, and canonical launch/status/stop flow. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The docs must distinguish the AWS PVM lane from the hosted standard lane and from repo-local proof harnesses without hiding those secondary surfaces. | SCOPE-01, SCOPE-02 | FR-03 | manual |
| SRS-04 | The docs must preserve explicit provider-aware and architecture-aware boundaries, including missing-host-kit failures, missing-artifact failures, no silent fallback, no arm64 PVM claim, and no implied GCP/Azure inheritance. | SCOPE-02, SCOPE-03 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The refreshed docs must reduce duplication and make the AWS/PVM narrative easier to navigate through deliberate cross-links. | SCOPE-01, SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Public docs and foundational docs must describe the same AWS production posture and boundaries. | SCOPE-01, SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
