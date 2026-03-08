---
id: 1vzJSi000
title: Implement Node Agent Pvm Launch Path
type: feat
status: backlog
created_at: 2026-03-08T12:06:48
updated_at: 2026-03-08T12:07:52
scope: 1vzJKE000/1vzJP2000
---

# Implement Node Agent Pvm Launch Path

## Summary

Extend the node-agent runtime so prepared Linux hosts can actually launch
x86_64 Firecracker/PVM machines while keeping Port's canonical runtime
manifests and guest attach behavior intact.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] The node agent launches x86_64 PVM machines on prepared
  Linux hosts using the prepared host-kit contract, canonical artifact
  selection, and canonical runtime metadata layout.
- [ ] [SRS-02/AC-02] Automated and CLI proof keep the standard Firecracker lane
  executable while the prepared-node PVM launch path lands.
