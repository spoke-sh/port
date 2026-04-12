# Ship Stable HA API Endpoint And Failover Proofs For Hosted AWS PVM - Product Requirements

## Problem Statement

The hosted AWS PVM lane does not yet provide a first-class stable API endpoint, failover posture, and proof surface for real HA control planes behind multiple execution hosts.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Provide one stable HA Kubernetes API endpoint for hosted AWS PVM control planes. | `port cluster kubeconfig` can hand off a stable endpoint that survives control-plane guest turnover or host loss in the first HA slice. | First stable-endpoint slice |
| GOAL-02 | Provide one human-reviewable failover proof surface for that endpoint. | Operators can review explicit evidence that the endpoint remains usable through a control-plane failure scenario. | First HA failover proof slice |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Infra Operator | Consumes Port kubeconfig handoff and API readiness downstream. | A stable endpoint instead of one guest-specific address. |
| Hosted Cluster Operator | Needs real HA, not only multi-host placement. | Clear failover behavior and inspection surfaces. |
| Review Surface Owner | Maintains Port's proof-backed mission contract. | Human-reviewable evidence for HA claims. |

## Scope

### In Scope

- [SCOPE-01] Stable API endpoint posture for hosted AWS PVM HA clusters.
- [SCOPE-02] Kubeconfig and status surfaces that expose that endpoint.
- [SCOPE-03] Failover proof and review artifacts for the first HA slice.

### Out of Scope

- [SCOPE-04] Control-plane spread logic itself; that belongs to the adjacent
  placement epic.
- [SCOPE-05] Generic multi-provider HA endpoint work.
- [SCOPE-06] Full multi-region or disaster-recovery posture.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must expose a stable HA API endpoint for hosted AWS PVM clusters that is not coupled to one control-plane guest IP. | GOAL-01 | must | Downstream kubeconfig handoff depends on endpoint stability. |
| FR-02 | `port cluster status`, `kubeconfig`, and related proof surfaces must report that stable endpoint and its HA posture explicitly. | GOAL-01, GOAL-02 | must | Operators and downstream tools need explicit truth, not inference. |
| FR-03 | Port must provide a proof-backed failover scenario showing the endpoint remains usable through the first supported control-plane loss condition. | GOAL-02 | must | HA claims need reviewable evidence. |
| FR-04 | Failure modes must stay explicit when the endpoint is not actually HA-capable or when failover prerequisites are missing. | GOAL-01, GOAL-02 | should | Silent partial HA would undermine the contract. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | HA endpoint proofs must remain human-reviewable through Port's canonical proof surfaces. | GOAL-02 | must | The mission contract depends on explicit review artifacts. |
| NFR-02 | Port must not claim stable HA if kubeconfig or endpoint recovery still requires manual downstream rewrites. | GOAL-01, GOAL-02 | must | Manual rewrite means the endpoint is not truly stable. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Stable endpoint handoff | Automated tests plus CLI proof | Story-level kubeconfig and status artifacts |
| Failover proof | Manual review plus scripted failure-path proof | Story-level review artifact and logs |
| Structural integrity | `keel doctor --status` | No board drift after decomposition and execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The placement epic will supply a real multi-host control-plane topology for this endpoint to front. | The epic could over-fit a non-HA topology. | Keep the dependency explicit during decomposition. |
| A first HA failover proof can stay bounded to one supported host/control-plane loss scenario. | The epic might over-scope into full disaster recovery. | Revisit only if the first proof would be misleading. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which stable endpoint mechanism should be canonical first for the AWS HA slice? | Epic owner | Open |
| Which failover scenario is the smallest honest first proof? | Mission owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Hosted AWS PVM clusters expose a stable HA API endpoint through Port's
  kubeconfig and status surfaces.
- [ ] Port has a human-reviewable failover proof for the first supported HA
  loss scenario.
- [ ] The HA claim fails honestly when stable-endpoint prerequisites are not
  satisfied.
<!-- END SUCCESS_CRITERIA -->
