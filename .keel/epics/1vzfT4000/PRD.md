# Hosted Service Reliability And Secret Hardening - Product Requirements

> Close the remaining Slicer-class hosted service gap by making `port service`
> restart-aware, health-aware, and safer for secret delivery without
> inventing a second hosted-only service model.

## Problem Statement

Port now ships hosted services and sandboxes through the canonical
`port service` surface, but the current runtime contract is still explicitly a
bootstrap slice:

- secret values are stored as runtime-owned JSON files
- restart policy is not enforced by a runtime supervisor
- health state is not surfaced through canonical `service status`
- scheduler policy is fixed to deterministic first-fit with no service-level
  policy contract

That leaves a meaningful operator gap against the requested Slicer-class
direction. Platform operators can place a service, but they cannot yet rely on
Port to keep it running, explain whether it is healthy, or handle secrets in a
way that is stronger than repo-local demo storage.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make service restart and health policy part of the canonical Port service model. | `port service apply|status|stop` can express and surface restart policy plus runtime health state | First implementation voyage complete |
| GOAL-02 | Harden secret delivery for hosted services and sandboxes. | Port stops treating plaintext runtime JSON as the canonical long-lived secret store for service execution | First implementation voyage complete |
| GOAL-03 | Preserve one canonical operator and client surface. | CLI, model, hosted API, and SDK all keep using `port service` and existing hosted routes | First implementation voyage complete |
| GOAL-04 | Keep the hosted scheduler story explicit and inspectable. | Operators can see policy, health, and restart state in status output and docs without reading runtime files | First implementation voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Platform Operator | Runs hosted Port nodes and expects service workloads to survive failures predictably. | A service plane that can restart workloads, report health, and explain current policy state |
| Application Operator | Runs services or sandboxes through Port instead of ad hoc SSH sessions. | Canonical service commands that can safely consume secrets and report whether a workload is actually healthy |
| Hosted Runtime Builder | Extends the control plane and node-agent runtime. | Stable model, route, and runtime contracts for policy, health, and secret materialization |

## Scope

### In Scope

- [SCOPE-01] Service restart-policy and health-policy contracts in the shared
  model, runtime state, CLI, hosted API, and SDK.
- [SCOPE-02] Hosted and local service runtime ownership changes needed to
  supervise a managed guest process rather than only storing desired state.
- [SCOPE-03] A stronger secret-backend and materialization contract for service
  and sandbox execution.
- [SCOPE-04] Operator docs, examples, and CLI help that publish the new
  reliability and secret-handling workflow and its remaining limits.

### Out of Scope

- [SCOPE-05] Full multi-tenant RBAC, external secret managers, or cloud KMS
  integrations.
- [SCOPE-06] Advanced placement policies such as autoscaling, priorities,
  preemption, or fair-share scheduling.
- [SCOPE-07] A second hosted-only services API or a second secret-management
  CLI family.
- [SCOPE-08] Production SLOs, billing, or organization-level tenancy.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must define restart-policy and health-policy contracts for services and sandboxes in the shared model and canonical `port service` surfaces. | GOAL-01, GOAL-03, GOAL-04 | must | Policy must be explicit and inspectable before runtime behavior can be trusted. |
| FR-02 | Port must implement managed-process supervision that can keep a service running according to its selected restart policy and record runtime health state under the existing runtime owner. | GOAL-01, GOAL-04 | must | Service definitions alone do not close the operator gap; Port must own restart behavior. |
| FR-03 | Port must replace plaintext runtime JSON secret values as the canonical execution input with a stronger secret-backend and materialization contract for `port service secret` and `port service apply`. | GOAL-02, GOAL-03 | must | Secret storage and delivery need a stronger story before hosted service adoption is credible. |
| FR-04 | Port must surface policy, restart, health, and secret-materialization behavior through the canonical CLI, hosted API, and SDK without introducing hosted-only aliases. | GOAL-03, GOAL-04 | must | Product coherence is a top-level user requirement. |
| FR-05 | Port must publish a repo-local operator workflow that proves a service can start, restart, report health, consume a secret, and stop through the shipped Port surface. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | should | Capability is not done until it is discoverable and executable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Restart, health, and secret state must remain deterministic and attributable to one runtime owner so `service status` is stable across local and hosted flows. | GOAL-01, GOAL-02, GOAL-04 | must | Operators need one place to inspect what Port believes is true. |
| NFR-02 | Unsupported policy, secret-backend, or runtime combinations must fail fast with explicit diagnostics and no fallback to legacy JSON-secret behavior. | GOAL-02, GOAL-03 | must | Hard cutover keeps the contract understandable and prevents silent weakening of the security story. |
| NFR-03 | The first voyage must leave the board with executable implementation stories and verification paths rather than another service-roadmap placeholder. | GOAL-01, GOAL-02, GOAL-03 | should | The user asked for autonomous delivery, not just backlog growth. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use model/runtime tests for policy parsing, runtime-state transitions, and
  secret-materialization ownership.
- Use hosted and local CLI proofs for `service apply|status|stop`,
  health-reporting, restart behavior, and secret consumption.
- Use docs/help review plus example workflows to prove discoverability.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Port's current managed-process runtime owner can be extended into a real supervisor without replacing the canonical `service` surface. | A separate job-runner model might be required. | Validate in the first runtime story. |
| A stronger in-repo secret backend can materially improve the hosted operator story before external secret-manager integrations exist. | The first hardening slice may not be worth shipping. | Validate through operator workflow and doc clarity in the first voyage. |
| Health state can start with simple process- and command-based checks before richer service probes or SLO systems exist. | The first health slice could under-serve operators. | Keep the health contract explicit and inspectable in docs and status output. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the right first hardened secret backend for Port: restricted on-disk store, encrypted local envelope, or a pure runtime materialization cache? | Product / Runtime | Open |
| Should restart policy initially cover services only, or both services and sandboxes through the same supervisor path? | Product / Runtime | Open |
| How much health-policy surface is required before `service status` is materially better than today's process state? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The board contains executable work that turns restart policy, health state, and stronger secret handling into concrete Port service behavior.
- [ ] Port can demonstrate a repo-local service workflow that consumes a secret, reports health, survives a restart according to policy, and stops through canonical Port verbs.
- [ ] README/docs/CLI help explain the shipped service reliability slice and its remaining limits without referring operators to an internal-only runtime model.
<!-- END SUCCESS_CRITERIA -->
