# AWS Hosted PVM Preparation And Launch - Product Requirements

## Problem Statement

Port has a generic prepared-node x86_64 Firecracker/PVM proof, but it still lacks a real provider-backed cloud-aws hosted runtime contract on regular AWS VMs. Downstream infrastructure currently falls onto the wrong standard Firecracker/KVM lane because Port does not yet own the AWS-specific host-kit preparation, readiness import, and live cloud-aws PVM proof surface.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Turn `cloud-aws` into a real hosted PVM lane on prepared x86_64 AWS Linux nodes. | Canonical `prepare-pvm-node`, `machine launch`, `machine status`, and `machine stop` complete against the live hosted control-plane and node-agent path. | One live AWS hosted PVM proof using the canonical commands |
| GOAL-02 | Make AWS prepared-host readiness explicit and Port-owned. | Port can prepare a node and prove imported readiness for host kit, custom kernel, `pti=off`, patched `firecracker-pvm`, and PVM artifacts without manual overlays. | One AWS host-kit preparation contract plus readiness evidence |
| GOAL-03 | Keep failure behavior honest and provider-aware. | Missing AWS-specific prerequisites fail with actionable `cloud-aws` guidance and do not fall back to the standard Firecracker/KVM lane. | One verified failure surface for each prerequisite class in scope |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Operator | A maintainer or evaluator proving Port on hosted infrastructure. | A canonical `cloud-aws` workflow that actually launches PVM-backed machines on a prepared AWS node. |
| Infrastructure Builder | The person preparing AWS nodes for Port-managed compute. | A concrete host-kit and readiness contract that does not require ad hoc overlays or generic-node substitution. |

## Scope

### In Scope

- [SCOPE-01] AWS-specific prepared-host contract work for x86_64 AWS Linux nodes, including host kit, readiness import, doctor/status surfacing, and PVM artifact expectations.
- [SCOPE-02] `cloud-aws` runtime routing and hosted control-plane/node-agent changes required so canonical `machine launch/status/stop` use the prepared AWS PVM lane.
- [SCOPE-03] Operator-facing proof and failure messaging for the hosted AWS PVM workflow.

### Out of Scope

- [SCOPE-04] EC2 provisioning, IAM, DNS, GitOps rollout, or broader cloud infrastructure automation beyond the Port runtime and host-preparation contract.
- [SCOPE-05] arm64 hosted PVM enablement, GCP or Azure hosted PVM rollout, or a generalized multi-provider scheduler contract.
- [SCOPE-06] Falling back to the standard Firecracker/KVM lane or bare-metal-only proof paths.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must expose an AWS prepared-host contract for x86_64 Linux nodes that verifies custom kernel, `pti=off`, patched `firecracker-pvm`, and required PVM artifacts through canonical preparation and readiness surfaces. | GOAL-02 | must | `cloud-aws` cannot be a real hosted lane until Port owns the AWS host prerequisites explicitly. |
| FR-02 | `port control-plane prepare-pvm-node` must prepare or import the AWS-specific host-kit readiness required for `cloud-aws` on a prepared x86_64 AWS Linux node without manual config overlays. | GOAL-01, GOAL-02 | must | Operators need one canonical preparation path rather than a manual readiness dance. |
| FR-03 | `port machine launch --machine cloud-aws`, `port machine status --machine cloud-aws`, and `port machine stop --machine cloud-aws` must route through the prepared AWS hosted PVM lane and succeed against the live hosted control-plane/node-agent path. | GOAL-01 | must | The hosted AWS lane is only real if canonical runtime commands work end to end. |
| FR-04 | Port must reject missing or stale AWS PVM prerequisites with actionable `cloud-aws` guidance and must not substitute the standard Firecracker/KVM lane. | GOAL-03 | must | Silent fallback would hide the real provider contract gap and reproduce the current failure mode. |
| FR-05 | Port must publish a canonical hosted AWS PVM proof surface that demonstrates the live AWS workflow and documents the AWS-only scope boundary. | GOAL-01, GOAL-03 | should | Operators need an explicit proof artifact and scope boundary once the lane exists. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The epic must preserve clear operator visibility through doctor, status, or imported-inventory surfaces so AWS readiness can be diagnosed without reading source code. | GOAL-02, GOAL-03 | must | Hosted PVM prep is operationally unsafe if readiness is implicit. |
| NFR-02 | Verification must include both live hosted AWS proof and focused automated coverage for routing and failure-path regressions. | GOAL-01, GOAL-03 | must | The contract needs both real proof and regression protection. |
| NFR-03 | Planning and implementation artifacts must keep scope explicitly limited to x86_64 AWS hosted PVM. | GOAL-03 | must | Scope control is part of the mission contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| AWS host preparation | Manual node preparation plus doctor/status/imported-readiness inspection | Story-level proof from a prepared x86_64 AWS node |
| Hosted runtime path | Live `cloud-aws` launch/status/stop workflow through the hosted control-plane and node agent | Operator proof artifact and command transcript |
| Failure behavior | Focused automated tests plus manual spot checks for missing prerequisites | Test coverage and failure-path transcripts |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Port can either build or consume an AWS host-kit artifact without expanding scope into general infrastructure provisioning. | The mission may stall on a product decision about host-kit ownership. | Treat host-kit ownership as the explicit yield decision if it cannot be resolved during implementation. |
| The current prepared-node PVM work and docs already provide enough baseline behavior to seed an AWS-specific hosted lane rather than starting from a blank design. | The epic could grow into a much larger runtime redesign. | Keep the first voyages tightly scoped to host preparation and runtime proof. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should Port build the AWS host kit directly or consume an external host-kit artifact as its canonical input? | Epic owner | Open |
| Live hosted AWS proof depends on access to a prepared x86_64 AWS node with the hosted control-plane path available. | Epic owner | Active risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A prepared x86_64 AWS Linux node can run canonical `prepare-pvm-node` and `cloud-aws` `machine launch/status/stop` through the live hosted PVM lane.
- [ ] Port exposes an AWS-specific host-preparation and readiness contract without manual config overlays or generic-node substitution.
- [ ] Missing AWS PVM prerequisites fail with actionable `cloud-aws` guidance and no standard-lane fallback.
- [ ] Planning and implementation stay explicitly scoped to x86_64 AWS hosted PVM only.
<!-- END SUCCESS_CRITERIA -->
