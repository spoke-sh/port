---
id: 1vzQJ6000
title: Implement Hosted Detached Forward Inventory
type: feat
status: backlog
created_at: 2026-03-08T19:25:20
updated_at: 2026-03-08T19:27:01
scope: 1vzETR000/1vzQEj000
---

# Implement Hosted Detached Forward Inventory

## Summary

Implement the node-owned detached forward start, list, and stop behavior for
hosted machines so lifecycle state lives under the runtime-owning node instead
of the repo-local CLI.

## Acceptance Criteria

<!-- verify: command, SRS-02:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] The node agent can start a detached hosted forward and return the resulting manifest summary from node-owned runtime state. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJ6000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Hosted detached forward list and stop operate on node-owned manifests and clean up runtime artifacts without depending on repo-local CLI state. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJ6000/verify-ac-2.sh, proof: ac-2.log -->
