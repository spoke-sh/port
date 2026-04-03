---
# system-managed
id: VFhYInU7j
status: done
created_at: 2026-04-02T22:07:33
updated_at: 2026-04-02T22:16:20
# authored
title: Fix AWS PVM Host Kit Kernel Default
type: fix
operator-signal:
started_at: 2026-04-02T22:07:33
submitted_at: 2026-04-02T22:16:19
completed_at: 2026-04-02T22:16:20
---

# Fix AWS PVM Host Kit Kernel Default

## Summary

Repair the exported AWS PVM Nix host-kit module so downstream image builds do
not fail on an invalid default kernel derivation, then align the AWS handoff
docs with the real direct-flake consumer path.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `nixosModules.aws-pvm-host` no longer injects a mismatched `modDirVersion` into its default kernel package set, and the flake proof records matching kernel version and module directory version values. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] A downstream `infra` AMI build against the repaired Port checkout gets past the previous `linux-port-pvm` / `modDirVersion 6.12.0-port-pvm should be 6.12.78` failure. <!-- [SRS-02/AC-02] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-03/AC-03] AWS production docs describe the current downstream seam accurately: direct Port flake import plus `--override-input port path:/...` for local checkout validation, not the removed module-path env handoff. <!-- [SRS-03/AC-03] verify: manual, proof: ac-3.log -->
