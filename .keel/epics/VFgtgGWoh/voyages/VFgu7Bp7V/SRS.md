# Upstream Shell Driver Contract - SRS

## Summary

Epic: VFgtgGWoh
Goal: Define the canonical upstream integration contract for guest-backed exec, pty, and forward on hosted AWS PVM without introducing a second shell protocol.

## Scope

### In Scope

- [SCOPE-01] Canonical upstream shell-driver contract for hosted guest-backed `exec`, `pty`, and `forward`.
- [SCOPE-02] Lifecycle and streaming expectations higher-level control planes can rely on.
- [SCOPE-03] Explicit provider-aware failure guidance for wrong lane or missing hosted prerequisites.

### Out of Scope

- [SCOPE-04] Creator-specific auth, tenancy, or product UX behavior.
- [SCOPE-05] A replacement shell protocol or non-AWS first rollout.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must define one canonical upstream shell-driver contract for hosted guest-backed `exec`, `pty`, and `forward`. | SCOPE-01 | FR-01 | automated |
| SRS-02 | The contract must preserve the existing Port guest protocol frames and verb model rather than introducing a second shell protocol. | SCOPE-01, SCOPE-02 | FR-02 | automated |
| SRS-03 | The contract must document and verify lifecycle expectations for command-style exec and streamed `pty` or `forward` behavior. | SCOPE-02 | FR-03 | manual |
| SRS-04 | Wrong lane, missing host kit, missing artifacts, or unsupported hosted prerequisites must fail with actionable guidance and no silent fallback. | SCOPE-03 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The contract must remain consumable through canonical Port CLI/runtime surfaces so local and hosted behavior stay comparable. | SCOPE-01, SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Verification must cover both successful shell-driver flows and explicit provider-aware failure surfaces. | SCOPE-02, SCOPE-03 | NFR-02 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
