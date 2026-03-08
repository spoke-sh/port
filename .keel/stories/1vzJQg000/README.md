---
id: 1vzJQg000
title: Define Prepared Pvm Host Kit Contract
type: feat
status: backlog
created_at: 2026-03-08T12:04:42
updated_at: 2026-03-08T12:07:52
scope: 1vzJKE000/1vzJP2000
---

# Define Prepared Pvm Host Kit Contract

## Summary

Define the canonical prepared-node PVM host-kit contract so Port can tell the
difference between a merely admission-ready node and a node that can actually
launch x86_64 Firecracker/PVM workloads.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Port model, doctor, and runtime preflight define the
  prepared-node x86_64 PVM host-kit inputs explicitly, including patched
  Firecracker binary selection and required host prerequisites.
- [ ] [SRS-01/AC-02] Missing or malformed prepared-node PVM host-kit state
  fails with explicit host-kit detail instead of generic runtime launch errors.
