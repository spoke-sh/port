# Attached Volume Contract Foundations - Software Design Description

> Define the canonical volume contract and the first direct-runtime
> attached-volume slice with proof-backed operator workflow.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns storage normalization into one bounded delivery slice. It
adds attached-volume semantics to the shared machine model, proves one
persistent file-backed data-volume workflow on the local Firecracker lane, and
publishes operator guidance and proof. Hosted and SSH-managed storage routing
stay out of scope for execution, but this slice must still fail fast and make
that boundary explicit.

## Context & Boundaries

### In Scope

- shared attached-volume model for machines
- one persistent `host-file` data volume on local Firecracker
- lifecycle output and guidance for direct-runtime storage ownership
- proof-backed docs for the first operator workflow

### Out of Scope

- hosted or SSH-managed attached-volume routing beyond explicit fail-fast guidance
- snapshot, clone, resize, multi-volume, or live attach/detach
- provider-native block disks or storage scheduling
- guest-side formatting and mount automation

```
┌──────────────────────────────────────────────────────────────────┐
│            Attached Volume Contract Foundations                 │
│                                                                  │
│  machine model + validation ─────┐                               │
│                                   ├──> local Firecracker launch   │
│  lifecycle/status surfaces ───────┤      with one data volume     │
│                                   │                               │
│  docs + recording proof ──────────┘                               │
└──────────────────────────────────────────────────────────────────┘
          ↑                          ↑
      boot artifacts            operator workflow
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/port-model` machine and artifact structs | internal code | add attached-volume contract while preserving boot-artifact semantics | current workspace |
| `crates/port-runtime` Firecracker config builder and local driver | internal code | translate one attached volume into an additional Firecracker drive | current workspace |
| `crates/port-cli` machine surfaces and docs references | internal code | keep operator output explicit and aligned with the shared model | current workspace |
| proof system with recording support | board workflow | capture a human-reviewable attached-volume workflow | current Keel verification toolchain |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First backend | persistent `host-file` only | smallest credible block-storage slice without faking cloud-wide storage support |
| First execution lane | local standard Firecracker only | keeps implementation bounded while the contract stays reusable for hosted and SSH follow-on work |
| Contract location | embed attached volumes in `MachineSpec` for the first slice | smallest viable seam; avoids introducing a full volume catalog prematurely |
| Operator surface | reuse `port machine launch|status|stop` | preserves one canonical CLI and keeps storage tied to machine lifecycle |
| Guest responsibility | no formatting or mount automation | block-storage semantics should stop at attachment in the first slice |

## Architecture

The voyage introduces four coordinated pieces:

1. shared machine-storage contract and validation
2. local Firecracker drive assembly for one attached data volume
3. lifecycle and failure surfaces that expose storage context
4. docs and proof artifacts for the first operator workflow

## Components

### Attached Volume Machine Contract

- Purpose: separate boot/rootfs concerns from operator-visible data-volume
  attachment.
- Interface: `MachineSpec`, config serialization, validation, and sample config
  docs.
- Behavior: accept one persistent `host-file` volume with explicit backend and
  path semantics while rejecting unsupported shapes.

### Local Firecracker Attached Volume Path

- Purpose: extend the local Firecracker lane with one additional drive.
- Interface: local machine launch config, runtime metadata, status, and stop
  surfaces.
- Behavior: preserve the rootfs drive and add one non-root data drive when the
  machine declares an attached volume.

### Storage Context Surfaces

- Purpose: make storage backend and ownership visible to operators.
- Interface: validation failures, lifecycle output, and docs.
- Behavior: surface machine name, backend token, host path, and owner detail
  instead of collapsing storage back into generic rootfs language, and fail
  fast when hosted or SSH-owned machines request the unsupported slice.

### Operator Proof Surface

- Purpose: publish a reviewable attached-volume workflow.
- Interface: docs plus a recording-backed proof artifact.
- Behavior: show one coherent local launch, status, and stop flow for a machine
  with an attached data volume.

## Interfaces

- `[[machines.<name>.volumes]]`
- `backend = "host-file"`
- `path = "<host-path>"`
- `persistence = "persistent"`
- `port machine launch --machine <name>`
- `port machine status --machine <name>`
- `port machine stop --machine <name>`
- recording-backed proof command, likely via a repo-local script or `vhs`

## Data Flow

1. Operator defines a machine with boot artifacts plus one attached `host-file`
   volume.
2. Config validation separates rootfs artifact checks from attached-volume
   contract checks.
3. `port machine launch` resolves the local Firecracker lane and builds a
   drive list containing the existing rootfs plus one data drive.
4. Runtime metadata and lifecycle output retain the volume backend and host
   path context.
5. `status` and `stop` reuse the same machine identity and storage context.
6. Docs and the proof artifact show the resulting attached-volume workflow.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Machine declares an unsupported backend or unsupported number of volumes | config validation or launch preflight | fail fast with explicit backend and machine detail | fix the machine config to the supported `host-file` contract |
| Declared host-file path is missing or unusable on the execution host | launch preflight or local path check | reject launch with explicit machine, path, and backend detail | create or fix the host path, then rerun launch |
| Local launch path would change rootfs-only behavior for machines without volumes | automated regression tests | fail the change before delivery | restore the current rootfs-only path for machines with no volumes |
| Proof artifact drifts from the actual attached-volume workflow | proof review or story verification | regenerate the recording from the canonical command path | keep proof commands tied to story verification |

## Story Decomposition

1. Model story: add the attached-volume contract to the shared machine model
   and validation.
2. Runtime story: attach one persistent `host-file` data volume in the local
   Firecracker launch path and preserve rootfs-only behavior.
3. Guidance story: explain prerequisites, backend semantics, local-lane
   boundaries, and hosted or SSH fail-fast behavior in docs and validation
   surfaces.
4. Proof story: publish a recording-backed operator workflow for review.
