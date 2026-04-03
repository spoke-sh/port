---
# system-managed
id: VFhIjGbxm
status: done
created_at: 2026-04-02T21:05:41
updated_at: 2026-04-02T21:15:41
# authored
title: Rewrite Foundational AWS PVM Production Narrative
type: feat
operator-signal:
scope: VFhICYohO/VFhIVMVEF
index: 1
started_at: 2026-04-02T21:06:03
submitted_at: 2026-04-02T21:15:39
completed_at: 2026-04-02T21:15:41
---

# Rewrite Foundational AWS PVM Production Narrative

## Summary

Rewrite Port's foundational and user-facing docs so AWS x86_64 hosted
Firecracker/PVM reads like one clear production-oriented narrative instead of a
set of fragmented proof notes.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] Foundational docs direct readers to one canonical AWS production path instead of scattering the contract across repeated summaries. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] The docs explain the AWS x86_64 hosted Firecracker/PVM lane in operational terms, including host kit, artifact kit, `prepare-pvm-node`, and canonical machine launch/status/stop flow. <!-- [SRS-02/AC-02] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-03/AC-03] The docs distinguish the AWS PVM lane from the hosted standard lane and from repo-local proof harnesses without hiding those secondary surfaces. <!-- [SRS-03/AC-03] verify: manual, proof: ac-3.log -->
<!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-04/AC-04] The docs preserve explicit provider-aware and architecture-aware boundaries, including missing-host-kit or missing-artifact failure posture, no silent fallback, no arm64 PVM claim, and no implied GCP/Azure inheritance. <!-- [SRS-04/AC-04] verify: manual, proof: ac-4.log -->
<!-- verify: manual, SRS-NFR-01:start:end, proof: ac-5.log -->
- [x] [SRS-NFR-01/AC-05] The refreshed docs reduce duplication and improve navigation through deliberate cross-links around the AWS/PVM path. <!-- [SRS-NFR-01/AC-05] verify: manual, proof: ac-5.log -->
<!-- verify: manual, SRS-NFR-02:start:end, proof: ac-6.log -->
- [x] [SRS-NFR-02/AC-06] Public docs and foundational docs describe the same AWS production posture and boundaries. <!-- [SRS-NFR-02/AC-06] verify: manual, proof: ac-6.log -->
