# Expose Hosted Cluster Status Schema - Software Design Description

> Publish one canonical hosted-cluster status contract for downstream rollout and inspection consumers.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines one canonical hosted-cluster status contract for Port. It
turns the current mix of placement files, service-status commands, and legacy
runtime artifacts into one explicit downstream surface that other repos can
consume safely.

## Context & Boundaries

The voyage changes only the contract surface for hosted status and the
supporting authored proof. It does not yet enforce managed-service ownership;
that remains the adjacent voyage.

```
hosted runtime state
        |
        v
HostedStatusAssembler
        |
        v
port cluster status
        |
        +--> local operators
        \--> downstream infra
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted runtime state files | Runtime dependency | Supply placement and managed-service truth | current repo |
| `port cluster status` seam | CLI dependency | Carries the canonical hosted status payload | current repo |
| Paired infra mission `VGcfT59ur` | Cross-repo dependency | Consumes the downstream contract | planned |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Status authority | One canonical hosted payload | Prevents downstream inference drift |
| Exposure path | Existing cluster status seam | Avoids hidden diagnostic-only contracts |
| Drift reporting | Include legacy detached-runtime truth explicitly | Downstream consumers must distinguish invalid runtime shape |

## Architecture

Three logical components shape the voyage:

- `HostedStatusAssembler`
- `HostedStatusSurface`
- `HostedStatusContractDocs`

## Components

- `HostedStatusAssembler`
  Collects machine, placement, service, and legacy-runtime facts into one
  typed payload.
- `HostedStatusSurface`
  Projects that payload through `port cluster status`.
- `HostedStatusContractDocs`
  Defines the fields, semantics, and proof posture expected by downstream
  consumers in [CONTRACT.md](CONTRACT.md).

## Interfaces

Planned interfaces:

- canonical hosted status JSON within existing cluster status output
- typed internal representation for hosted machine/service truth
- authored contract docs for downstream consumers in [CONTRACT.md](CONTRACT.md)

## Data Flow

1. Collect hosted machine, placement, and service facts from runtime state.
2. Detect legacy detached-runtime drift.
3. Assemble the canonical hosted status payload.
4. Emit the payload through `port cluster status`.
5. Validate the documented contract against real command output.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted placement or service facts are missing | Assembler validation against `SRS-01` | Mark the hosted status incomplete or degraded explicitly | Repair the upstream runtime state source |
| Legacy detached-runtime artifacts exist alongside managed state | Drift detection against `SRS-01` | Surface explicit legacy-runtime drift in status | Replace with the managed-service path |
| Contract docs diverge from the emitted payload | Proof or tests against `SRS-03` | Fail the story or verification | Update docs or payload before release |
