# Hosted Standard Cloud Launch - Product Requirements

## Problem Statement

Port already models provider-backed remote Linux machines and ships the first
hosted control-plane plus node-agent runtime ownership split, but the standard
`generic-linux`, `aws`, and `gcp` lanes still stop at provider-aware denial
text. That leaves the hosted product story incomplete for the most practical
cloud-cost lane: standard Firecracker execution on registered remote Linux
nodes.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship executable hosted standard-lane launch for provider-backed remote Linux machines. | `port machine launch`, `status`, and `stop` succeed for `cloud-generic`, `cloud-aws`, and `cloud-gcp` through the hosted control plane and registered node agents in repo-local proof. | All three sample providers launch through the hosted demo lane. |
| GOAL-02 | Keep provider identity explicit while moving from guidance-only to real hosted placement. | CLI output, machine status, and failure paths always include machine, host, provider, control plane, and candidate or selected node context. | No provider-backed standard launch silently falls back to local runtime ownership. |
| GOAL-03 | Publish the new operator workflow at the canonical surfaces. | README, `docs/cloud.md`, `docs/hosted.md`, and CLI help replace stale “run Port on that host directly” guidance for the shipped hosted standard lane. | A new operator can follow one documented hosted standard-lane workflow end-to-end. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Hosted Operator | Runs Port from a control-plane context and needs to target remote Linux nodes without manual per-host CLI ownership. | A real hosted launch path that keeps provider and placement context visible. |
| Platform Engineer | Maintains the node-agent fleet and needs deterministic routing, failure context, and repeatable proofs for provider-backed nodes. | Explicit node admission, runtime ownership, and verification evidence. |

## Scope

### In Scope

- [SCOPE-01] Admit provider-backed `standard` Firecracker machines onto
  registered hosted nodes through the current control-plane and node-agent
  stack.
- [SCOPE-02] Route canonical `machine launch|status|stop` through that hosted
  runtime path for the sample `generic-linux`, `aws`, and `gcp` hosts and
  machines.
- [SCOPE-03] Publish CLI help, README, cloud docs, and hosted docs for the
  shipped hosted standard-lane workflow.

### Out of Scope

- [SCOPE-04] Direct SSH orchestration or agentless remote launch.
- [SCOPE-05] Cloud Hypervisor, OCI artifact mobility, or additional hypervisor
  work.
- [SCOPE-06] New scheduler policy, health checks, or hardened hosted secret
  backends.
- [SCOPE-07] Firecracker/PVM beyond the already shipped prepared-node lane.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must admit `standard` provider-backed remote Linux machines onto eligible registered hosted nodes without erasing provider identity. | GOAL-01, GOAL-02 | must | Hosted standard-lane launch needs deterministic placement before runtime routing can exist. |
| FR-02 | Port must route canonical `machine launch`, `status`, and `stop` through the hosted control plane and node agent for the sample `generic-linux`, `aws`, and `gcp` machines. | GOAL-01 | must | This is the executable outcome that removes guidance-only remote launch for the standard lane. |
| FR-03 | Port must publish the hosted standard-lane workflow through the canonical CLI and operator docs. | GOAL-03 | must | A capability is not shipped until it is discoverable and learnable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Hosted standard-lane launch must fail fast with explicit machine, host, provider, control plane, and candidate or selected node detail when placement or routing fails. | GOAL-02 | must | Hosted launch is only operable if routing failures are explicit. |
| NFR-02 | The hosted standard-lane implementation must preserve the existing local Linux and prepared-node PVM launch paths. | GOAL-01 | must | This new lane cannot regress the already shipped runtime paths. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Hosted placement and routing | Rust tests plus repo-local CLI proofs against the demo control plane and node agents | Story-level command logs for launch, status, stop, and failure paths |
| Operator discovery | CLI help inspection plus README and docs review | Story-level doc/help evidence tied to the published workflow |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The registered-node hosted demo lane remains the canonical first execution path for remote standard-lane work. | A different hosted ownership model would need separate planning before implementation. | Validate against `docs/hosted.md` and existing control-plane code. |
| The sample `generic-linux-node`, `aws-linux-node`, and `gcp-linux-node` are sufficient to prove provider-aware hosted standard launch without introducing new providers. | The epic would need broader config or provider planning work before implementation. | Validate against `examples/port.toml` during execution. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much placement work is already covered by the hosted service and PVM slices, and where do standard-lane constraints diverge? | Epic owner | Open |
| Some providers may still require config or artifact-path adjustments before the first end-to-end proof. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `cloud-generic`, `cloud-aws`, and `cloud-gcp` launch through the hosted control plane and node-agent path with recorded evidence.
- [ ] Provider-aware hosted status and stop work for those launched machines without falling back to local runtime ownership.
- [ ] Canonical docs and CLI help publish the hosted standard cloud workflow and remove stale guidance for the shipped demo lane.
<!-- END SUCCESS_CRITERIA -->
