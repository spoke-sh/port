# Live Detector And Recovery Runner Wiring - Software Design Description

> Spawn the wedge detector tick and recovery runner from serve_control_plane so the existing pure-function library code becomes an active runtime contract: wedge_state populates against live heartbeat ages, decide_recovery_action drives tier-1 guest restart and tier-2 overlay recreate against the runtime root, and tier-3 emits an awaiting-host-recycle signal. Recovery actions stay opt-in via ClusterRecoveryConfig.enabled.

**SRS:** [SRS.md](SRS.md)

## Overview

Mission `VGzwzdKvB` shipped two pure decision functions — `run_wedge_detector_tick` and `decide_recovery_action` — plus all the supporting types (`ClusterDetectionConfig`, `ClusterRecoveryConfig`, `WedgeFact`, `RecoveryAction`, `PersistedRecoveryRecord`, `RecoveryEventSink`, `try_acquire_recovery_lock`). They live as `#[allow(dead_code)]` library code with full unit coverage. `serve_control_plane` does not invoke either function in the live process, so `wedge_state` never updates from runtime data and no recovery actions ever fire.

This voyage wires both functions into `serve_control_plane` as periodic background workers, mirroring the existing `spawn_guest_heartbeat_probe_loop` pattern that the node-agent already uses for the heartbeat probe. Two separate workers — one for detection, one for recovery — keep concerns and panic surfaces independent. Detection runs unconditionally so the new wedge fields surface even on clusters that have not opted into recovery; recovery runs per cluster according to `ClusterRecoveryConfig.enabled`.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────────────────┐
│                          serve_control_plane                            │
│                                                                         │
│   build_state(config, request)                                          │
│        │                                                                │
│        ├── existing background workers                                  │
│        │     └── thread::spawn refresh_state                            │
│        │                                                                │
│        ├── NEW: thread::spawn wedge_detector_loop(state)                │
│        │     ├── loop { sleep(interval); run_wedge_detector_tick(...) }│
│        │     └── writes ControlPlaneStateInner.wedge_state              │
│        │                                                                │
│        └── NEW: thread::spawn recovery_runner_loop(state)               │
│              ├── loop { sleep(interval);                                │
│              │         for cluster in state.recovery_configs:           │
│              │           if cluster.enabled:                            │
│              │             reconcile_recovery(cluster, state); }        │
│              ├── reads wedge_state, persisted recovery records          │
│              ├── decide_recovery_action(...)                            │
│              ├── try_acquire_recovery_lock(...)                         │
│              ├── tier-1: stop_machine + launch_local_machine            │
│              ├── tier-2: drop_machine_rootfs_overlay + launch           │
│              ├── tier-3: emit_tier_3_escalation event sink              │
│              └── persist updated record + emit event                    │
│                                                                         │
│   axum::serve(...)  ← unaffected                                        │
└────────────────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `run_wedge_detector_tick` | Existing pure function | Compute wedge facts from heartbeat ages and write to wedge_state map | `crates/port-runtime/src/hosted_control_plane.rs:5063` |
| `decide_recovery_action` | Existing pure function | Decide tier-1/2/3/auto-clear from wedge fact + persisted record + config | shipped in `34428c83` |
| `try_acquire_recovery_lock` | Existing RAII | Per-machine serialization with human lifecycle ops | shipped in `34428c83` |
| `RecoveryEventSink` | Existing | JSON-per-line event log with monotonic seq | shipped in `34428c83` |
| `load_recovery_record` / `save_recovery_record` / `clear_recovery_record` | Existing | Atomic persistence under `runtime/recovery/<machine>.json` | shipped in `34428c83` |
| `stop_machine`, `launch_local_machine`, `drop_machine_rootfs_overlay` | Existing | Tier-1 and tier-2 actions against the runtime root | crate-level, in `port-runtime` |
| `spawn_guest_heartbeat_probe_loop` pattern | Existing convention | `thread::spawn` of an infinite sleep+tick loop | `crates/port-runtime/src/hosted_control_plane.rs:5087` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Two workers vs one | Two separate `thread::spawn` workers (detector, recovery) | Matches existing `spawn_guest_heartbeat_probe_loop` separation of concerns; a panic in recovery cannot drop detector observation; tests can exercise either alone |
| Worker lifecycle | Spawned at `build_state` end, before `axum::serve`; never joined | Same pattern as the existing registration-refresh and heartbeat-probe workers; control-plane process exit takes them down |
| Per-tick error handling | Wrap each tick body in a closure; log on Err, continue loop | Per `SRS-NFR-02`; one bad cluster cannot stop the loop |
| Configurable intervals | Pull from `ClusterDetectionConfig` / `ClusterRecoveryConfig` validated values; tests use low values via the existing `ClusterDetectionConfig::with_…` test constructors | Per `SRS-NFR-03`; avoids hard-coded sleeps in tests |
| When to clear `AwaitingTier3HostRecycle` | Inside `launch_local_machine` (or its caller in the lifecycle path), via `clear_recovery_record` | Per `SRS-05`; the launch is the proof a host recycle ended successfully |
| What happens with a cluster missing from `PortConfig.clusters[*].recovery` | Treated as `enabled = false` (default) | Matches `RecoveryState::Disabled` semantics from existing types |

## Architecture

### Story decomposition

This voyage decomposes into two ordered stories:

1. **S2.1 — Wedge detector loop.** Spawn the detector worker from `serve_control_plane`. Detector-only mode: populates `wedge_state` against live heartbeat ages, never touches recovery records, never executes tier-1/2/3. Read-only side effects on `wedge_state`. Unblocks consumers (including spoke-sh/infra) from seeing wedge facts on every cluster regardless of recovery posture.

2. **S2.2 — Recovery runner loop.** Spawn the recovery worker from `serve_control_plane`. Reads `wedge_state` populated by S2.1, evaluates `decide_recovery_action` per machine when `ClusterRecoveryConfig.enabled`, executes tier-1 / tier-2, emits tier-3 escalation/return signal, persists records, acquires per-machine lock. Wires the auto-clear of `AwaitingTier3HostRecycle` on launch.

### Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `spawn_wedge_detector_loop(state: ControlPlaneState)` | New `thread::spawn` worker; periodic `run_wedge_detector_tick` against current heartbeat ages | New private function in `hosted_control_plane.rs`; called from `build_state` epilog |
| `spawn_recovery_runner_loop(state: ControlPlaneState)` | New `thread::spawn` worker; periodic `decide_recovery_action` per cluster + per machine | New private function in `hosted_control_plane.rs`; called from `build_state` epilog |
| `reconcile_machine_recovery(state, cluster, machine)` | Per-machine inner loop body: lock, decide, act, persist, emit | New private function; unit-testable in isolation |
| Auto-clear hook in `launch_local_machine` (or the hosted control-plane launch caller) | Calls `clear_recovery_record` on successful launch | New call site in the existing launch path |

### Test strategy

- **Unit:** `reconcile_machine_recovery` with stubbed action executors, exercising every `RecoveryAction` variant (Tier1Restart, Tier2Recreate, Tier3Escalate, Tier3AutoClear, Noop) and the lock-contention path.
- **Unit:** Auto-clear hook clears the persisted record on successful launch, leaves it untouched on failure.
- **Integration:** `serve_control_plane` started against a sample config with `recovery.enabled = true` populates `wedged_since_unix_s` for a machine whose guest heartbeat age exceeds the trigger; the same scenario with `enabled = false` populates the field but never persists a recovery record.
- **Integration:** Two consecutive ticks with a wedge that does not clear advance the recovery ladder Tier1 → Tier2 → Tier3 according to config thresholds.

## Risks

| Risk | Mitigation |
|------|------------|
| Recovery worker calls `launch_local_machine` while a human runs `port machine launch` on the same machine | `try_acquire_recovery_lock` serializes; lock contention skips and retries |
| Detector worker holds the wedge_state RwLock across a long tick | Tick body restricts the write lock to the moment of `apply_wedge_observation`; reads (heartbeat ages) acquire their own short-lived locks |
| Worker thread panics and silently dies | Per-tick wrapper logs and continues; if needed, a future story adds a watchdog that re-spawns dead workers |
| Recovery record persistence races with tier execution | `save_recovery_record` is atomic via `tempfile + rename`; `try_acquire_recovery_lock` covers the read-modify-write window |
| Tests become flaky on CI sleep timing | All intervals come from config; tests construct configs with sub-second tick intervals so the sleep-tick loop runs deterministically inside test time budgets |
