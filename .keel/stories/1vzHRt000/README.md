---
id: 1vzHRt000
title: Model Pvm Node Capability Contract
type: feat
status: done
created_at: 2026-03-08T09:57:49
updated_at: 2026-03-08T10:08:54
scope: 1vz3ck000/1vzHPo000
started_at: 2026-03-08T10:00:09
submitted_at: 2026-03-08T10:08:49
completed_at: 2026-03-08T10:08:54
---

# Model Pvm Node Capability Contract

## Summary

Add one canonical x86_64 PVM capability contract that can be resolved from the
local Firecracker lane and from hosted node inventory without implying
`aarch64` support.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `port-model` exposes explicit x86_64 PVM capability state for local and hosted execution, and the sample config serializes that state without widening the planned lane beyond `x86_64`. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRt000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-01/AC-02] Hosted protocol or SDK contracts can carry the same capability state so hosted placement logic can consume it without inventing a second PVM vocabulary. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRt000/verify-ac-2.sh, proof: ac-2.log -->
