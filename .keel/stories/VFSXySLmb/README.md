---
# system-managed
id: VFSXySLmb
status: done
created_at: 2026-03-31T08:32:10
updated_at: 2026-03-31T10:06:07
# authored
title: Add Configurable Host-to-Guest Port Forwards
type: feat
operator-signal:
scope: VFSWpHXG1/VFSXWpO18
index: 3
started_at: 2026-03-31T10:04:58
submitted_at: 2026-03-31T10:06:01
completed_at: 2026-03-31T10:06:07
---

# Add Configurable Host-to-Guest Port Forwards

## Summary

Allow operators to declare additional host→guest port forwards beyond the API
tunnel (:6443) in port.toml or machine spec. Port establishes these forwards
at boot so workstation tools can reach cluster services like MinIO console,
Envoy preview proxy, and Prometheus without manual kubectl port-forward.

## Acceptance Criteria

- [x] [SRS-07/AC-01] Operators can declare additional host→guest port forwards in port.toml or machine spec, and Port establishes them at boot. `ServiceForwardSpec` added to `ClusterLifecycleSpec.forwards`; `cluster up` calls `ensure_detached_forward()` for each; `cluster down` tears them down. Example in `examples/port.toml`: `nodeport-http` and `nodeport-https`. <!-- verify: manual, SRS-07:start:end -->
- [x] [SRS-NFR-03/AC-02] The implementation stays bounded to local single-node NAT networking; no bridged, routed, AWS, or multi-node networking is introduced. All changes are scoped to local Firecracker TAP/NAT with static IPs on 172.16.0.0/24. <!-- verify: manual, SRS-NFR-03:start:end -->
