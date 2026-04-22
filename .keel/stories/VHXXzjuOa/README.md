---
# system-managed
id: VHXXzjuOa
status: done
created_at: 2026-04-22T10:01:22
updated_at: 2026-04-22T11:22:03
# authored
title: Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background
type: feat
operator-signal:
scope: VHXXs1f1f/VHXXxt7rF
index: 2
started_at: 2026-04-22T10:58:33
completed_at: 2026-04-22T11:22:03
---

# Move Hosted Placement Repair Out Of Read Paths And Reconcile In The Background

## Summary

Move hosted placement repair out of synchronous read handlers and into explicit
reconcile hooks so placement persistence becomes deterministic, canonical, and
separate from operator truth surfaces.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] Hosted request handlers stop persisting placement state on read paths; placement repair is triggered only by startup, registration, or lifecycle hooks. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime registration_reconcile_ -- --nocapture && cargo test -p port-runtime proxy_bytes_does_not_purge_stale_machine_placement_when_guest_socket_is_missing -- --nocapture', proof: ac-2.log -->
<!-- verify: command, SRS-03:end -->
<!-- verify: command, SRS-04:start:end, proof: ac-5.log -->
- [x] [SRS-04/AC-02] The placement reconciler canonicalizes legacy node aliases to configured node identities and persists repaired machine placement without requiring a user read path. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime startup_reconcile_canonicalizes_legacy_node_alias_to_configured_node_identity -- --nocapture', proof: ac-5.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-8.log -->
- [x] [SRS-NFR-02/AC-03] No hosted request handler reintroduces control-plane self-recursion or synchronous write-on-read repair while reconciling placement. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime effective_recovery_wedge_uses_live_node_service_status_without_self_calling_control_plane -- --nocapture && cargo test -p port-runtime proxy_bytes_does_not_purge_stale_machine_placement_when_guest_socket_is_missing -- --nocapture', proof: ac-8.log -->
<!-- verify: command, SRS-NFR-02:end -->
