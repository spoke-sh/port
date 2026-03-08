---
id: 1vz3l2000
title: Define AVF Execution Contract
type: feat
status: done
created_at: 2026-03-07T18:20:40
updated_at: 2026-03-07T19:11:32
scope: 1vz3ck000/1vz3j0000
started_at: 2026-03-07T19:06:55
submitted_at: 2026-03-07T19:11:27
completed_at: 2026-03-07T19:11:32
---

# Define AVF Execution Contract

## Summary

Define the first Apple Virtualization Framework execution contract for Port,
covering launch ownership, guest transport mapping, and operator workflow on
macOS.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] Port defines the AVF runtime contract, including how canonical lifecycle and guest operations map onto AVF-specific primitives. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3l2000/verify-ac-1.sh, proof: ac-1.log-->
<!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] [SRS-05/AC-01] The story produces an implementation-ready AVF follow-on slice with explicit docs and verification expectations for macOS operators and leaves the voyage with a coherent ordered implementation set. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3l2000/verify-ac-2.sh, proof: ac-2.log-->
