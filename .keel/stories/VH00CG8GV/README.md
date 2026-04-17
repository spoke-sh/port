---
# system-managed
id: VH00CG8GV
status: done
created_at: 2026-04-16T16:20:08
updated_at: 2026-04-16T17:20:08
# authored
title: Surface Wedged Since And Wedge Class In Cluster Status
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxlScKS
index: 3
started_at: 2026-04-16T17:16:38
submitted_at: 2026-04-16T17:20:08
completed_at: 2026-04-16T17:20:08
---

# Surface Wedged Since And Wedge Class In Cluster Status

## Summary

Thread the detector's `wedge_state` map through the existing machine status path so `port cluster status --format json` exposes `wedged_since: Option<u64>` and `wedge_class: Option<String>` per machine, omitted via `skip_serializing_if` when `None`. Mirror the pattern established by `refresh_age_seconds` for the human-readable render: add two new lines with a `(none)` fallback so operators can read it without `--format json`.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] `MachineStatus` grows `wedged_since_unix_s: Option<u64>` and `wedge_class: Option<String>` (skipped when `None`); the control plane's `annotate_machine_status_with_fleet_state` reads the per-cluster `wedge_state` and populates them; the CLI human-readable render includes two new lines with `(none)` fallbacks. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime -- annotate_machine_status_surfaces_wedge, proof: ac-1.log -->
