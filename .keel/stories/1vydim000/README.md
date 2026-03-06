---
id: 1vydim000
title: Implement Local Firecracker Launch
type: feat
status: done
created_at: 2026-03-06T14:32:36
updated_at: 2026-03-06T15:02:59
scope: 1vydg7000/1vydgL000
started_at: 2026-03-06T14:48:57
submitted_at: 2026-03-06T15:02:51
completed_at: 2026-03-06T15:02:59
---

# Implement Local Firecracker Launch

## Summary

Implement the Linux host preflight and the first real local Firecracker launch
path, including runtime state directories, Firecracker config generation, and a
recorded end-to-end proof.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] `port doctor` validates Linux host support, `/dev/kvm`, Firecracker availability, and required tooling with actionable errors. <!-- [SRS-02/AC-01] verify: cargo run -p port-cli -- --config /tmp/port-proof/port.toml doctor, proof: ac-2.log-->
- [x] [SRS-02/AC-02] Failure modes for unsupported hosts and launch failures preserve actionable diagnostics. <!-- [SRS-02/AC-02] verify: bash /tmp/port-proof/verify-launch-failure.sh, proof: ac-3.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-4.log-->
- [x] [SRS-03/AC-01] `port machine launch` boots a Firecracker VM from Port-managed artifacts and records runtime metadata and log locations. <!-- [SRS-03/AC-01] verify: bash /tmp/port-proof/verify-launch-success.sh, proof: ac-5.log-->
- [x] [SRS-03/AC-02] Automated tests cover Firecracker config generation and runtime path/state behavior without requiring KVM in every test. <!-- [SRS-03/AC-02] verify: cargo test, proof: ac-6.log-->
