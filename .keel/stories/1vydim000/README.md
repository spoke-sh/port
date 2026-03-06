---
id: 1vydim000
title: Implement Local Firecracker Launch
type: feat
status: backlog
created_at: 2026-03-06T14:32:36
updated_at: 2026-03-06T14:40:27
scope: 1vydg7000/1vydgL000
---

# Implement Local Firecracker Launch

## Summary

Implement the Linux host preflight and the first real local Firecracker launch
path, including runtime state directories, Firecracker config generation, and a
recorded end-to-end proof.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `port doctor` validates Linux host support, `/dev/kvm`, Firecracker availability, and required tooling with actionable errors.
- [ ] [SRS-03/AC-01] `port machine launch` boots a Firecracker VM from Port-managed artifacts and records runtime metadata and log locations.
- [ ] [SRS-03/AC-02] Automated tests cover Firecracker config generation and runtime path/state behavior without requiring KVM in every test.
- [ ] [SRS-02/AC-02] Failure modes for unsupported hosts and launch failures preserve actionable diagnostics.
