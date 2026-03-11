# Hosted Detached Forward Lifecycle - Software Requirements Specification

> Finish hosted guest forward with node-owned list, stop, name, and detached lifecycle semantics through the control plane and node agent.

**Epic:** [1vzETR000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Shared hosted control-plane and node-agent contracts for detached
  forward start, list, stop, and named session semantics.
- [SCOPE-02] Node-owned detached forward manifests and lifecycle actions under
  the resolved runtime root for hosted machines.
- [SCOPE-03] Canonical CLI and SDK routing for hosted
  `port guest forward --lifecycle detached`, `--list`, `--stop`, and `--name`.
- [SCOPE-04] Help text, docs, and proof for the hosted detached forward
  operator workflow.

### Out of Scope

- [SCOPE-07] Multi-tenant auth, RBAC, or hosted billing changes beyond the
  existing single-node bearer-token lane.
- [SCOPE-08] Multi-node scheduling or host-group policy changes beyond the
  current hosted machine-to-node resolution contract.
- [SCOPE-09] PVM host-kit implementation changes required outside the current
  hosted runtime-owner boundary.
- [SCOPE-10] AVF execution work beyond preserving the existing operator model.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The shipped hosted control-plane and node-agent demo lane remains the canonical remote runtime path for hosted machines. | Dependency | Detached lifecycle work would have no live transport to attach to. |
| Local detached forward manifests under the runtime root remain the canonical source of forward session truth. | Assumption | The voyage would need a broader state-model redesign instead of a transport-focused extension. |
| Hosted machines already resolve to one runtime-owning node before guest transport begins. | Dependency | Detached list and stop operations would need new scheduling logic, which is out of scope here. |
| Existing CLI and SDK `guest forward` command/request shapes remain the canonical product surface. | Assumption | The voyage would fragment the operator model rather than extending it. |

## Constraints

- Keep one canonical `guest forward` command family across local and hosted
  execution.
- Keep detached forward state node-owned; the control plane may route it, but
  must not become a second runtime state store.
- Fail fast when hosted list or stop cannot resolve a machine, node, or
  detached forward name.
- Preserve the existing foreground hosted start path while adding detached
  lifecycle behavior.
- Keep the first hosted detached lifecycle lane repository-local and
  reproducible for verification.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define shared hosted request and response contracts for detached guest forward start, list, and stop operations, including named session identity and route context. | SCOPE-01, SCOPE-03 | FR-04 | automated test + inspection |
| SRS-02 | The node agent must own detached forward manifests and lifecycle actions for hosted machines so list and stop operate on node runtime state instead of repo-local CLI state. | SCOPE-01, SCOPE-02 | FR-04 | automated test + CLI demo |
| SRS-03 | Hosted `port guest forward --lifecycle detached`, `--list`, `--stop`, and `--name` must execute through the live control-plane and node-agent path while preserving the canonical command family. | SCOPE-01, SCOPE-03 | FR-04 | automated test + CLI demo |
| SRS-04 | Port must publish help text, docs, and workflow evidence for hosted detached forward lifecycle management. | SCOPE-04 | FR-05 | manual review + CLI demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted detached forward failures must include enough machine, node, runtime-root, and forward-name context for operators to identify the broken runtime owner or route. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated test + inspection |
| SRS-NFR-02 | The detached forward contract must preserve the current runtime-root manifest model so future multi-node and scheduler work can extend it without reworking the operator surface again. | SCOPE-01, SCOPE-02 | NFR-04 | design review + docs inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
