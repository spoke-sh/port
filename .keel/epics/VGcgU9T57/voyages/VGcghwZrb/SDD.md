# Seal Managed Hosted K3s Ownership - Software Design Description

> Keep hosted K3s lifecycle under explicit managed-service ownership and prove it across the observed worker-loss window.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage seals hosted K3s under explicit Port-managed service ownership. It
removes the legacy detached-process path, persists placement/service truth
durably, and defines a soak-oriented proof for the worker-loss window.

## Context & Boundaries

The voyage changes hosted runtime ownership and proof inside Port. It does not
change downstream `infra` logic directly, but it provides the runtime behavior
that downstream simplification depends on.

```
cluster up / reuse
        |
        v
Managed hosted K3s services
        |
        +--> durable placement/service records
        +--> service status surfaces
        \--> soak proof
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted runtime state roots | Runtime dependency | Persist placement and service ownership | current repo |
| Managed service supervisor | Runtime dependency | Own hosted K3s lifecycle and recovery | current repo |
| Soak proof tooling | Verification dependency | Record long-running worker stability evidence | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Service ownership | Managed-service path is canonical | Detached processes are the failure mode to eliminate |
| Runtime truth | Placement and service records persist durably | Service status cannot depend on transient launch memory |
| Stability proof | Record a soak-oriented proof slice | The incident pattern is time-based, not only launch-time |

## Architecture

Three logical components shape the voyage:

- `HostedPlacementPersistence`
- `HostedManagedK3sLifecycle`
- `HostedWorkerSoakProof`

## Components

- `HostedPlacementPersistence`
  Persists placement and service ownership across reuse and runtime restarts.
- `HostedManagedK3sLifecycle`
  Ensures hosted server and worker K3s processes exist only as managed
  services.
- `HostedWorkerSoakProof`
  Collects durable evidence over the failure window to prove worker stability
  or managed recovery.

## Interfaces

Planned interfaces:

- runtime-state files for hosted placement and managed services
- service status surfaces that reflect managed ownership
- proof artifacts tied to a soak-oriented verification path

## Data Flow

1. Launch or reuse a hosted cluster through the managed-service path.
2. Persist placement and service records durably.
3. Reject or replace legacy detached K3s artifacts if found.
4. Observe worker health over the target drift window.
5. Record reviewable proof of stability or recovery.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Placement or service records disappear after reuse | Validation against `SRS-01` | Treat the service as unhealthy and rehydrate or rebuild | Repair persistence and rerun hosted bring-up |
| Legacy detached K3s path appears | Runtime drift detection against `SRS-02` | Mark the runtime invalid and switch back to the managed path | Replace the cluster or service through managed lifecycle |
| Soak proof fails to demonstrate stability | Long-running verification against `SRS-03` | Treat the mission slice as incomplete | Fix lifecycle behavior and rerun the proof |
