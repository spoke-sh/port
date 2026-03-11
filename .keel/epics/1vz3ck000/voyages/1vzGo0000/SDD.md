# X86 64 PVM Host Kit Foundation - Software Design Description

> Define and begin implementing the x86_64 Firecracker/PVM host-kit, doctor, and artifact foundations for cost-controlled cloud execution.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage treats Firecracker/PVM as a host-kit plus artifact-kit foundation
problem instead of pretending it is a launch flag on top of the current
standard Linux lane.

The design adds three coordinated pieces:

1. a shared model contract for the x86_64 PVM host kit
2. runtime doctor checks that enforce that contract explicitly
3. artifact pipelines and docs that materialize `x86_64/firecracker/pvm`
   variants and explain the operator workflow

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                        This Voyage                           │
│                                                             │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────────┐ │
│  │  port-model  │   │ port-runtime │   │ artifact scripts│ │
│  │ host-kit +   │   │ doctor +     │   │ x86_64/fc/pvm   │ │
│  │ selector     │   │ diagnostics  │   │ build/validate  │ │
│  └──────────────┘   └──────────────┘   └─────────────────┘ │
│           │                  │                    │          │
│           └────────────┬─────┴────────────┬───────┘          │
│                        ▼                  ▼                  │
│                 CLI help + docs     repository-local proof   │
└─────────────────────────────────────────────────────────────┘
           ↑                                   ↑
   prepared x86_64 host kit              future PVM launch lane
```

### Out of Scope

- executing a real Firecracker/PVM launch
- arm64 protected virtualization claims
- hosted node placement based on PVM capability
- production distribution of the custom host kernel or patched VMM

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | Internal crate | Host-kit and artifact selector contract rendering | workspace |
| `port-runtime` | Internal crate | `doctor` checks and runtime-facing diagnostics | workspace |
| `scripts/artifacts/*` | Repository scripts | Kernel and guest-image build/validate pipelines | workspace |
| Linux `/proc/cmdline` and host architecture | Platform interface | Detect prepared x86_64 host-kit state | host OS |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First PVM implementation scope | x86_64 only | This is the only currently supportable Firecracker/PVM lane Port is willing to plan as implementation-backed. |
| Host-kit enforcement | Fail fast in doctor and launch-adjacent diagnostics | The PVM lane should not silently degrade into the standard lane. |
| Artifact identity | Reuse existing `architecture/substrate/protection_mode` selectors | Port already has the right canonical artifact vocabulary; the missing work is materializing and validating the PVM variants. |
| Operator proof | Repository-local demo and docs, not fake launch success | The foundation slice must be reproducible without overstating runtime readiness. |

## Architecture

The voyage touches four layers:

- `port-model`: host-kit contract and artifact compatibility rendering
- `port-runtime`: doctor checks and PVM-specific diagnostics
- artifact scripts/config: dedicated build and validate pipelines for
  `x86_64/firecracker/pvm`
- CLI/docs/evidence: operator-facing explanation and reproducible proof

## Components

### Model Contract

Purpose:

- represent the prepared x86_64 PVM host-kit requirement explicitly
- keep arm64 research-only in the rendered model and validation path

Behavior:

- names the PVM host-kit expectation separately from the standard Firecracker
  lane
- provides enough structure for doctor and future launch work to key off the
  same contract

### Runtime Doctor

Purpose:

- turn the PVM host-kit contract into executable diagnostics

Behavior:

- checks host architecture and platform
- inspects boot-line expectations such as `pti=off`
- verifies the expected PVM Firecracker binary contract
- explains why the lane is blocked instead of silently degrading

### Artifact Pipelines

Purpose:

- materialize the PVM-specific kernel and guest-image variants

Behavior:

- builds and validates `x86_64/firecracker/pvm` explicitly
- rejects fallback to `standard`
- keeps output paths deterministic under the existing artifact selector model

### Operator Surface

Purpose:

- keep the CLI and docs honest about what ships today

Behavior:

- `port doctor` exposes the new host-kit checks
- help text and docs explain the x86_64 keep and arm64 research-only boundary
- evidence scripts prove the documented workflow is reproducible

## Interfaces

CLI and docs:

- `port doctor`
- `port artifacts build --artifact <name> --architecture x86_64 --substrate firecracker --protection-mode pvm`
- `port artifacts validate --artifact <name> --architecture x86_64 --substrate firecracker --protection-mode pvm`

Model/runtime contract additions:

- explicit PVM host-kit description attached to the relevant Linux host lane
- dedicated PVM diagnostics surfaced through doctor output

## Data Flow

1. The operator selects the `x86_64/firecracker/pvm` lane in the model or CLI.
2. `port doctor` evaluates the prepared host-kit contract and prints explicit
   readiness or blocking diagnostics.
3. Artifact commands resolve the same selector and invoke the dedicated PVM
   build/validate scripts.
4. Docs and proof scripts demonstrate the same lane with no hidden fallback.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Host is not Linux x86_64 | Doctor reads host architecture/platform | Report the lane as unsupported for the current host | Move to a supported x86_64 Linux host or choose a different lane |
| Boot line lacks `pti=off` | Doctor inspects `/proc/cmdline` | Fail the PVM host-kit check with an explicit message | Reboot into the prepared PVM host kit |
| PVM Firecracker binary contract is absent | Doctor cannot resolve the configured/expected PVM binary | Fail fast with the expected binary guidance | Install or build the patched PVM Firecracker binary |
| PVM artifact variant is missing | Artifact selector cannot resolve `x86_64/firecracker/pvm` | Fail the artifact command immediately | Build or publish the missing PVM variant |
| Operator targets arm64 PVM | Model or doctor resolves `aarch64/firecracker/pvm` | Explain that the lane remains research-only | Use x86_64 PVM or a different supported lane |
