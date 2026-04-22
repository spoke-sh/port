---
# system-managed
id: VHVArxbfS
status: done
created_at: 2026-04-22T00:16:58
updated_at: 2026-04-22T00:42:26
# authored
title: Unstick Hosted Service Truth Routing
type: bug
operator-signal:
scope: VGzxMc4G4
index: 1
started_at: 2026-04-22T00:17:02
completed_at: 2026-04-22T00:42:26
---

# Unstick Hosted Service Truth Routing

## Summary

Stop hosted control-plane wedge evaluation from routing hosted K3s service
truth back through the hosted control-plane client. That self-recursive path
can starve the two-thread control-plane runtime and stall cluster-status reads
that need kubeconfig or guest exec.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] Hosted wedge evaluation reads hosted K3s service runtime from the live node-agent status route with a short timeout and stored-runtime fallback instead of calling the hosted control-plane service-status path from inside the control-plane server. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime effective_recovery_wedge -- --nocapture && cargo test -p port-runtime recovery_runner_restarts_machine_through_live_hosted_route -- --nocapture && cargo test -p port-runtime hosted_service_list_status_and_stop_follow_stored_placement -- --nocapture && cargo test -p port-runtime hosted_k3s_machine_truth_leaves_wedge_fields_default_when_wedge_route_unreachable -- --nocapture', proof: ac-1.log -->
