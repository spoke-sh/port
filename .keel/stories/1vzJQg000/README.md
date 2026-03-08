---
id: 1vzJQg000
title: Define Prepared Pvm Host Kit Contract
type: feat
status: done
created_at: 2026-03-08T12:04:42
updated_at: 2026-03-08T12:22:35
scope: 1vzJKE000/1vzJP2000
started_at: 2026-03-08T12:09:18
completed_at: 2026-03-08T12:22:35
---

# Define Prepared Pvm Host Kit Contract

## Summary

Define the canonical prepared-node PVM host-kit contract so Port can tell the
difference between a merely admission-ready node and a node that can actually
launch x86_64 Firecracker/PVM workloads.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] Port model, doctor, and runtime preflight define the prepared-node x86_64 PVM host-kit inputs explicitly, including patched Firecracker binary selection and required host prerequisites. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQg000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-01/AC-02] Missing or malformed prepared-node PVM host-kit state fails with explicit host-kit detail instead of generic runtime launch errors. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzJQg000/verify-ac-2.sh, proof: ac-2.log -->
