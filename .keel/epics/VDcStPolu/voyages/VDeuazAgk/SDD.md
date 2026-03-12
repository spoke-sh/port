# SSH-First Hybrid Execution Foundations - Software Design Description

> Define and deliver the first SSH-first remote Linux lane with explicit ownership, remote readiness, lifecycle routing, and operator proof while preserving local and hosted execution semantics.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the hybrid-execution bearing into one bounded delivery slice.
It introduces SSH-managed remote Linux execution as a third ownership lane next
to the existing local runtime and hosted control-plane paths. The first slice
stays narrow on purpose: add the route contract, remote readiness surfaces, one
canonical machine lifecycle flow, and one human-reviewable operator proof.

## Context & Boundaries

### In Scope

- SSH-managed host connection modeling and route semantics
- remote doctor, help, and bootstrap guidance
- canonical `machine launch`, `status`, and `stop` for one SSH-managed Linux
  host
- docs and recording-backed proof for the first hybrid workflow

### Out of Scope

- guest-operation parity on the SSH lane
- scheduler or multi-node hosted placement work
- provider credential automation or network provisioning
- second command families or compatibility bridges

```
┌──────────────────────────────────────────────────────────────────┐
│             SSH-First Hybrid Execution Foundations              │
│                                                                  │
│  host model / route contract ─────┐                              │
│                                    ├──> machine lifecycle lane    │
│  doctor + bootstrap guidance ──────┤      (`launch|status|stop`) │
│                                    │                              │
│  docs + proof recording ───────────┘                              │
└──────────────────────────────────────────────────────────────────┘
          ↑                         ↑                        ↑
      local lane                hosted lane            remote SSH host
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/port-model` route and ownership enums | internal code | extend host connection and machine route vocabulary for the SSH lane | current workspace |
| `crates/port-cli` doctor and machine command surfaces | internal code | surface readiness and execute canonical lifecycle verbs | current workspace |
| existing hosted and cloud docs | internal docs | preserve one ownership model across local, hosted, and SSH lanes | current `docs/cloud.md` and `docs/hosted.md` |
| proof system with recording support | board workflow | capture a human-reviewable hybrid workflow artifact | current Keel verification toolchain |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First SSH slice scope | machine lifecycle only | keeps the initial hybrid slice bounded while making SSH a real product lane |
| Hybrid execution vocabulary | extend existing route and ownership tokens instead of creating a second remote-only CLI | preserves product coherence and follows the research recommendation |
| Readiness surface | make `port doctor` the canonical SSH preflight entrypoint | operators already expect doctor to explain whether a machine can run on a target host |
| Human proof | record at least one reviewable operator workflow artifact through the proof system | the user explicitly wants high-level artifacts that can be reviewed in terminal surfaces |

## Architecture

The voyage introduces four coordinated layers:

1. host connection and route contract
2. remote readiness and doctor surfaces
3. SSH-managed lifecycle execution path
4. docs and proof artifacts

## Components

### Host Connection And Route Contract

- Purpose: add SSH-managed remote ownership without disturbing the existing
  local and hosted lanes.
- Interface: `HostConnection`, route or ownership enums, config validation, and
  CLI-rendered route context.
- Behavior: model SSH as an explicit lane that participates in lifecycle and
  diagnostics rather than as prose-only guidance.

### Remote Readiness Surface

- Purpose: tell operators what must exist on the remote host before launch.
- Interface: `port doctor`, help text, config validation, and failure messages.
- Behavior: separate local prerequisites, hosted prerequisites, and SSH remote
  prerequisites clearly, including auth/bootstrap expectations.

### SSH Lifecycle Adapter

- Purpose: route canonical `machine launch`, `status`, and `stop` for one
  remote Linux host reachable over SSH.
- Interface: machine command path in the CLI and the runtime or transport layer
  it delegates to.
- Behavior: use the shared machine model while making route ownership explicit
  in success and failure surfaces.

### Hybrid Operator Proof Surface

- Purpose: publish the operator contract and a human-reviewable proof.
- Interface: docs plus a recording-backed proof artifact.
- Behavior: show one coherent workflow for local, hosted, and SSH-targeted
  machines without inventing separate terminology.

## Interfaces

- `hosts.<name>.connection.mode = "ssh"` with explicit connection fields added
  by implementation stories
- `port doctor`
- `port machine launch --machine <name>`
- `port machine status --machine <name>`
- `port machine stop --machine <name>`
- recording-backed proof command, likely via `vhs <tape>.tape`

## Data Flow

1. Operator targets a machine whose host declares SSH-managed remote ownership.
2. `port doctor` evaluates local client prerequisites plus remote-host
   readiness and auth/bootstrap expectations.
3. `port machine launch` resolves the machine route as SSH-managed rather than
   local or hosted.
4. The SSH lifecycle adapter realizes or verifies remote prerequisites, launches
   the machine, and writes or reads route-aware status context.
5. `port machine status` and `stop` reuse that same route contract.
6. Docs and the proof artifact show the resulting hybrid operator workflow.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| SSH host is selected but lacks required remote bootstrap or auth material | doctor check or launch preflight | fail fast with explicit SSH-lane guidance and the missing prerequisite | satisfy the remote requirement and rerun doctor or launch |
| CLI would otherwise fall back to local or hosted semantics for an SSH-targeted machine | route resolution or lifecycle test | reject the action and surface the resolved machine, host, and route context | fix the config or implementation so the requested lane stays explicit |
| SSH lifecycle path succeeds but status or stop cannot resolve the same ownership contract | automated regression or command proof | fail the command with route-aware lifecycle context | restore shared route resolution across lifecycle verbs |
| proof artifact drifts from the actual workflow | proof review or story verification | regenerate the recording from the canonical command path | keep proof commands tied to story verification |

## Story Decomposition

1. Route-contract story: extend the host connection and route or ownership model
   for the SSH lane.
2. Doctor/readiness story: surface SSH bootstrap, auth, and prerequisite
   guidance.
3. Lifecycle story: implement canonical `machine launch`, `status`, and `stop`
   for SSH-managed hosts.
4. Operator-proof story: publish docs and a recording-backed human review path.
