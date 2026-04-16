---
# system-managed
id: VH00C3h3h
status: icebox
created_at: 2026-04-16T16:20:07
updated_at: 2026-04-16T16:20:07
# authored
title: Implement Control-Plane Wedge Detector Task
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxlScKS
index: 2
---

# Implement Control-Plane Wedge Detector Task

## Summary

Add the background detector task on the hosted control plane. The task evaluates each registered machine's `refresh_age_seconds` and `guest_refresh_age_seconds` against the configured thresholds at a fixed interval, and writes a `WedgeFact { wedged_since_unix_s, wedge_class }` into an in-memory `wedge_state` map when a trigger fires. When both triggers fire on the same machine, prefer `wedge_class = "node"` because tier-1/tier-2 recovery cannot reach a silent node-agent. The detector must not mutate any machine or guest runtime — this is a pure observer.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] A periodic detector task in the hosted control plane walks the registered-machine list, evaluates both triggers, and writes `(wedged_since_unix_s, wedge_class)` into `wedge_state` on the first stale read; the task clears the entry when both triggers are false again. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- wedge_detector_sets_and_clears, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] When both node and guest triggers fire on the same machine, the detector records `wedge_class = "node"` (tie-breaker covered by a dedicated test). <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- wedge_class_prefers_node_when_both_trigger, proof: ac-2.log -->
- [ ] [SRS-NFR-01/AC-01] A fault-injection test seeds heartbeat staleness for several machines and asserts the detector produces no `machine stop/launch` side effects — only `wedge_state` writes. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- wedge_detector_has_no_lifecycle_side_effects, proof: ac-3.log -->
- [ ] [SRS-NFR-02/AC-01] The detector task owns its own `tokio::time::interval` handle; a unit test constructs the runner in isolation and asserts its tick frequency is governed by config rather than by any external loop or read path. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- wedge_detector_runs_on_independent_interval, proof: ac-4.log -->
