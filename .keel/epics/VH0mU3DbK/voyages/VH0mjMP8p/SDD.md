# Cluster Aggregate Wedge Field Threading - Software Design Description

> Thread the per-machine wedge fields onto HostedK3sMachineTruth so consumers polling port cluster status --format json see wedged_since, wedge_class, recovery_attempts, last_recovery_action, recovery_state, and guest_refresh_age_seconds on the cluster aggregate without needing per-machine port machine status calls.

**SRS:** [SRS.md](SRS.md)

## Overview

`HostedK3sClusterAccessReport` (returned by `port cluster status --format json`) already aggregates per-machine truth in `machines: Vec<HostedK3sMachineTruth>` and per-service truth in `managed_services`. The wedge/recovery fields shipped under mission `VGzwzdKvB` only appear on `MachineStatus` (per-machine endpoint), so consumers of the cluster aggregate cannot see them today.

The fix is to extend `HostedK3sMachineTruth` with the same six fields and populate them during cluster-status assembly. The existing CLI cluster-status path already does live HTTP introspection per machine for managed services (via `list_machine_services` inside `hosted_k3s_managed_service_truth`); this voyage adopts the same pattern for `machine_status`. That keeps the in-memory `wedge_state` map on the running control-plane as the single source of truth.

Population is best-effort: if a per-machine `machine_status` call fails (host unreachable, transport timeout), the new fields stay at their serde defaults. The cluster aggregate continues to surface that machine's row with `Unreachable` managed service, mirroring today's behavior.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI process                              │
│   port cluster status --format json                              │
│   └── hosted_k3s_cluster_access(config, runtime_root, name)      │
│        ├── hosted_k3s_managed_service_truth (existing N HTTP)    │
│        └── hosted_k3s_machine_truth (new — N HTTP for status)    │
└────────────────────────┬─────────────────────────────────────────┘
                         │  http://127.0.0.1:17040/v1/...
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Hosted control-plane process                    │
│   list_machines handler                                          │
│   └── annotate_machine_status_with_fleet_state                   │
│        ├── reads ControlPlaneStateInner.wedge_state              │
│        └── reads runtime/recovery/<machine>.json                 │
└─────────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `machine_status(config, runtime_root, machine_name)` | Existing in-tree function | Returns `MachineStatus` with all wedge fields populated for hosted machines via the live control-plane route | `crates/port-runtime/src/lib.rs:4619` |
| `RecoveryAttemptCounters`, `RecoveryActionRecord`, `RecoveryState` | Existing types | Already serde-ready and skip-empty on the wire | `crates/port-runtime/src/lib.rs:481-518` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Where to populate wedge fields on `HostedK3sMachineTruth` | Inside `hosted_k3s_machine_truth`, by calling `machine_status` per machine | Mirrors the existing pattern in `hosted_k3s_managed_service_truth` (per-machine `list_machine_services`), keeps the in-memory `wedge_state` on the live control plane as the single source of truth, avoids a new bulk-status HTTP route |
| Failure semantics for the per-machine call | Best-effort; on error, leave new fields at serde defaults | Cluster status must still produce a report when one machine is unreachable; the existing `managed_services` row already surfaces `Unreachable` for that machine, so the wedge fields being default is a consistent fallback rather than a separate failure mode |
| Threading `runtime_root` and `config` into `hosted_k3s_machine_truth` | Add both as new parameters, mirroring `hosted_k3s_managed_service_truth` | Internal callers already have both at the call site (`hosted_k3s_cluster_access`) |
| Breaking the wire format | Avoid via `#[serde(default, skip_serializing_if = ...)]` on every new field | Older infra pins must continue to decode the new payload; absent fields stay off the wire so older Port producers keep producing valid output |

## Architecture

Three thin layers touch this change:

1. **`HostedK3sMachineTruth`** (`crates/port-runtime/src/lib.rs:265-273`) — extend the struct with the six new optional fields, all `#[serde(default, skip_serializing_if = "...")]` consistent with the equivalent fields on `MachineStatus` (lines 460-477).
2. **`hosted_k3s_machine_truth`** (line 2748) — change signature to accept `config: &PortConfig` and `runtime_root: &Path`. For each `HostedK3sMachineAccess`, call `machine_status(config, runtime_root, &machine_name)`; if `Ok`, copy the six wedge/recovery fields onto the constructed `HostedK3sMachineTruth`; if `Err`, leave them at default. Update the single call site in `hosted_k3s_cluster_access` (line 3329) to pass `config` and `runtime_root`.
3. **`print_cluster_status_report`** (`crates/port-cli/src/lib.rs:1628`) — extend the per-machine render block to print the same lines that `print_machine_status` already prints for the wedge fields (`guest refresh age seconds:`, `wedged since:`, `wedge class:`).

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `HostedK3sMachineTruth` extended | Carries per-machine wedge/recovery state on the cluster aggregate | Six new optional fields, all serde-skip-empty |
| `hosted_k3s_machine_truth` extended | Populates the new fields per machine via a live `machine_status` call | New `(config, runtime_root)` parameters |
| `print_cluster_status_report` extended | Renders the new fields in the text output | New `writeln!` lines per machine block |

## Interfaces

### Wire shape

```json
{
  "machines": [
    {
      "role": "worker",
      "machine_name": "cloud-aws-worker-2",
      "node_name": "aws-linux-cell-1",
      "runtime_root": "/var/lib/port/aws-hosted/runtime/...",
      "detail": "...",

      "guest_refresh_age_seconds": 248,
      "wedged_since_unix_s": 1745000000,
      "wedge_class": "guest",
      "recovery_attempts": { "tier_1": 1, "tier_2": 0, "tier_3": 0 },
      "last_recovery_action": { "tier": 1, "timestamp_unix_s": 1745000060, "outcome": "restart-issued" },
      "recovery_state": "in-progress"
    }
  ]
}
```

All six new fields are absent from the JSON when unset. Decoders on prior Port pins ignore them; decoders on the new pin default `recovery_attempts` to all-zero, `recovery_state` to `Ok`, and the rest to `None`.

### Test strategy

- Unit test: serde round-trip of `HostedK3sMachineTruth` with the new fields populated and with all defaults. Assert wire shape matches the convention.
- Unit test: `hosted_k3s_machine_truth` returns the populated fields when `machine_status` succeeds (using a stub or test-only branch); returns defaults when `machine_status` fails.
- Render test: `print_cluster_status_report` emits the new lines exactly when `print_machine_status` does, and emits `(none)` for absent values.

## Risks

| Risk | Mitigation |
|------|------------|
| N additional HTTP calls per `port cluster status` invocation | The same N pattern already exists for `hosted_k3s_managed_service_truth`; this voyage doubles N, not multiplies it. A bulk HTTP route is a separate, future improvement. |
| Per-machine `machine_status` failure masks wedge state for that machine | Failure path leaves fields at default, mirroring the existing `Unreachable` managed-service signal — consumers see "no wedge info" rather than a hard error |
| Older consumers (infra on held pin) break on new wire fields | All six fields are `#[serde(default, skip_serializing_if = ...)]` — older decoders simply ignore them |
