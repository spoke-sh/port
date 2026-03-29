---
# system-managed
id: VFH8C1KHM
status: done
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T10:59:04
# authored
title: Restore Live Cluster Status And Kubeconfig Handoff
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 2
started_at: 2026-03-29T10:24:05
submitted_at: 2026-03-29T10:59:03
completed_at: 2026-03-29T10:59:04
---

# Restore Live Cluster Status And Kubeconfig Handoff

## Summary

Restore the live cluster handoff so Port reports a healthy single-node cluster
and returns a kubeconfig downstream tooling can use directly.

## Acceptance Criteria

- [x] [SRS-02/AC-01] `port cluster status --cluster demo --runtime-root <tmp> --format json` reports readiness=`ready`, machine_state=`running`, and kubeconfig_available=`true` after the repaired local cluster boots. Verified in `EVIDENCE/ac-1.cluster-status.json`. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-NFR-02/AC-02] `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` plus `kubectl get nodes -o wide` works without downstream kubeconfig rewriting or fallback `guest exec` choreography. Verified in `EVIDENCE/ac-2.cluster-kubeconfig.json`, `EVIDENCE/ac-2.kubectl.log`, and `EVIDENCE/ac-2.cluster-down.json`. <!-- verify: manual, SRS-NFR-02:start:end -->
