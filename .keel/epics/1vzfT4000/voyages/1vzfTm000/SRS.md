# Service Policy And Secret Runtime Foundations - Software Requirements Specification

> Define and ship the first restart-policy, health-state, and hardened secret-materialization slice through the canonical port service workflow.

**Epic:** [1vzfT4000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

In scope:
- [SCOPE-01] Add shared restart-policy and health-policy fields for Port
  services and sandboxes across the model, runtime, CLI, hosted API, and SDK.
- [SCOPE-02] Implement runtime-owned supervision and status reporting for
  managed service processes, including restart and health state.
- [SCOPE-03] Replace plaintext runtime JSON as the canonical service-execution
  secret input with a stronger backend plus explicit materialization.
- [SCOPE-04] Publish a repo-local operator workflow and evidence for the new
  service reliability slice.

Out of scope:
- [SCOPE-05] External secret-manager integrations, KMS, or tenant-aware auth.
- [SCOPE-06] Autoscaling, preemption, richer fleet scheduling, or service
  orchestration beyond the current runtime owner.
- [SCOPE-07] A second hosted-only service API or a second secret-management
  command family.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Existing `port service apply|list|status|stop` flows already resolve one runtime owner for local and hosted machines. | dependency | A separate runtime model would be required before policy or health can stay coherent. |
| The hosted control plane and node-agent path can carry richer service status without inventing a new route family. | assumption | The voyage would need broader API surgery than planned. |
| Repo-local service proofs can launch a long-running demo process and observe restart plus health transitions deterministically. | assumption | Verification would become flaky and the operator workflow would be less credible. |
| Rust unit/integration tests remain the primary automated verification path, with CLI proof scripts and optional VHS recordings for operator evidence. | dependency | Story verification planning would need to change before decomposition. |

## Constraints

- Keep `port service` as the single canonical surface for services and
  sandboxes across local and hosted lanes.
- Replace, do not bridge: do not preserve legacy plaintext JSON secret values
  as the canonical execution path once the stronger backend ships.
- Keep runtime ownership explicit and inspectable in `service status`,
  including where restart, health, and secret state originate.
- Stay within repo-local operator proofs for the first slice; external secret
  managers and hosted control-plane hard multi-tenancy are out of scope.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define restart-policy and health-policy contracts in the shared model for `port service apply` when launching either a service or a sandbox, and those contracts must surface through CLI help, hosted API payloads, and SDK request/status types. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-02 | Port must implement runtime-owned managed-process supervision that records process state, restart count, last exit detail, and health state, and that enforces the selected restart policy without introducing a second service runtime family. | SCOPE-02 | FR-02 | automated test + CLI proof |
| SRS-03 | Port must replace plaintext runtime JSON secret values as the canonical service-execution input with a stronger secret backend plus materialization contract that `port service secret` and `port service apply` both use. | SCOPE-03 | FR-03 | automated test + CLI proof |
| SRS-04 | Port must publish a repo-local operator workflow that proves a service can consume a secret, report health, restart according to policy, and stop through the canonical `port service` surface for at least one local or hosted path. | SCOPE-04 | FR-05 | CLI proof + docs review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Restart, health, and secret state must remain deterministic and attributable to one runtime owner so repeated `port service status` reads return stable policy and state provenance. | SCOPE-02, SCOPE-03 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | Unsupported policy values, runtime combinations, or secret-materialization requests must fail fast with explicit diagnostics and no fallback to legacy JSON-secret execution behavior. | SCOPE-01, SCOPE-03 | NFR-02 | automated test + CLI proof |
| SRS-NFR-03 | Story verification for this voyage must use the repository's detected techniques: Rust tests for behavior, CLI proof scripts for operator evidence, and optional VHS recording only where it materially improves discoverability. | SCOPE-04 | NFR-03 | planning review + story verification annotations |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
