# Streamed Guest Sessions And Hosted Transfer - Product Requirements

> Port now has real local Linux, prepared-node PVM, and AVF runtime lanes, but
> it still falls short of the hosted-product and Slicer-class operator story in
> one high-value area: streamed guest control. Operators cannot open a truly
> interactive shell, follow logs as they arrive, or move bytes through hosted
> copy and forward paths without bootstrap assumptions that leak node-local
> details.

## Problem Statement

The current guest control surface is split across two levels of maturity:

- local runtimes can execute the canonical `guest` verbs, but `pty` and
  `logs --follow` still behave like completed transcripts instead of streamed
  interactive sessions
- hosted `guest copy` still assumes the referenced host paths are visible on
  the node host
- hosted `guest forward` still depends on repo-local listener ownership instead
  of a real hosted transport path

That leaves a major gap versus the intended hosted Port product and versus the
operator expectations set by comparable systems such as SlicerVM.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ship streamed guest shell and log-follow workflows through the canonical Port CLI and SDK | Interactive PTY and `logs --follow` proof on the shared guest protocol | First streamed guest-session rollout |
| GOAL-02 | Remove bootstrap hosted transport assumptions from copy and forward | Hosted copy and forward work without node-visible host-path or repo-local listener assumptions | First real hosted transport rollout |
| GOAL-03 | Preserve one operator model while streaming lands | No new substrate- or hosted-only guest command family | Ongoing |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Hosted Operator | Runs customer or team workloads through Port's hosted control plane | Real remote shell, logs, copy, and forward workflows without leaking node-local assumptions |
| CLI Operator | Uses `port` directly for debugging and day-to-day VM control | Interactive guest control through the existing command model |
| SDK Integrator | Builds higher-level automation against `port-sdk` and hosted APIs | Stable streamed contracts that map cleanly to machine and guest ownership |

## Scope

### In Scope

- [SCOPE-01] Streamed PTY and log-follow contracts across the guest protocol,
  CLI, and SDK.
- [SCOPE-02] Real hosted copy and forward transport through the control-plane
  and node-agent ownership model.
- [SCOPE-03] Operator-facing docs, help, and recorded evidence for the streamed
  guest workflows.

### Out of Scope

- [SCOPE-04] Scheduler policy, durable node registration, or host-group
  selection changes.
- [SCOPE-05] Real hosted service execution and teardown beyond guest transport.
- [SCOPE-06] New hypervisor programs such as Cloud Hypervisor delivery.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must support streamed guest shell and log-follow workflows through the canonical CLI, shared guest protocol, and SDK. | GOAL-01, GOAL-03 | must | Interactive sessions are a major remaining operator gap. |
| FR-02 | Hosted guest copy and forward must use real streamed transport through Port's hosted ownership model instead of node-host path assumptions or repo-local listener ownership. | GOAL-02, GOAL-03 | must | Hosted product credibility depends on this transport becoming real. |
| FR-03 | Help text, docs, and proofs must keep streamed guest behavior discoverable and explicit without inventing a second guest command family. | GOAL-03 | must | The CLI is a first-class product surface. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Streamed guest sessions and transport must have deterministic attach, EOF, exit, and cleanup behavior. | GOAL-01, GOAL-02, GOAL-03 | must | Streaming failures are hard to debug without explicit lifecycle rules. |
| NFR-02 | Existing local Linux, hosted PVM, and AVF lanes must not silently fall back or regress while streamed transport lands. | GOAL-03 | must | Port cannot trade one lane's clarity for another's completeness. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove streamed session behavior with story-level tests and CLI proofs for PTY,
  log-follow, hosted copy, and hosted forward.
- Validate documentation and help surfaces through command proofs and recorded
  operator evidence.
- Use the existing Rust unit-test and CLI-proof stack as the primary automated
  gate.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing guest protocol can absorb streaming control messages without forcing a second guest API. | The epic would need a broader protocol split and CLI redesign. | Validate in the first contract story. |
| Hosted control-plane and node-agent ownership can relay streamed guest transport without redesigning the current control split. | Hosted transport work would expand into a broader daemon or scheduler rewrite. | Validate during the transport implementation stories. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether hosted streaming should proxy through the control plane or resolve into direct node-agent attach for long-lived sessions | Runtime/SDK | Open |
| How much terminal raw-mode support belongs in the first streamed PTY slice versus a later ergonomics pass | CLI/Runtime | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Interactive `port guest pty` and `port guest logs --follow` proofs exist through the canonical command model.
- [ ] Hosted `port guest copy` and `port guest forward` no longer depend on node-visible host paths or repo-local listener ownership.
- [ ] CLI help, docs, and SDK surface the streamed guest control contract coherently.
<!-- END SUCCESS_CRITERIA -->
