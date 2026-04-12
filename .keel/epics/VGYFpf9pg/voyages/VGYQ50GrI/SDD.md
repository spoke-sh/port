# Prove Runtime Class Identity And Guard Rails - Software Design Description

> Deliver the promotion-runner execution contract as a clean-room runtime class
> that stays inspectably distinct from scratch authoring.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage builds on the shared runtime-class vocabulary from the builder epic
and turns `blessed-closure-promotion-runner` into a constrained clean-room
execution contract. The core design rule is separation: promotion execution
must never inherit scratch writable state, creator credentials, or hidden local
cache state.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────┐
│                    This Voyage                            │
│                                                            │
│  shared runtime class ──> promotion runner contract        │
│           │                       │                        │
│           │                       ├──> validation guard    │
│           │                       └──> proof surfaces      │
└────────────────────────────────────────────────────────────┘
          ↑                               ↑
   builder-epic vocabulary         infra publication substrate
```

### In Scope

- promotion-runner runtime-class semantics
- clean-room and declared-input validation
- machine-facing execution identity and proof fields

### Out of Scope

- cache publication and signing
- rollback state
- creator-facing approval flows

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| shared runtime-class model from builder epic | internal dependency | provide canonical naming and base metadata | current mission |
| `port-runtime` machine metadata and proof surfaces | internal runtime | carry promotion-runner identity to operators and downstream tooling | current workspace |
| `infra` publication/evidence planning contracts | external planning input | keep Port scoped to execution proof only | current adjacent repo planning docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Separation rule | Promotion runner uses a distinct runtime class, never a promoted scratch-builder instance. | The trust boundary depends on state and identity separation, not on operator discipline. |
| Input posture | Model declared immutable inputs explicitly in the runtime-class contract. | Downstream evidence needs to know what the clean room was allowed to see. |
| Proof surface | Reuse Port machine-facing proof and identity surfaces rather than inventing a publication-specific API. | Port should remain the runtime owner, not the publication control plane. |
| Scope guard | Stop at runtime-class semantics and proof surfaces. | Publication policy belongs downstream in `infra` and `spoke`. |

## Architecture

The voyage adds three pieces:

1. promotion-runner-specific runtime-class semantics in the shared model
2. validation that rejects collapsed scratch/promotion state
3. machine-facing proof fields that expose promotion-runner execution identity

## Components

### Promotion Runtime-Class Contract

- Purpose: represent clean-room promotion execution in Port.
- Interface: shared runtime-class metadata attached to machines.
- Behavior: describe the lane as declared-input-only, promotion-trusted, and
  distinct from scratch writable state.

### Guard-Rail Validation

- Purpose: prevent unsafe reuse of scratch state or creator credentials.
- Interface: config validation in `port-model`.
- Behavior: fail contradictory declarations before the runtime launches.

### Proof and Inspection Surfaces

- Purpose: let downstream tooling and operators link runtime evidence to the
  promotion lane.
- Interface: launch/status/monitor or adjacent machine metadata surfaces.
- Behavior: surface runtime class, clean-room posture, and trust-material
  expectations consistently.

## Interfaces

- Config/runtime-class metadata for `blessed-closure-promotion-runner`
- Validation errors when scratch and promotion state are conflated
- Machine-facing proof output that includes promotion-runner identity and
  posture

## Data Flow

1. Machine config declares the promotion-runner runtime class.
2. `port-model` validates the clean-room contract and rejects unsafe reuse.
3. `port-runtime` carries the runtime-class contract into execution metadata.
4. Operators or downstream tooling inspect the resulting proof surfaces.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Promotion runner tries to reuse scratch writable state | config validation | reject with explicit scratch/promotion conflict detail | declare separate clean-room state and rerun |
| Promotion runner omits declared-input posture | config validation or tests | fail the story; the contract is incomplete | add explicit input posture fields and validation |
| Machine proof surfaces hide promotion identity | CLI proof or automated tests | keep the voyage open | wire promotion metadata through runtime structs and rendering |
| Port changes start absorbing publication policy | planning or code review | stop and re-scope | keep policy state in downstream repos and narrow Port back to execution proof |
