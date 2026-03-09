# Enable Hosted Standard Cloud Launch - Software Requirements Specification

> Route the sample `generic-linux`, `aws`, and `gcp` standard Firecracker lanes through the live hosted control-plane and node-agent path so remote cloud launch becomes executable instead of guidance-only.

**Epic:** [1vzXFf000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:

- [SCOPE-01] Admit `standard` provider-backed Firecracker machines onto
  registered hosted nodes using the current control-plane and node-agent
  contract.
- [SCOPE-02] Route canonical `machine launch|status|stop` through that hosted
  runtime owner for `cloud-generic`, `cloud-aws`, and `cloud-gcp`.
- [SCOPE-03] Publish operator-facing docs, help text, and command proof for the
  shipped hosted standard-lane workflow.

Out of scope:

- [SCOPE-04] Direct SSH orchestration or agentless remote launch.
- [SCOPE-05] Cloud Hypervisor, OCI artifact transport, or new hypervisor work.
- [SCOPE-06] Scheduler policy, health checks, or hardened hosted secret
  backends.
- [SCOPE-07] New PVM work beyond the already shipped prepared-node lane.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Registered hosted nodes remain the canonical ownership boundary for remote launch. | dependency | The voyage would need a different control-plane or daemon contract. |
| The sample `generic-linux-node`, `aws-linux-node`, and `gcp-linux-node` stay the operator-facing proof set. | assumption | The voyage would need new sample config or provider planning before verification. |
| Existing hosted status, stop, and guest routing continue to use stored placement once a machine launches successfully. | dependency | The voyage would need follow-on runtime work beyond launch routing. |

## Constraints

- No silent fallback from hosted provider-backed launch to local runtime
  ownership.
- Provider identity must remain visible in placement, status, and failure
  output.
- The canonical CLI stays `port machine ...`; do not introduce a second remote
  machine command family.
- Verification must use repo-local hosted proofs with the existing demo control
  plane and node agents.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must resolve candidate and selected hosted nodes for `standard` provider-backed machines (`generic-linux`, `aws`, `gcp`) without erasing provider identity, and reject unresolved or ineligible nodes with explicit routing context. | SCOPE-01 | FR-01 | automated test + command proof |
| SRS-02 | `port machine launch --machine <cloud-machine>` must route `cloud-generic`, `cloud-aws`, and `cloud-gcp` through the configured hosted control plane and node agent so the selected node owns the runtime root and hypervisor process for the standard lane. | SCOPE-01, SCOPE-02 | FR-02 | automated test + command proof |
| SRS-03 | `port machine status` and `port machine stop` must keep working for those hosted standard-lane machines using stored placement, and their output must include hosted routing and provider context. | SCOPE-02 | FR-02 | automated test + command proof |
| SRS-04 | README, `docs/cloud.md`, `docs/hosted.md`, and CLI help must publish the shipped hosted standard cloud workflow and remove stale guidance that tells operators to run Port on the provider host directly for these demo lanes. | SCOPE-03 | FR-03 | doc/help proof + command proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted standard-lane failures must include machine, host, provider, control plane, and candidate or selected node detail instead of generic unsupported-host output. | SCOPE-01, SCOPE-02 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | The hosted standard-lane changes must preserve the existing local Linux and prepared-node PVM launch paths. | SCOPE-01, SCOPE-02 | NFR-02 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Planned Story Slices

| Story | Outcome | Requirements |
|-------|---------|--------------|
| Define Hosted Standard Placement Contract | Provider-backed standard machines resolve onto explicit candidate nodes with actionable rejection and placement detail. | SRS-01, SRS-NFR-01 |
| Route Standard Cloud Launch Through Hosted Runtime | Canonical `machine launch|status|stop` run through the hosted control plane and node agent for the sample cloud machines. | SRS-02, SRS-03, SRS-NFR-01, SRS-NFR-02 |
| Publish Hosted Standard Cloud Workflow | Canonical docs and help publish the shipped workflow and replace stale guidance with executable proof. | SRS-04, SRS-NFR-02 |
