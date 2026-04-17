# VOYAGE REPORT: Wedge Detector And Cluster Status Fields

## Voyage Metadata
- **ID:** VGzxlScKS
- **Epic:** VGzxKV9OX
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Cluster Detection Config Block With Threshold Defaults
- **ID:** VH00Brdus
- **Status:** done

#### Summary
Introduce the configuration surface the detector reads from. Add a `[clusters.<name>.detection]` block in `port.toml` with two knobs: `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds`. Pick conservative defaults (guest: 90s, node: 120s) so an absent block still produces sensible behaviour. Reject zero or negative values with a clear validation error. This story owns only the config plumbing; the detector task that reads these values comes next.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` defines `ClusterDetectionConfig` parsed from `[clusters.<name>.detection]` with `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds`; absent block falls back to documented defaults (guest: 90s, node: 120s); zero values produce an actionable validation error. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_detection_config, proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00Brdus/EVIDENCE/ac-1.log)

### Implement Control-Plane Wedge Detector Task
- **ID:** VH00C3h3h
- **Status:** done

#### Summary
Add the background detector task on the hosted control plane. The task evaluates each registered machine's `refresh_age_seconds` and `guest_refresh_age_seconds` against the configured thresholds at a fixed interval, and writes a `WedgeFact { wedged_since_unix_s, wedge_class }` into an in-memory `wedge_state` map when a trigger fires. When both triggers fire on the same machine, prefer `wedge_class = "node"` because tier-1/tier-2 recovery cannot reach a silent node-agent. The detector must not mutate any machine or guest runtime — this is a pure observer.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] A periodic detector task in the hosted control plane walks the registered-machine list, evaluates both triggers, and writes `(wedged_since_unix_s, wedge_class)` into `wedge_state` on the first stale read; the task clears the entry when both triggers are false again. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- wedge_detector_sets_and_clears, proof: ac-2.log -->
- [x] [SRS-03/AC-01] When both node and guest triggers fire on the same machine, the detector records `wedge_class = "node"` (tie-breaker covered by a dedicated test). <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- wedge_class_prefers_node_when_both_triggers_fire, proof: ac-4.log -->
- [x] [SRS-NFR-01/AC-01] A fault-injection test seeds heartbeat staleness for several machines and asserts the detector produces no `machine stop/launch` side effects — only `wedge_state` writes. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- wedge_detector_tick_has_no_machine_lifecycle_side_effects, proof: ac-3.log -->
- [x] [SRS-NFR-02/AC-01] The detector task owns its own interval constant; a unit test pins the constant so a future refactor can't silently wire the detector into an unrelated loop. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- wedge_detector_interval_is_a_dedicated_positive_duration, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00C3h3h/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH00C3h3h/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH00C3h3h/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VH00C3h3h/EVIDENCE/ac-4.log)

### Surface Wedged Since And Wedge Class In Cluster Status
- **ID:** VH00CG8GV
- **Status:** done

#### Summary
Thread the detector's `wedge_state` map through the existing machine status path so `port cluster status --format json` exposes `wedged_since: Option<u64>` and `wedge_class: Option<String>` per machine, omitted via `skip_serializing_if` when `None`. Mirror the pattern established by `refresh_age_seconds` for the human-readable render: add two new lines with a `(none)` fallback so operators can read it without `--format json`.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `MachineStatus` grows `wedged_since_unix_s: Option<u64>` and `wedge_class: Option<String>` (skipped when `None`); the control plane's `annotate_machine_status_with_fleet_state` reads the per-cluster `wedge_state` and populates them; the CLI human-readable render includes two new lines with `(none)` fallbacks. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- annotate_machine_status_surfaces_wedge, proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH00CG8GV/EVIDENCE/ac-1.log)


