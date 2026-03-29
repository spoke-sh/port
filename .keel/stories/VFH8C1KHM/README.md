---
# system-managed
id: VFH8C1KHM
status: backlog
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T09:41:57
# authored
title: Restore Live Cluster Status And Kubeconfig Handoff
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 2
---

# Restore Live Cluster Status And Kubeconfig Handoff

## Summary

Restore the live cluster handoff so Port reports a healthy single-node cluster
and returns a kubeconfig downstream tooling can use directly.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `port cluster status --cluster demo --runtime-root <tmp> --format json` reports readiness=`ready`, machine_state=`running`, and kubeconfig_available=`true` after the repaired local cluster boots. <!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-NFR-02/AC-02] `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` plus `kubectl get nodes -o wide` works without downstream kubeconfig rewriting or fallback `guest exec` choreography. <!-- verify: manual, SRS-NFR-02:start:end -->
