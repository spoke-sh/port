---
# system-managed
id: VGafyU6FW
status: done
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T08:53:41
# authored
title: Require Honest Real-HA Topology Admission For Hosted AWS PVM
type: feat
operator-signal:
scope: VGYFpfUph/VGafx2cmq
index: 1
started_at: 2026-04-12T08:50:46
completed_at: 2026-04-12T08:53:41
---

# Require Honest Real-HA Topology Admission For Hosted AWS PVM

## Summary

Define the admission boundary for real HA on hosted AWS PVM so Port only treats
clusters as HA-capable when the current topology and scheduler contract can
actually spread the control plane across distinct execution hosts.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Hosted AWS PVM K3s configs that try to claim real HA without at least three control-plane machines and `control_plane_scheduler = "spread"` are rejected or classified as non-HA. Verified by `cargo test -q -p port-model hosted_k3s -- --nocapture` in `EVIDENCE/ac-1.model.log`, which covers the new topology posture classification for two- and three-control-plane clusters. <!-- verify: command, SRS-01:start:end, proof: ac-1.model.log -->
- [x] [SRS-01/AC-02] Hosted admission fails with explicit host-group and candidate-node detail when distinct execution hosts are unavailable for the requested control-plane spread. Verified by `cargo test -q -p port-runtime hosted_k3s_spread_scheduler -- --nocapture` in `EVIDENCE/ac-2.runtime.log`. <!-- verify: command, SRS-01:start:end, proof: ac-2.runtime.log -->
- [x] [SRS-NFR-02/AC-03] Port does not silently reuse an occupied execution host and still present the cluster as HA. Verified by `cargo test -q -p port-runtime hosted_k3s_spread_scheduler -- --nocapture` in `EVIDENCE/ac-2.runtime.log`, which exercises the occupied-host rejection path for spread scheduling. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.runtime.log -->
