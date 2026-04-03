# Upstream Shell Driver Contract - Software Design Description

> Define the canonical upstream integration contract for guest-backed exec, pty, and forward on hosted AWS PVM without introducing a second shell protocol.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines the contract upstream control planes should rely on when they use
Port as a guest-backed shell driver on hosted AWS PVM. The design keeps `exec`, `pty`,
and `forward` on the current Port verbs and guest protocol while making lifecycle,
streaming, and failure behavior explicit enough for creator-platform integration.

## Context & Boundaries

This voyage covers:
- upstream expectations for guest-backed `exec`, `pty`, and `forward`
- canonical lifecycle semantics for command-style and streamed shell operations
- provider-aware failure behavior on the hosted AWS PVM lane

This voyage excludes:
- creator-domain policy or UX
- any second shell protocol or non-AWS first rollout

```
┌────────────────────────────────────────────────────┐
│      Upstream Shell Driver Integration Contract    │
│                                                    │
│  Upstream control plane  Port verbs/protocol       │
│  lifecycle + audit       hosted runtime path       │
│                                                    │
└────────────────────────────────────────────────────┘
          ↑                            ↑
   Creator platform             AWS hosted PVM lane
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `VFgcPDfEj` hosted AWS PVM epic | Board/runtime contract | Supplies the provider-backed runtime lane this contract depends on | current repo state |
| Existing Port CLI verbs | Interface surface | Keeps upstream integration on canonical `guest exec|pty|forward` verbs | current repo state |
| Guest protocol and hosted transport | Runtime path | Carries byte streams and lifecycle events across hosted execution | current repo state |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Integration surface | Reuse canonical Port verbs and guest protocol | Avoids a second shell protocol. |
| Contract scope | Define lifecycle and failure semantics explicitly | Upstream control planes need stable, reviewable expectations. |
| Failure behavior | Preserve provider-aware explicit errors with no fallback | Honest failures are part of the mission contract. |

## Architecture

The voyage treats the shell-driver contract as a thin layer over existing runtime pieces:

1. Upstream control planes call the canonical Port guest verbs.
2. Port routes those requests through the hosted control-plane and node-agent path on
   the verified AWS PVM lane.
3. Docs, tests, and proof artifacts define what behavior upstream systems may depend on
   for shell lifecycle, streaming, and failure handling.

## Components

- Upstream contract surface: the documented guarantees for `exec`, `pty`, and `forward`.
- Port CLI/runtime: continues to expose the canonical verbs and marshals guest protocol traffic.
- Hosted runtime path: the control-plane and node-agent components that keep hosted behavior aligned with local semantics.
- Proof/test layer: captures integration behavior and provider-aware failures in artifacts upstream teams can review.

## Interfaces

The voyage does not add a new API family. It standardizes:
- which existing Port verbs represent the shell-driver surface
- how command-style exec differs from streamed `pty` and `forward`
- how provider/runtime errors are surfaced to upstream callers

## Data Flow

An upstream control plane invokes the canonical Port guest verb, Port resolves it onto the
hosted AWS PVM runtime path, and the guest protocol carries the resulting command or stream.
The contract clarifies which lifecycle events and byte-stream behaviors the caller may rely
on, and which failures indicate provider/runtime contract breakage instead of shell misuse.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Wrong hosted lane is selected | Provider/runtime validation rejects non-`cloud-aws` mapping | Return actionable provider-aware error | Restore canonical AWS hosted routing |
| Host kit or artifacts are missing | Hosted prerequisites fail readiness checks | Stop before attach/stream and explain the missing dependency | Prepare the node or artifact set, then retry |
| Upstream assumes unsupported lifecycle semantics | Contract verification or docs review reveals unsupported expectation | Reject the unsupported contract shape instead of approximating it | Update the integration or explicitly extend the contract in follow-on planning |
