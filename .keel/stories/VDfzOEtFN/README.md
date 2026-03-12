---
id: VDfzOEtFN
title: Implement Hosted K3s Bootstrap And Join Workflow
type: feat
status: backlog
created_at: 2026-03-12T10:44:50
updated_at: 2026-03-12T10:46:00
operator-signal: 
scope: VDcStSMlp/VDfytSpPs
index: 3
---

# Implement Hosted K3s Bootstrap And Join Workflow

## Summary

Implement the first hosted K3s bootstrap workflow so Port can bring up one K3s
server node, join at least one worker node, and keep that lifecycle on the
canonical hosted machine and guest path.

## Acceptance Criteria

<!-- verify: command, SRS-02:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] The hosted K3s workflow bootstraps one server machine and joins at least one worker machine through canonical machine lifecycle and guest-control surfaces. <!-- [SRS-02/AC-01] verify: cargo test -q hosted_k3s_bootstrap_and_join_workflow, proof: ac-1.log -->
