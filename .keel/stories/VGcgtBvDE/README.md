---
# system-managed
id: VGcgtBvDE
status: done
created_at: 2026-04-12T16:39:11
updated_at: 2026-04-12T17:20:58
# authored
title: Persist Hosted Placement And Service Records Across Reuse
type: feat
operator-signal:
scope: VGcgU9T57/VGcghwZrb
index: 1
started_at: 2026-04-12T17:15:58
completed_at: 2026-04-12T17:20:58
---

# Persist Hosted Placement And Service Records Across Reuse

## Summary

Make hosted placement and managed-service records durable across reuse so
status, recovery, and downstream inspection do not depend on transient launch
state.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Hosted placement and managed-service records persist durably enough for reuse and service status to survive beyond launch-time state. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_bootstrap_persists_placement_and_service_records', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The persistence contract remains explicit in runtime artifacts and service-status surfaces. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_k3s_service_status_survives_from_persisted_records_after_launch', SRS-NFR-01:start:end, proof: ac-2.log-->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records `cargo test -q -p port-runtime hosted_k3s_bootstrap_persists_placement_and_service_records`, proving hosted K3s bootstrap writes durable placement state plus persisted `k3s-server` and `k3s-agent` service definitions under the hosted runtime roots.
- AC-02: `EVIDENCE/ac-2.log` records `cargo test -q -p port-runtime hosted_k3s_service_status_survives_from_persisted_records_after_launch`, proving hosted service-status lookups still return explicit managed-service ownership from persisted records after launch-time control-plane state is gone.
