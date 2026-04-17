---
# system-managed
id: VH0owEJfH
status: done
created_at: 2026-04-16T19:41:41
updated_at: 2026-04-16T23:04:13
# authored
title: Spawn Wedge Detector Tick Loop From Live Control Plane
type: feat
operator-signal:
scope: VH0mU3DbK/VH0mlnCSq
index: 1
started_at: 2026-04-16T20:37:40
submitted_at: 2026-04-16T23:04:13
completed_at: 2026-04-16T23:04:13
---

# Spawn Wedge Detector Tick Loop From Live Control Plane

## Summary

Spawn the wedge detector tick as a `thread::spawn` background worker from `serve_control_plane`, mirroring the existing `spawn_guest_heartbeat_probe_loop` pattern. The worker periodically calls a new `reconcile_wedge_detector_tick` helper that derives node-side ages from `ControlPlaneStateInner.node_receipt_instants` per machine placement and feeds them into the existing `run_wedge_detector_tick`. Each tick is wrapped in `catch_unwind` so a panic does not stop the loop. Detector observation is unconditional and only writes to `wedge_state`; recovery actions are deferred to the sibling story.

Per-cluster `ClusterDetectionConfig` lives on the higher-level `ClusterSpec` (PortConfig.clusters) rather than on `K3sClusterSpec` (which is what the hosted control plane keys off). For the live wiring we use the default thresholds (node 120s, guest 90s) and leave the per-cluster override plumbing as a follow-up that maps a k3s cluster name to its parent ClusterSpec. Guest-side ages are not yet available on the control plane (they live on the node-agent's heartbeat sidecar) so the detector currently catches node-side wedges only — the matching plumbing for guest_age is a separate follow-up story.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end -->
<!-- verify: manual, SRS-NFR-01:start:end -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-01/AC-01] `serve_control_plane` spawns a wedge detector worker; calling `reconcile_wedge_detector_tick` against a state with stale `node_receipt_instants` for a placed machine populates `wedge_state` with `wedge_class = Node`, regardless of any cluster's recovery configuration. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime --lib detector_loop_populates_wedge_state_from_live_node_receipt_instants, proof: ac-1.log -->
- [x] [SRS-NFR-01/AC-02] `spawn_wedge_detector_loop` returns to the caller in well under one second so `axum::serve` can take over the foreground; the detector runs entirely on a separate `thread::spawn`. <!-- [SRS-NFR-01/AC-02] verify: cargo test -p port-runtime --lib detector_loop_does_not_block_serve_control_plane_on_spawn, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-03] Each detector tick is wrapped in `catch_unwind`; the `panic_message` helper renders `&'static str`, `String`, and other panic payloads so a panic on one tick is observable in the next log line and the loop continues running. <!-- [SRS-NFR-02/AC-03] verify: cargo test -p port-runtime --lib detector_loop_panic_payload_decoder_handles_str_string_and_other, proof: ac-3.log -->
