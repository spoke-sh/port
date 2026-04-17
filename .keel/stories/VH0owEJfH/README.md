---
# system-managed
id: VH0owEJfH
status: icebox
created_at: 2026-04-16T19:41:41
updated_at: 2026-04-16T19:41:41
# authored
title: Spawn Wedge Detector Tick Loop From Live Control Plane
type: feat
operator-signal:
scope: VH0mU3DbK/VH0mlnCSq
index: 1
---

# Spawn Wedge Detector Tick Loop From Live Control Plane

## Summary

Spawn the wedge detector tick as a `thread::spawn` background worker from `build_state` inside `serve_control_plane`, mirroring the existing `spawn_guest_heartbeat_probe_loop` pattern. The worker periodically calls `run_wedge_detector_tick` against the heartbeat ages tracked in `ControlPlaneStateInner`, with intervals and trigger thresholds pulled from `ClusterDetectionConfig`. Detector observation is unconditional and writes only to `wedge_state`; recovery actions are deferred to the sibling story.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `serve_control_plane` started against a sample config spawns a wedge detector worker; after one detector interval, `wedge_state` reflects the wedge classification produced by `run_wedge_detector_tick` against the current heartbeat ages, regardless of any cluster's recovery configuration. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime detector_loop_populates_wedge_state, proof: ac-1.log -->
- [ ] [SRS-NFR-01/AC-02] The detector worker runs as a separate `thread::spawn` and does not block `axum::serve` or any HTTP handler. <!-- [SRS-NFR-01/AC-02] verify: cargo test -p port-runtime detector_loop_does_not_block_serve, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-03] A panic inside one detector tick is wrapped, logged, and does not stop the loop or the control-plane process; the next tick proceeds normally. <!-- [SRS-NFR-02/AC-03] verify: cargo test -p port-runtime detector_loop_survives_panic, proof: ac-3.log -->
