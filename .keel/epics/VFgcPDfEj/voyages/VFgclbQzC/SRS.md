# Cloud Aws PVM Runtime Proof - SRS

## Summary

Epic: VFgcPDfEj
Goal: Route cloud-aws through the prepared AWS PVM lane and prove canonical launch, status, and stop against a live hosted control-plane and node-agent path with provider-aware failure behavior.

## Scope

### In Scope

- [SCOPE-01] Route canonical `cloud-aws` machine lifecycle commands through the prepared AWS hosted PVM lane.
- [SCOPE-02] Keep failure behavior provider-aware when AWS hosted PVM readiness or prerequisites are missing.
- [SCOPE-03] Publish the live hosted AWS PVM operator proof for prepare, launch, status, and stop.

### Out of Scope

- [SCOPE-04] Broader scheduler rollout, multi-node placement, or non-AWS hosted providers.
- [SCOPE-05] Generalizing the proof to `cloud-generic` or claiming arm64 hosted PVM support.
- [SCOPE-06] Reworking external AWS infrastructure setup beyond the Port runtime workflow.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port machine launch --machine cloud-aws`, `port machine status --machine cloud-aws`, and `port machine stop --machine cloud-aws` must route through the live hosted control-plane and node-agent path when an x86_64 AWS node advertises ready PVM preparation. | SCOPE-01 | FR-03 | manual |
| SRS-02 | If the AWS hosted PVM lane is not ready, Port must fail with actionable `cloud-aws` guidance and must not substitute the standard Firecracker/KVM path. | SCOPE-02 | FR-04 | automated |
| SRS-03 | Port must publish a canonical hosted AWS PVM proof that demonstrates `prepare-pvm-node` plus `cloud-aws` launch/status/stop on a prepared x86_64 AWS node. | SCOPE-03 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The hosted AWS routing and failure behavior must have focused automated regression coverage in the repository. | SCOPE-01, SCOPE-02 | NFR-02 | automated |
| SRS-NFR-02 | The proof and documentation must keep the operator boundary explicit: x86_64 AWS hosted PVM only. | SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
