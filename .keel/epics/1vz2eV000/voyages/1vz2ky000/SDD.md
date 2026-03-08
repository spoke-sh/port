# Hosted Control And Substrate Foundations - Software Design Description

> Define the substrate-aware model, hosted control-plane contract, and first machine lifecycle plus artifact mobility slices for Port's expansion beyond the local Firecracker MVP.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage establishes the first shared foundation for Port's expansion beyond
its local Firecracker MVP:

- evolve the model from "provider + local Firecracker" into "substrate +
  protection mode + provider + artifact reference";
- add local machine inventory and lifecycle commands that can later point at a
  hosted daemon instead of only at local runtime directories;
- define the first hosted Port control-plane split so the CLI can become a
  client rather than the only process with runtime ownership; and
- make artifact handling honest about remote distribution and variant selection.

## Context & Boundaries

In scope:

- model and CLI/runtime changes for substrate-aware planning;
- local runtime lifecycle inspection and stop behavior;
- initial hosted-control service and client contract;
- artifact mobility contracts and docs;
- support-matrix/help/documentation updates.

Out of scope:

- shipping a complete PVM runtime;
- shipping full AVF execution;
- full scheduler/host-group and auth implementation;
- production-ready remote artifact backends.

```
┌────────────────────────────────────────────────────────────────┐
│                           port CLI                             │
│ machine list/status/stop + artifact + guest + future client   │
└───────────────────────────────┬────────────────────────────────┘
                                │
                ┌───────────────┴────────────────┐
                │                                │
      ┌─────────▼─────────┐            ┌─────────▼─────────┐
      │   local runtime   │            │ hosted control /  │
      │  manifests + pids │            │ daemon API layer  │
      └─────────┬─────────┘            └─────────┬─────────┘
                │                                │
        ┌───────▼────────┐               ┌───────▼────────┐
        │ substrate-aware │               │ artifact refs / │
        │ machine model   │               │ variant policy  │
        └─────────────────┘               └─────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | workspace crate | shared host, machine, and artifact schema evolution | workspace current |
| `port-runtime` | workspace crate | runtime manifests, PID inspection, and lifecycle actions | workspace current |
| `port-cli` | workspace crate | canonical operator surface and help system | workspace current |
| existing guest protocol | workspace contract | preserve guest-operation semantics across local and hosted paths | workspace current |
| local runtime root manifests | local runtime interface | bootstrap machine list/status/stop without adding a datastore first | current runtime layout |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Capability axis | Introduce substrate and protection-mode concepts alongside provider identity | Provider alone no longer explains Port's actual runtime choices |
| Lifecycle source of truth | Use runtime manifests plus live PID inspection for the first `list/status/stop` slice | Fastest path to a real operator surface without waiting for a daemon |
| Hosted-control shape | Add a long-lived control service contract and treat the CLI as a future client of that service | Hosted Port cannot rely on one short-lived CLI process owning all state |
| Artifact evolution | Extend artifact references toward publish/pull/cache semantics and variant metadata before binding to one remote backend | Keeps the operator contract stable while backend choices remain open |
| Substrate rollout | Treat Firecracker KVM as proven, Firecracker PVM as an explicit lane, and AVF/Cloud Hypervisor as first-class planned substrates | Honest scope boundaries prevent fake parity claims |

## Architecture

The voyage introduces four connected layers:

1. `port-model`
   grows substrate and artifact-reference vocabulary.
2. `port-runtime`
   gains machine inventory/status/stop logic over runtime manifests and local
   processes.
3. a new hosted-control layer
   defines daemon/API/client seams for later remote operation.
4. CLI/docs
   present the expanded lifecycle and support matrix through canonical commands.

## Components

- `port-model`
  adds substrate-aware machine and host fields plus artifact reference or
  variant metadata.
- `port-runtime`
  enumerates runtime directories, loads manifests, inspects PID liveness, and
  performs stop operations by signaling local runtime owners.
- `port-cli`
  adds machine lifecycle verbs and help/examples that describe local versus
  hosted behavior consistently.
- hosted-control contract
  introduces the first shared service/client API types or crate boundaries so
  Port can move from direct local orchestration toward a daemon-backed model.
- docs
  publish the new support matrix, artifact mobility contract, and honest lane
  boundaries for KVM, PVM, Cloud Hypervisor, and AVF.

## Interfaces

- CLI lifecycle:
  `port machine list`, `port machine status --machine <name>`, and
  `port machine stop --machine <name>` become canonical lifecycle surfaces.
- Runtime inspection:
  runtime manifests remain file-backed for the first slice; status resolves from
  manifest presence plus live process inspection.
- Hosted control:
  the voyage defines a service/client contract for machine lifecycle and guest
  transport brokerage, even if only partially scaffolded in this slice.
- Artifact contract:
  artifact identifiers evolve from local paths toward logical refs with local
  build outputs plus later publish/pull/cache backends.

## Data Flow

1. CLI loads the model and resolves substrate-aware machine metadata.
2. For local lifecycle commands, runtime enumerates runtime roots, reads
   manifests, inspects PID/process state, and renders status.
3. For stop, runtime signals the local owner process and updates runtime
   inspection output deterministically.
4. For hosted-control paths, the CLI will eventually target the daemon/client
   contract instead of direct local ownership, but the command verbs remain the
   same.
5. Artifact selection resolves by architecture, substrate, and protection mode
   before any backend-specific fetch logic runs.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Runtime manifest exists but PID is dead | PID inspection or missing process | report stopped/stale state explicitly | allow operator cleanup or relaunch |
| `stop` targets an unknown or already-stopped machine | manifest lookup and liveness check | return a precise lifecycle error without pretending success | rerun against a valid machine or relaunch |
| model requests an unsupported substrate/protection combination | config validation or CLI/runtime preflight | fail fast with operator guidance | choose a supported lane or wait for implementation |
| artifact ref cannot resolve a variant for the requested lane | variant selection step | return a deterministic resolution error | publish/build the missing variant or choose another lane |
