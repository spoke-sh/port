---
# system-managed
id: VFgcoUMUb
status: done
created_at: 2026-04-02T18:19:12
updated_at: 2026-04-02T18:33:47
# authored
title: Define AWS PVM Prepared Host Contract
type: feat
operator-signal:
scope: VFgcPDfEj/VFgclbAzD
index: 1
started_at: 2026-04-02T18:25:37
submitted_at: 2026-04-02T18:33:27
completed_at: 2026-04-02T18:33:47
---

# Define AWS PVM Prepared Host Contract

## Summary

Define the implementation-ready AWS hosted PVM prepared-host contract for
`cloud-aws`, keeping the lane explicitly tied to x86_64 AWS Linux, custom
kernel and boot requirements, patched `firecracker-pvm`, and dedicated PVM
artifacts without generic-node substitution.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] Port publishes an explicit `cloud-aws` x86_64 prepared-host contract that captures the required host kit, `pti=off`, patched `firecracker-pvm`, and PVM artifact prerequisites. <!-- [SRS-01/AC-01] verify: manual, proof: ac-2.log-->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-01/AC-02] The contract keeps the scope boundary explicit: x86_64 AWS hosted PVM only, with no arm64 or non-AWS support claim. <!-- [SRS-NFR-01/AC-02] verify: manual, proof: ac-4.log-->
