---
# system-managed
id: VH00CG8GV
status: icebox
created_at: 2026-04-16T16:20:08
updated_at: 2026-04-16T16:20:08
# authored
title: Surface Wedged Since And Wedge Class In Cluster Status
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxlScKS
index: 3
---

# Surface Wedged Since And Wedge Class In Cluster Status

## Summary

Thread the detector's `wedge_state` map through the existing machine status path so `port cluster status --format json` exposes `wedged_since: Option<u64>` and `wedge_class: Option<String>` per machine, omitted via `skip_serializing_if` when `None`. Mirror the pattern established by `refresh_age_seconds` for the human-readable render: add two new lines with a `(none)` fallback so operators can read it without `--format json`.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] The per-machine status struct and its JSON serialization grow `wedged_since: Option<u64>` and `wedge_class: Option<String>` (skipped when `None`); the CLI human-readable render includes two new lines with `(none)` fallbacks and an integration test covers both the JSON shape and the render lines. <!-- [SRS-04/AC-01] verify: cargo test -p port --lib -- hosted_fleet_render_includes_wedge_fields, proof: ac-1.log -->
