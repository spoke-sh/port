---
# system-managed
id: VHXXzjYOd
status: done
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T10:21:46
# authored
title: Make Hosted Machine And Service Status Live-First Under Placement Drift
type: feat
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 1
started_at: 2026-04-22T10:05:16
completed_at: 2026-04-22T10:21:46
---

# Make Hosted Machine And Service Status Live-First Under Placement Drift

## Summary

Replace stored-placement-first request handling for hosted machine and service
status with live-first resolution so the control plane reports runtime truth
instead of `malformed` when node-agent paths are still healthy.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-01/AC-01] `list_machines` and `machine_status` return live runtime truth when stored placement is missing or stale but a live node-agent route can still resolve the machine. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime control_plane_machine_status_falls_back_when_stored_placement_is_unusable -- --nocapture', proof: ac-2.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-5.log -->
- [x] [SRS-02/AC-02] Hosted service status and guest-route resolution prefer live placement or candidate-node resolution before failing on missing stored placement, and return explicit degraded detail when live refresh cannot succeed. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime control_plane_service_status_uses_live_route_without_stored_placement -- --nocapture && cargo test -p port-runtime control_plane_guest_exec_prefers_stored_placement_over_candidate_order -- --nocapture', proof: ac-5.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-8.log -->
- [x] [SRS-NFR-01/AC-03] Hosted machine/service status fan-out uses bounded per-machine deadlines so one bad route degrades partial truth instead of wedging the full response. <!-- [SRS-NFR-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime list_machines_degrades_a_timed_out_route_without_failing_the_fleet -- --nocapture', proof: ac-8.log -->
