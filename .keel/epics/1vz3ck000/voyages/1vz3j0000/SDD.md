# Substrate Drivers And Host Kits - Software Design Description

> Define and sequence the first implementation slices for substrate drivers, hosted node-agent runtime ownership, x86_64 PVM host kits, and AVF execution.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage does not attempt to ship every new execution lane immediately.
Instead, it creates the first implementation-ready architecture that can carry
them without breaking the existing local Linux workflow:

- isolate substrate-specific runtime behavior behind a driver boundary;
- define hosted lifecycle ownership above that boundary;
- treat x86_64 PVM as a host-kit and artifact-kit problem, not a runtime flag;
- treat AVF as a substrate-specific driver with the same CLI verbs and guest
  protocol semantics;
- decompose follow-on stories so execution can continue immediately after
  planning.

## Context & Boundaries

### In Scope

- substrate driver architecture and migration seams,
- hosted inventory/lifecycle contract over drivers,
- x86_64 PVM host-kit and artifact-kit contract,
- AVF execution and guest-transport contract,
- story decomposition and verification planning.

### Out of Scope

- fully shipping PVM runtime execution,
- fully shipping AVF execution,
- multi-user auth or full control-plane implementation,
- scheduler and host-group rollout.

```
┌────────────────────────────────────────────────────────────────┐
│                           port CLI                             │
│          machine / guest / artifacts / future hosted          │
└───────────────────────────────┬────────────────────────────────┘
                                │
                ┌───────────────┴────────────────┐
                │                                │
      ┌─────────▼─────────┐            ┌─────────▼─────────┐
      │ substrate driver   │            │ hosted lifecycle  │
      │ boundary           │            │ and inventory     │
      └───────┬────────────┘            └─────────┬─────────┘
              │                                   │
   ┌──────────▼──────────┐            ┌───────────▼──────────┐
   │ firecracker driver   │            │ node agent / control │
   │ avf driver (planned) │            │ plane contract       │
   └──────────┬──────────┘            └──────────────────────┘
              │
   ┌──────────▼──────────┐
   │ x86_64 PVM host kit │
   │ artifact variants   │
   └─────────────────────┘
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | workspace crate | shared substrate, artifact, and machine vocabulary | workspace current |
| `port-runtime` | workspace crate | source of current Firecracker-local launch and guest transport behavior | workspace current |
| `docs/hosted.md` | canonical doc contract | current hosted control split to extend rather than replace | repo current |
| x86_64 PVM host components | external runtime dependency | host-kernel and VMM requirements for future PVM lane | custom / out-of-tree |
| Apple Virtualization Framework | platform API | future macOS execution substrate | Apple platform API |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime abstraction | Introduce substrate driver traits and move Firecracker ownership behind them | Required to support AVF and hosted node agents without forking the CLI |
| Hosted layering | Put hosted lifecycle ownership above the driver boundary, not beside it | Hosted/local parity depends on shared lifecycle semantics |
| PVM framing | Treat x86_64 PVM as a host-kit + artifact-kit lane | The external evidence shows prepared host components are mandatory |
| arm64 PVM framing | Keep as research only | Current evidence does not justify implementation claims |
| AVF framing | Keep AVF first-class but separate from Firecracker transport assumptions | AVF is a real lane with different runtime primitives |

## Architecture

The intended architecture after this voyage:

1. `port-model`
   carries substrate, protection-mode, artifact, and machine-location terms.
2. `port-runtime`
   exposes substrate-driver traits for launch, inventory, stop, and guest
   attach.
3. Firecracker local execution
   becomes one concrete driver implementation.
4. hosted node agents
   become another implementation target over the same lifecycle boundary.
5. x86_64 PVM host kits and AVF drivers
   become explicit follow-on implementations rather than free-floating docs.

## Components

- substrate driver boundary:
  trait(s) or module boundary for machine lifecycle and guest attachment that
  local Firecracker, AVF, and hosted drivers can implement.
- hosted inventory/lifecycle model:
  shared types for local versus hosted ownership, inventory records, and
  machine status sources.
- x86_64 PVM host kit:
  planning contract for host kernel, VMM, and artifact variants plus validation
  expectations.
- AVF driver contract:
  planning contract for launch, transport, and operator workflow on macOS.
- CLI/docs:
  canonical verbs and help text that remain substrate-aware but not
  substrate-specific in naming.

## Interfaces

- CLI lifecycle interface:
  `machine launch|list|status|stop` remains canonical, but the underlying owner
  may be local runtime roots, node agents, or future substrate drivers.
- guest interface:
  current guest protocol remains canonical; only the transport or owner changes.
- artifact/host-kit interface:
  x86_64 PVM must declare prepared-host prerequisites and artifact variants
  explicitly rather than overloading standard Firecracker artifacts.
- AVF interface:
  launch and guest-attach mapping must translate the same lifecycle/guest verbs
  onto AVF runtime primitives.

## Data Flow

1. CLI resolves machine and artifact intent from the shared model.
2. Runtime selects a substrate driver rather than branching inline on
   Firecracker-only behavior.
3. Local Firecracker driver uses the current runtime-root and guest-vsock path.
4. Hosted driver later brokers the same lifecycle and guest verbs through node
   agents.
5. PVM host-kit and AVF lanes plug into that boundary with their own host
   preparation and launch behavior.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Driver extraction destabilizes local Firecracker lane | tests and CLI proofs fail | stop rollout and preserve current driver behavior | ship extraction incrementally behind clear interfaces |
| Hosted inventory contract diverges from local lifecycle semantics | doc/design review finds mismatched verbs or state model | reject the story | keep one lifecycle vocabulary and shared status model |
| PVM planning pretends standard artifacts are sufficient | host-kit contract lacks kernel/VMM prerequisites | fail review | require explicit host-kit and artifact-kit acceptance criteria |
| AVF planning collapses into Linux-only guidance | operator workflow or transport mapping is missing | fail review | require AVF-specific contract and macOS workflow evidence |
