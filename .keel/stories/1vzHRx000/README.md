---
id: 1vzHRx000
title: Gate Hosted Pvm Placement
type: feat
status: done
created_at: 2026-03-08T09:57:53
updated_at: 2026-03-08T11:45:39
scope: 1vz3ck000/1vzHPo000
started_at: 2026-03-08T10:16:00
completed_at: 2026-03-08T11:45:39
---

# Gate Hosted Pvm Placement

## Summary

Extend the hosted control-plane and node-agent path so PVM machines can only be
placed onto nodes that advertise the required x86_64 PVM capability.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-03/AC-01] Hosted protocol and runtime behavior expose node PVM readiness and use it when resolving machine placement or denial reasons. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRx000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-02] Hosted `port machine launch|status` proofs reject unplaceable PVM machines without regressing standard hosted machine workflows. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzHRx000/verify-ac-2.sh, proof: ac-2.log -->
