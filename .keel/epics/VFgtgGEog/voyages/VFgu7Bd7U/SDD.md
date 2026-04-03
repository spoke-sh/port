# Guest Session Identity Contract - Software Design Description

> Define the stable guest-session identity and driver metadata contract that upstream creator systems can audit across hosted AWS PVM guest-backed shell flows.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes Port's guest-backed shell sessions legible to upstream creator systems
without changing the underlying hosted AWS PVM runtime contract. The design reuses the
verified `cloud-aws` substrate and existing guest protocol surfaces, then defines where
stable session identity and driver metadata should be attached so upstream systems can
audit one Port-owned shell driver.

## Context & Boundaries

This voyage covers:
- session identity and driver metadata surfaced by Port for hosted guest-backed shell flows
- canonical guest-backed `exec`, `pty`, and `forward` on `cloud-aws`
- explicit failure handling when that metadata cannot be provided

This voyage excludes:
- creator-domain auth, policy, tenancy, or audit-retention semantics
- alternate providers or a second creator-specific shell protocol

```
┌────────────────────────────────────────────────────┐
│        Session Identity And Audit Surface          │
│                                                    │
│  CLI/runtime  Hosted control plane  Node/guest     │
│  surfaces     session broker        transport      │
│                                                    │
└────────────────────────────────────────────────────┘
        ↑                               ↑
  Creator platform                Verified AWS PVM lane
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `VFgcPDfEj` hosted AWS PVM epic | Board/runtime contract | Provides the canonical `cloud-aws` substrate this voyage builds on | current repo state |
| Existing Port guest protocol | Internal protocol | Carries guest-backed shell operations without introducing a new protocol | current repo state |
| Hosted control plane and node agent | Runtime path | Brokers session identity from upstream request through runtime execution | current repo state |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime substrate | Reuse the verified hosted AWS PVM lane | Avoids reopening already-finished infrastructure work. |
| Metadata surface | Attach identity and driver metadata to canonical Port surfaces | Keeps upstream integration on the existing Port contract. |
| Failure behavior | Fail explicitly when metadata is missing or unsupported | Anonymous fallback would break auditing and mission honesty. |

## Architecture

The design spans three layers:

1. A Port-facing contract layer that defines the stable session identifier and driver
   metadata shape visible to upstream consumers.
2. A hosted runtime layer that preserves those values across control-plane and node-agent
   handoff for guest-backed shell flows.
3. A proof layer of docs and regression tests that demonstrates the contract and its
   failure behavior.

## Components

- CLI/runtime metadata surface: presents stable session identity and driver metadata on
  the existing guest-backed shell commands or status surfaces.
- Hosted control plane/session broker: carries the identity contract through authorization
  and scheduling without rewriting it into provider-specific labels.
- Node/guest execution path: preserves the session identity across `exec`, `pty`, and
  `forward` attachments for the same hosted session.
- Proof/docs/test surface: records the contract so downstream teams can integrate against it.

## Interfaces

No new protocol family is introduced. The voyage defines:
- the stable session identifier shape upstream systems can store and correlate
- the driver metadata fields that describe the Port shell driver
- the explicit errors returned when Port cannot honor that contract

## Data Flow

An upstream request enters Port through the canonical hosted guest surface, is bound to a
hosted session on the verified AWS PVM lane, then returns or streams results with the same
session identity and driver metadata attached. Follow-on `exec`, `pty`, or `forward`
operations reuse that identity instead of minting verb-specific audit keys.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Session identity cannot be resolved | Hosted request lacks the required session context | Return explicit contract error; do not emit ambiguous metadata | Fix runtime/session plumbing before retry |
| Driver metadata is incomplete | Validation detects unsupported or missing fields | Reject the contract surface with actionable guidance | Add the missing metadata definition |
| Runtime falls onto the wrong lane | Existing provider checks detect non-`cloud-aws` or unprepared state | Return provider-aware error with no fallback | Restore the AWS hosted PVM prerequisite path |
