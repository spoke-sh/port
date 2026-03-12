# Hybrid Local Remote And SSH Execution - Product Requirements

> Port can become a credible hybrid execution tool if one canonical CLI and guest
model spans local machines, hosted control-plane execution, and SSH-first
remote Linux workflows without forcing operators into separate toolchains.

## Problem Statement

Port already models remote Linux providers and ships a live hosted control
plane plus node-agent split, but direct remote SSH usage is still a boundary
instead of a first-class product surface. The user wants to deploy to the
cloud and operate across local plus remote environments with first-class SSH.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define one canonical hybrid execution contract for local runtime, hosted control plane, and SSH-first remote Linux ownership. | `port` planning artifacts and CLI-facing route or ownership language describe one shared machine and guest model instead of separate local, hosted, and remote vocabularies. | Epic PRD, voyage SRS/SDD, and downstream stories all preserve one command family with explicit ownership boundaries. |
| GOAL-02 | Ship the first executable SSH-first remote Linux workflow through the canonical machine lifecycle surfaces. | A scoped voyage lands `machine launch`, `status`, and `stop` expectations for a remote Linux host reachable over SSH with proof-backed verification. | One bounded SSH-first delivery slice is planned and ready for operator execution. |
| GOAL-03 | Make remote auth, bootstrap, and readiness explicit for operators before execution begins. | `port doctor`, docs, and planning artifacts distinguish local prerequisites from remote-host readiness, auth material, and bootstrap requirements. | Operators can tell whether a machine targets local runtime, hosted control plane, or SSH-managed remote ownership before launch. |
| GOAL-04 | Preserve and extend the existing local and hosted product lanes rather than replacing them. | The epic leaves local Linux and hosted control-plane flows intact and treats SSH as an additional ownership lane, not a compatibility shim or second CLI. | No planned slice relies on fallback from SSH to local runtime or on a new remote-only command family. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Solo Operator | Runs Port from a laptop or workstation and wants to target a remote Linux host directly without standing up a hosted fleet first. | A canonical SSH-first path that feels like the same Port product, not a second orchestration tool. |
| Platform Engineer | Maintains remote Linux execution hosts and wants clear ownership of auth, bootstrap, diagnostics, and runtime state. | Explicit route, readiness, and failure semantics for remote hosts. |
| Hosted Operator | Already uses the control-plane and node-agent path and needs SSH work to align with hosted ownership rather than compete with it. | One mental model across local, hosted, and direct remote operation. |

## Scope

### In Scope

- [SCOPE-01] Canonical host and machine ownership language for `local`,
  `hosted-control-plane`, and future `ssh` connection modes.
- [SCOPE-02] The first SSH-first remote Linux execution slice through the
  canonical `machine` lifecycle and guest-operation surfaces.
- [SCOPE-03] Remote readiness, auth, bootstrap, and doctor/help guidance for
  operators targeting SSH-managed Linux hosts.
- [SCOPE-04] Human-reviewable proof and operator documentation for the hybrid
  execution contract.

### Out of Scope

- [SCOPE-90] Multi-node scheduler policy, autoscaling, or hosted fleet
  management beyond the existing control-plane and node-agent contract.
- [SCOPE-91] Cloud credential brokering, provider-specific network automation,
  or new provider onboarding beyond the current provider-aware model.
- [SCOPE-92] Kubernetes, GPU orchestration, or broader workload products that
  depend on hybrid execution but do not define it.
- [SCOPE-93] A second remote-only CLI or compatibility bridge that preserves
  legacy route semantics alongside the new canonical path.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must model hybrid execution with one canonical host-connection contract that covers local runtime ownership, hosted control-plane ownership, and SSH-first remote Linux ownership. | GOAL-01, GOAL-04 | must | The product cannot support hybrid execution credibly if each ownership lane invents separate route semantics. |
| FR-02 | Port must support a first scoped SSH-first remote Linux workflow through canonical `machine launch`, `status`, and `stop` semantics rather than requiring operators to run Port manually on the remote host. | GOAL-02 | must | The epic exists to convert SSH from a prose seam into a real product lane. |
| FR-03 | Port must surface remote readiness, auth, and bootstrap expectations through `port doctor`, help text, and operator docs before an SSH-targeted launch begins. | GOAL-03 | must | Remote execution is unusable if operators cannot tell what must exist on the remote host or how auth is supplied. |
| FR-04 | Port must keep provider, host, route, and ownership context explicit across local, hosted, and SSH lanes. | GOAL-01, GOAL-03, GOAL-04 | must | Hybrid execution becomes opaque and error-prone if route or owner context is hidden. |
| FR-05 | Port must publish a proof-backed operator workflow for the hybrid execution contract, including at least one human-reviewable artifact path. | GOAL-02, GOAL-03 | should | The first hybrid slice needs a reviewable demonstration, not only code and prose. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Hybrid execution must fail fast with explicit route, host, provider, and ownership detail when a target cannot use the requested lane. | GOAL-03, GOAL-04 | must | Operators need actionable failure context instead of implicit fallback or vague remote errors. |
| NFR-02 | The SSH-first lane must preserve the shipped local Linux and hosted control-plane workflows without silent behavior changes. | GOAL-04 | must | Hybrid work cannot regress the current credible product lanes. |
| NFR-03 | Verification for the first hybrid slice must use recommended repo-local techniques, including Rust tests and at least one human-reviewable proof path. | GOAL-02, GOAL-03 | must | The board should carry executable evidence, not only planning prose. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove hybrid route and lifecycle behavior through story-level Rust tests and
  repo-local command proofs mapped to voyage requirements.
- Use the proof system for at least one human-reviewable artifact path, such as
  a recording-backed operator workflow for SSH-first launch or diagnostics.
- Validate non-functional posture with explicit failure-path tests, doctor
  proofs, and documented ownership artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing local and hosted command model remains the correct foundation for adding SSH-first remote execution. | The epic would need a broader CLI or runtime redesign instead of a bounded hybrid slice. | Validate against `docs/cloud.md`, `docs/hosted.md`, and current CLI output before implementation. |
| A first SSH-first slice can target one remote Linux host deterministically before broader fleet placement or scheduler work exists. | The epic would need to start with hosted-fleet or scheduler infrastructure instead of direct remote ownership. | Validate in the first voyage design and story decomposition. |
| Remote-host readiness can be expressed clearly through `port doctor` and docs without requiring cloud-provider credential automation in the first slice. | The first voyage would need to expand into provider-credential management work. | Validate against current doctor surfaces and the shipped provider-aware model. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first SSH lane own only machine lifecycle, or should guest operations like `exec` and `copy` land in the same voyage? | Epic owner | Open |
| How should remote auth material be supplied and rotated without drifting away from the hosted auth contract? | Epic owner | Open |
| Some route and ownership enums only model local and hosted today, so introducing SSH may require broader route-surface updates than the first slice expects. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The epic defines one canonical ownership and route model for local, hosted, and SSH-first remote execution.
- [ ] At least one voyage plans a concrete SSH-first remote Linux slice with executable stories and traceable verification.
- [ ] Auth, bootstrap, and `port doctor` behavior are explicit for SSH-managed remote hosts before launch work begins.
- [ ] The hybrid execution plan preserves one CLI and guest vocabulary instead of inventing a second remote-only toolchain.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Findings

- The hybrid foundation already exists in the current control-plane and cloud
  docs [SRC-01][SRC-02].
- SSH is explicitly present as the next seam to productize rather than a new
  idea to invent from scratch [SRC-03][SRC-04].
- This work preserves the user's preferred local-plus-remote toolchain shape
  without requiring a second command family [SRC-01][SRC-02].

### Opportunity Cost

Pursuing hybrid execution first delays some higher-level workload work such as
k3s or GPU orchestration, but those features are less credible until the base
remote and SSH ownership model is first-class [SRC-02][SRC-04].

### Dependencies

- Provider-aware remote modeling and current hosted workflows [SRC-01]
- Hosted control-plane and node-agent ownership contract [SRC-02]
- Existing SSH seam in earlier planning artifacts [SRC-03]

### Alternatives Considered

- Focus only on hosted control-plane refinements and leave SSH for later.
  Rejected because the user explicitly wants first-class remote usage over SSH
  and the repo already models SSH as the intended seam [SRC-03][SRC-04].
- Add a separate remote-only CLI surface. Rejected because the current product
  direction has repeatedly preserved one canonical CLI and guest vocabulary
  across ownership modes [SRC-01][SRC-02].

---

*This PRD was seeded from bearing `VDcStPolu`. See `bearings/VDcStPolu/` for original research.*
