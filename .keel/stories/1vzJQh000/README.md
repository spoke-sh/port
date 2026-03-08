---
id: 1vzJQh000
title: Route Hosted Launch Through Prepared Pvm Nodes
type: feat
status: in-progress
created_at: 2026-03-08T12:04:43
updated_at: 2026-03-08T12:35:15
scope: 1vzJKE000/1vzJP2000
started_at: 2026-03-08T12:35:15
---

# Route Hosted Launch Through Prepared Pvm Nodes

## Summary

Replace hosted PVM provider guidance with a live control-plane and node-agent
launch path once a machine has been admitted onto a prepared x86_64 PVM node.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-01] Hosted `port machine launch` routes admission-ready PVM machines through the live control-plane and prepared-node node-agent path instead of stopping at provider guidance. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQh000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-02] Hosted PVM launch failures surface explicit placement or host-kit causes without regressing existing hosted standard-machine workflows. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQh000/verify-ac-2.sh, proof: ac-2.log -->
