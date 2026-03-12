---
id: VDfzLrZ4e
title: Introduce Hosted K3s Cluster Contract
type: feat
status: done
created_at: 2026-03-12T10:44:41
updated_at: 2026-03-12T10:51:00
operator-signal: 
scope: VDcStSMlp/VDfytSpPs
index: 1
started_at: 2026-03-12T10:47:38
completed_at: 2026-03-12T10:51:00
---

# Introduce Hosted K3s Cluster Contract

## Summary

Add the first explicit hosted K3s cluster contract so Port can describe one
hosted-control-plane cluster in terms of an existing control plane, one host
group, one server machine, and one or more worker machines without replacing
the current machine and guest model.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] The model and config surfaces add a canonical hosted K3s cluster contract that binds one control plane, one host group, one server machine, one or more worker machines, and first-slice bootstrap metadata. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-model hosted_k3s_cluster_contract, proof: ac-1.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-02] Existing hosted machine, guest, service, local, and SSH contracts remain valid for configs that do not declare a K3s cluster. <!-- [SRS-NFR-02/AC-02] verify: cargo test -q -p port-model hosted_k3s_cluster_contract_regression_existing_routes, proof: ac-2.log -->
