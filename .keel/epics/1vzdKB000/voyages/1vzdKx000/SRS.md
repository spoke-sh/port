# Foundation And Hosted Cloud Hypervisor Lane - Software Requirements Specification

> Model, launch, and document the first Cloud Hypervisor standard lane through Port's canonical CLI, runtime driver boundary, and hosted control path.

**Epic:** [1vzdKB000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Extend the shared Port model, artifact selection, and doctor output
  so Cloud Hypervisor is an executable `standard` Linux substrate rather than a
  planned-only token.
- [SCOPE-02] Implement the local Cloud Hypervisor launch, status, and stop path
  through the existing `MachineDriver` seam.
- [SCOPE-03] Reuse the canonical guest protocol for Cloud Hypervisor guest
  exec, copy, pty, logs, and forward.
- [SCOPE-04] Route the hosted control-plane and node-agent lifecycle through
  the same Cloud Hypervisor lane without Firecracker-only assumptions.
- [SCOPE-05] Publish operator docs, help text, and proof commands for the
  shipped Cloud Hypervisor workflow.

### Out of Scope

- [SCOPE-06] Confidential or protected Cloud Hypervisor modes.
- [SCOPE-07] macOS-native Cloud Hypervisor execution.
- [SCOPE-08] scheduler-policy or auth redesign unrelated to Cloud Hypervisor
  routing.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Linux hosts can provide a `cloud-hypervisor` binary plus KVM access for the first runtime slice. | Dependency | Local and hosted execution would need a different substrate or a narrower support boundary. |
| Port's current kernel and guest-image artifact model can carry Cloud Hypervisor standard variants without introducing a second artifact vocabulary. | Assumption | Artifact commands and docs could become inconsistent across substrates. |
| The guest agent can remain substrate-agnostic if the host side maps Cloud Hypervisor guest transport onto the existing Port protocol. | Assumption | Port would risk inventing a second guest-control API. |
| Repo-local verification will run primarily on Linux `x86_64`; `aarch64` support may be modeled and tested without direct host execution in this voyage. | Constraint | Architecture claims must stay explicit in docs and diagnostics. |

## Constraints

- The voyage must preserve the single canonical Port CLI and model; no
  `cloud-hypervisor`-specific command tree is allowed.
- Unsupported substrate, protection-mode, or architecture combinations must
  fail fast with explicit detail and no fallback to Firecracker.
- Verification planning should prefer the repo's active techniques:
  `cargo test`, `llm-judge` where useful for doc/help review, and CLI proof or
  VHS-style recordings when operator evidence adds value.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must model Cloud Hypervisor as an executable `standard` substrate and resolve matching kernel and guest-image variants explicitly through `port doctor` and artifact selection. | SCOPE-01 | FR-01 | automated Rust tests + CLI proof |
| SRS-02 | Local machine launch, status, and stop must execute a Cloud Hypervisor machine through the `MachineDriver` seam with actionable host-preflight detail. | SCOPE-02 | FR-02 | automated Rust tests + CLI proof |
| SRS-03 | Cloud Hypervisor machines must expose guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical Port guest protocol and runtime metadata paths. | SCOPE-03 | FR-03 | automated Rust tests + CLI proof |
| SRS-04 | Hosted control-plane and node-agent flows must place, launch, inspect, and stop Cloud Hypervisor machines through the existing hosted contracts. | SCOPE-04 | FR-04 | automated Rust tests + hosted CLI proof |
| SRS-05 | README, help text, examples, and dedicated cloud/operator docs must publish one coherent local and hosted Cloud Hypervisor workflow with explicit proof commands. | SCOPE-05 | FR-05 | doc/help proof + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported or incomplete Cloud Hypervisor claims must fail fast with machine, substrate, architecture, and host detail; Port must not silently route them to Firecracker. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-01 | automated Rust tests + CLI proof |
| SRS-NFR-02 | Cloud Hypervisor must reuse the existing Port guest protocol and hosted route families instead of inventing a new substrate-specific guest API. | SCOPE-03, SCOPE-04 | NFR-02 | code inspection + automated Rust tests |
| SRS-NFR-03 | Every story in this voyage must carry executable verification through `cargo test` and story-level CLI/doc proof before acceptance. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-03 | story verify scripts + board evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
