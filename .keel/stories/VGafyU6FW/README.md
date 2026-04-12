---
# system-managed
id: VGafyU6FW
status: backlog
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T08:28:03
# authored
title: Require Honest Real-HA Topology Admission For Hosted AWS PVM
type: feat
operator-signal:
scope: VGYFpfUph/VGafx2cmq
index: 1
---

# Require Honest Real-HA Topology Admission For Hosted AWS PVM

## Summary

Define the admission boundary for real HA on hosted AWS PVM so Port only treats
clusters as HA-capable when the current topology and scheduler contract can
actually spread the control plane across distinct execution hosts.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Hosted AWS PVM K3s configs that try to claim real HA without at least three control-plane machines and `control_plane_scheduler = "spread"` are rejected or classified as non-HA. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-01/AC-02] Hosted admission fails with explicit host-group and candidate-node detail when distinct execution hosts are unavailable for the requested control-plane spread. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] Port does not silently reuse an occupied execution host and still present the cluster as HA. <!-- verify: automated, SRS-NFR-02:start:end -->
