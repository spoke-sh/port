---
# system-managed
id: VH00Brdus
status: done
created_at: 2026-04-16T16:20:06
updated_at: 2026-04-16T17:11:00
# authored
title: Add Cluster Detection Config Block With Threshold Defaults
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxlScKS
index: 1
started_at: 2026-04-16T17:08:30
submitted_at: 2026-04-16T17:10:59
completed_at: 2026-04-16T17:11:00
---

# Add Cluster Detection Config Block With Threshold Defaults

## Summary

Introduce the configuration surface the detector reads from. Add a `[clusters.<name>.detection]` block in `port.toml` with two knobs: `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds`. Pick conservative defaults (guest: 90s, node: 120s) so an absent block still produces sensible behaviour. Reject zero or negative values with a clear validation error. This story owns only the config plumbing; the detector task that reads these values comes next.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `port-model` defines `ClusterDetectionConfig` parsed from `[clusters.<name>.detection]` with `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds`; absent block falls back to documented defaults (guest: 90s, node: 120s); zero values produce an actionable validation error. <!-- [SRS-01/AC-01] verify: cargo test -p port-model -- cluster_detection_config, proof: ac-1.log -->
