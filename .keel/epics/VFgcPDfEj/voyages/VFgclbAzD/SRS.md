# AWS PVM Host Kit Preparation - SRS

## Summary

Epic: VFgcPDfEj
Goal: Define and implement the AWS-specific prepared-host contract so a normal x86_64 AWS Linux node can advertise cloud-aws PVM readiness through Port-managed preparation and imported inventory surfaces.

## Scope

### In Scope

- [SCOPE-01] Define the provider-specific host-kit contract for `cloud-aws` on x86_64 AWS Linux, including custom kernel, `pti=off`, patched `firecracker-pvm`, and PVM artifact expectations.
- [SCOPE-02] Teach the canonical preparation/import path to materialize AWS hosted PVM readiness without manual config overlays or generic-node substitution.
- [SCOPE-03] Surface AWS hosted PVM readiness and prerequisite failures through operator-visible readiness, doctor, or status outputs.

### Out of Scope

- [SCOPE-04] Live `cloud-aws` machine launch/status/stop proof through the hosted control plane.
- [SCOPE-05] arm64 prepared-host enablement or non-AWS hosted providers.
- [SCOPE-06] EC2 provisioning, IAM, DNS, or broader infrastructure rollout outside Port's runtime contract.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define an explicit AWS x86_64 hosted PVM prepared-host contract for `cloud-aws` that captures custom kernel, `pti=off`, patched `firecracker-pvm`, host-kit identity, and required PVM artifact-kit availability. | SCOPE-01 | FR-01 | manual |
| SRS-02 | `port control-plane prepare-pvm-node` must move an eligible AWS node from planned to ready for the hosted PVM lane without requiring manual imported-inventory edits or custom config overlays. | SCOPE-02 | FR-02 | manual |
| SRS-03 | Port must surface missing, stale, or mismatched AWS hosted PVM readiness with provider-aware context that keeps `cloud-aws` separate from the standard Firecracker/KVM lane. | SCOPE-03 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Planning and operator-facing contract text must keep this voyage scoped to x86_64 AWS hosted PVM only. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | manual |
| SRS-NFR-02 | Readiness information must be inspectable through canonical Port surfaces rather than hidden in ad hoc local files or one-off operator steps. | SCOPE-02, SCOPE-03 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
