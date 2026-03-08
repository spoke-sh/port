---
id: 1vzJSi000
title: Implement Node Agent Pvm Launch Path
type: feat
status: done
created_at: 2026-03-08T12:06:48
updated_at: 2026-03-08T12:34:45
scope: 1vzJKE000/1vzJP2000
started_at: 2026-03-08T12:23:08
completed_at: 2026-03-08T12:34:45
---

# Implement Node Agent Pvm Launch Path

## Summary

Extend the node-agent runtime so prepared Linux hosts can actually launch
x86_64 Firecracker/PVM machines while keeping Port's canonical runtime
manifests and guest attach behavior intact.

## Acceptance Criteria

<!-- verify: command, SRS-02:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] The node agent launches x86_64 PVM machines on prepared Linux hosts using the prepared host-kit contract, canonical artifact selection, and canonical runtime metadata layout. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJSi000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Automated and CLI proof keep the standard Firecracker lane executable while the prepared-node PVM launch path lands. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJSi000/verify-ac-2.sh, proof: ac-2.log -->
