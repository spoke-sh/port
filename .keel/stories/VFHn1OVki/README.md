---
# system-managed
id: VFHn1OVki
status: backlog
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T12:24:38
# authored
title: Replace Demo Local Cluster Stub With Real K3s Control Plane
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 1
---

# Replace Demo Local Cluster Stub With Real K3s Control Plane

## Summary

Replace the shipped demo local control-plane behavior with a real single-node
K3s boot path so `port cluster up --cluster demo` brings up an actual local
cluster rather than a stub API.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux and the resulting cluster runtime is backed by a real K3s control plane rather than the current demo or stub path. <!-- verify: manual, SRS-01:start:end -->
- [ ] [SRS-NFR-02/AC-02] The implementation keeps Port as the owner of cluster boot and readiness; no downstream `guest exec`, kubeconfig rewrite, or raw machine choreography is reintroduced as part of the fix. <!-- verify: manual, SRS-NFR-02:start:end -->
