---
# system-managed
id: VFhLmKkbA
status: done
created_at: 2026-04-02T21:17:48
updated_at: 2026-04-02T21:37:05
# authored
title: Ship AWS PVM Nix Host Kit Surface
type: feat
operator-signal:
scope: VFhLhfrqk/VFhLjViAG
index: 1
started_at: 2026-04-02T21:24:26
submitted_at: 2026-04-02T21:37:04
completed_at: 2026-04-02T21:37:05
---

# Ship AWS PVM Nix Host Kit Surface

## Summary

Export Port's AWS x86_64 PVM host-kit contract as a first-class flake module
and companion package surface, then document and verify the downstream AMI
handoff.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `port.nixosModules.aws-pvm-host` is exported and a downstream `nixosSystem { modules = [ port.nixosModules.aws-pvm-host ]; }` evaluates successfully. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Port exports a companion `firecracker-pvm-host-kit` package/metadata surface whose canonical host-kit identity matches the AWS x86_64 PVM host-kit contract already used by `prepare-pvm-node`. <!-- [SRS-02/AC-02] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-03/AC-03] The exported module configures the host contract needed for the canonical local AWS PVM doctor surface: Linux/x86_64 posture, required boot args including `pti=off`, and the canonical `firecracker-pvm` path/env surface. <!-- [SRS-03/AC-03] verify: manual, proof: ac-3.log -->
<!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-04/AC-04] Port docs show the supported downstream AMI handoff using the Port-owned module/package surface instead of a downstream repo-local host-kit module. <!-- [SRS-04/AC-04] verify: manual, proof: ac-4.log -->
<!-- verify: manual, SRS-NFR-01:start:end, proof: ac-5.log -->
- [x] [SRS-NFR-01/AC-05] The Nix export stays mechanically aligned with Port-owned canonical host-kit data so the flake surface does not drift from the runtime/readiness contract. <!-- [SRS-NFR-01/AC-05] verify: manual, proof: ac-5.log -->
<!-- verify: manual, SRS-NFR-02:start:end, proof: ac-6.log -->
- [x] [SRS-NFR-02/AC-06] The implementation and docs keep the scope truthful: host-kit definition is in Port, but AMI import/export and downstream orchestration remain explicitly outside Port. <!-- [SRS-NFR-02/AC-06] verify: manual, proof: ac-6.log -->
