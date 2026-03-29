---
# system-managed
id: VFHn1OVki
status: done
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T16:17:57
# authored
title: Replace Demo Local Cluster Stub With Real K3s Control Plane
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 1
started_at: 2026-03-29T12:27:56
submitted_at: 2026-03-29T16:17:49
completed_at: 2026-03-29T16:17:57
---

# Replace Demo Local Cluster Stub With Real K3s Control Plane

## Summary

Replace the shipped demo local control-plane behavior with a real single-node
K3s boot path so `port cluster up --cluster demo` brings up an actual local
cluster rather than a stub API.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux and the resulting cluster runtime is backed by a real K3s control plane rather than the current demo or stub path. Verified in `EVIDENCE/ac-1.cluster-up.json`, `EVIDENCE/ac-1.console.stdout.log`, and `EVIDENCE/ac-1.firecracker-config.json`. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-NFR-02/AC-02] The implementation keeps Port as the owner of cluster boot and readiness; no downstream `guest exec`, kubeconfig rewrite, or raw machine choreography is reintroduced as part of the fix. Verified in `EVIDENCE/ac-1.cluster-up.json` and `EVIDENCE/ac-1.cluster-down.json`, where Port owns launch, bootstrap, readiness, and teardown through the cluster surface alone. <!-- verify: manual, SRS-NFR-02:start:end -->
