---
id: 1vzJQh000
title: Route Hosted Launch Through Prepared Pvm Nodes
type: feat
status: backlog
created_at: 2026-03-08T12:04:43
updated_at: 2026-03-08T12:07:52
scope: 1vzJKE000/1vzJP2000
---

# Route Hosted Launch Through Prepared Pvm Nodes

## Summary

Replace hosted PVM provider guidance with a live control-plane and node-agent
launch path once a machine has been admitted onto a prepared x86_64 PVM node.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Hosted `port machine launch` routes admission-ready PVM
  machines through the live control-plane and prepared-node node-agent path
  instead of stopping at provider guidance.
- [ ] [SRS-03/AC-02] Hosted PVM launch failures surface explicit placement or
  host-kit causes without regressing existing hosted standard-machine
  workflows.
