---
id: VDfzOEdFM
title: Add Hosted K3s Access And Boundary Surfaces
type: feat
status: backlog
created_at: 2026-03-12T10:44:50
updated_at: 2026-03-12T10:46:00
operator-signal: 
scope: VDcStSMlp/VDfytSpPs
index: 2
---

# Add Hosted K3s Access And Boundary Surfaces

## Summary

Surface cluster access, placement detail, and first-slice failure boundaries so
operators can inspect the hosted K3s lane through canonical Port surfaces
without mistaking it for an HA or persistent Kubernetes platform.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] The hosted K3s lane exposes kubeconfig or equivalent cluster access plus node or workload visibility through canonical operator surfaces. <!-- [SRS-03/AC-01] verify: cargo test -q hosted_k3s_cluster_access_contract, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Unsupported hosted K3s requests fail fast with explicit boundary guidance for missing host-group capacity, persistence, HA, ingress, or non-hosted ownership routes. <!-- [SRS-04/AC-02] verify: cargo test -q hosted_k3s_boundary_failures, proof: ac-2.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-3.log -->
- [ ] [SRS-NFR-01/AC-03] Placement and lifecycle output for hosted K3s keeps control-plane, host-group, candidate-node, selected-node, and rejected-node detail explicit. <!-- [SRS-NFR-01/AC-03] verify: cargo test -q hosted_k3s_route_context_visibility, proof: ac-3.log -->
