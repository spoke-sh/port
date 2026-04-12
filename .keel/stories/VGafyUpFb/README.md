---
# system-managed
id: VGafyUpFb
status: backlog
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T08:28:05
# authored
title: Model Stable HA Endpoint Handoff In Cluster Output
type: feat
operator-signal:
scope: VGYFpfmpi/VGafx2vn4
index: 1
---

# Model Stable HA Endpoint Handoff In Cluster Output

## Summary

Make the stable HA API endpoint explicit in Port's cluster handoff surfaces so
downstream consumers receive one canonical `api_endpoint` contract instead of a
guest-specific address that drifts during failover.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port cluster up`, `port cluster status`, and `port cluster kubeconfig` hand off the configured `api_endpoint` as the stable cluster address for eligible hosted AWS PVM HA clusters. <!-- verify: automated, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Cluster-facing output reports stable-endpoint HA posture and missing failover prerequisites explicitly. <!-- verify: automated, SRS-02:start:end -->
- [ ] [SRS-NFR-02/AC-03] Port does not claim a stable HA endpoint when the flow still depends on manual downstream rewrites or one control-plane guest address. <!-- verify: automated, SRS-NFR-02:start:end -->
