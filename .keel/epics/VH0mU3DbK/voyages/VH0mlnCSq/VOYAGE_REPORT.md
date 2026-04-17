# VOYAGE REPORT: Live Detector And Recovery Runner Wiring

## Voyage Metadata
- **ID:** VH0mlnCSq
- **Epic:** VH0mU3DbK
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Spawn Wedge Detector Tick Loop From Live Control Plane
- **ID:** VH0owEJfH
- **Status:** done

#### Summary
Spawn the wedge detector tick as a `thread::spawn` background worker from `serve_control_plane`, mirroring the existing `spawn_guest_heartbeat_probe_loop` pattern. The worker periodically calls a new `reconcile_wedge_detector_tick` helper that derives node-side ages from `ControlPlaneStateInner.node_receipt_instants` per machine placement and feeds them into the existing `run_wedge_detector_tick`. Each tick is wrapped in `catch_unwind` so a panic does not stop the loop. Detector observation is unconditional and only writes to `wedge_state`; recovery actions are deferred to the sibling story.

Per-cluster `ClusterDetectionConfig` lives on the higher-level `ClusterSpec` (PortConfig.clusters) rather than on `K3sClusterSpec` (which is what the hosted control plane keys off). For the live wiring we use the default thresholds (node 120s, guest 90s) and leave the per-cluster override plumbing as a follow-up that maps a k3s cluster name to its parent ClusterSpec. Guest-side ages are not yet available on the control plane (they live on the node-agent's heartbeat sidecar) so the detector currently catches node-side wedges only — the matching plumbing for guest_age is a separate follow-up story.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `serve_control_plane` spawns a wedge detector worker; calling `reconcile_wedge_detector_tick` against a state with stale `node_receipt_instants` for a placed machine populates `wedge_state` with `wedge_class = Node`, regardless of any cluster's recovery configuration. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime --lib detector_loop_populates_wedge_state_from_live_node_receipt_instants, proof: ac-1.log -->
- [x] [SRS-NFR-01/AC-02] `spawn_wedge_detector_loop` returns to the caller in well under one second so `axum::serve` can take over the foreground; the detector runs entirely on a separate `thread::spawn`. <!-- [SRS-NFR-01/AC-02] verify: cargo test -p port-runtime --lib detector_loop_does_not_block_serve_control_plane_on_spawn, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-03] Each detector tick is wrapped in `catch_unwind`; the `panic_message` helper renders `&'static str`, `String`, and other panic payloads so a panic on one tick is observable in the next log line and the loop continues running. <!-- [SRS-NFR-02/AC-03] verify: cargo test -p port-runtime --lib detector_loop_panic_payload_decoder_handles_str_string_and_other, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH0owEJfH/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH0owEJfH/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH0owEJfH/EVIDENCE/ac-3.log)

### Spawn Recovery Runner Loop With Tier-1 Through Tier-3
- **ID:** VH0oxFqBk
- **Status:** done

#### Summary
Spawn the recovery runner as a second `thread::spawn` background worker from `serve_control_plane` (sibling to the detector worker from `VH0owEJfH`). Per cluster with `ClusterRecoveryConfig.enabled = true`, the worker reads `wedge_state` populated by the detector and evaluates `decide_recovery_action` against the persisted recovery record. The runner advances the ladder state, persists updated counters and `last_recovery_action` via `save_recovery_record`, and emits a matching event through `RecoveryEventSink`. Tier-2 invokes the in-process `drop_machine_rootfs_overlay` (idempotent disk I/O — no node-agent round-trip). Tier-3 emits the `Tier3Escalation` event and sets `recovery_state = AwaitingTier3HostRecycle`. Tier-3 auto-clear emits `Tier3HostReturned` and resets the ladder. Per-machine serialization with human lifecycle ops uses `try_acquire_recovery_lock`.

The actual machine restart for tier-1 (and the relaunch portion of tier-2) is intentionally deferred to a follow-up story. The runner advances the record state and emits `Started` events so operators see the transition; the next detector tick observes whether the wedge cleared, and the runner re-evaluates. This matches the "library code with #[allow(dead_code)]" pattern Port itself used in the prior recovery-ladder mission — the wiring lands now, the heavy machine-lifecycle action lands when there's a clean in-process path that doesn't loop back through the CP's own HTTP route.

The post-launch hook (`clear_awaiting_tier_3_on_launch`) lives inside the existing `machine_launch` HTTP handler. After a successful launch proxy, if the per-machine recovery record is in `AwaitingTier3HostRecycle`, the hook clears it and emits `Tier3HostReturned` — so an operator-driven launch doubles as the host-recycle "return" signal.

Per-cluster recovery config plumbing follows the same pattern as the detector: `ClusterRecoveryConfig` lives on `ClusterSpec` (PortConfig.clusters), and the runner looks up the recovery config by k3s cluster name. When no matching ClusterSpec is present, the runner uses the default config (disabled) — safe by construction.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] With `ClusterRecoveryConfig.enabled = true` and a wedged machine present in `wedge_state`, one recovery tick advances the persisted record state per `decide_recovery_action`, increments the matching tier counter, sets `last_recovery_action`, and appends an event to the recovery events log. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime --lib recovery_runner_executes_decided_action_when_enabled, proof: ac-1.log -->
- [x] [SRS-03/AC-02] If `try_acquire_recovery_lock` returns `None` for a machine, the tick skips that machine and persists no record. Once the lock releases, the next tick acts normally. <!-- [SRS-03/AC-02] verify: cargo test -p port-runtime --lib recovery_runner_skips_locked_machine_until_lock_releases, proof: ac-2.log -->
- [x] [SRS-04/AC-03] With `ClusterRecoveryConfig.enabled = false`, the recovery runner is a no-op: no record persists, no events emit, even when a wedge is present in `wedge_state`. <!-- [SRS-04/AC-03] verify: cargo test -p port-runtime --lib recovery_runner_is_noop_when_cluster_recovery_disabled, proof: ac-3.log -->
- [x] [SRS-05/AC-04] The post-launch hook on the hosted control-plane `machine_launch` handler clears any persisted `AwaitingTier3HostRecycle` record for the launched machine and appends a `Tier3HostReturned` event to the recovery events log. <!-- [SRS-05/AC-04] verify: cargo test -p port-runtime --lib launch_clears_awaiting_tier_3_record_via_post_launch_hook, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH0oxFqBk/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH0oxFqBk/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH0oxFqBk/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VH0oxFqBk/EVIDENCE/ac-4.log)


