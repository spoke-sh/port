---
id: 1vz3kt000
title: Define Hosted Machine Inventory Contract
type: feat
status: done
created_at: 2026-03-07T18:20:31
updated_at: 2026-03-07T18:59:29
scope: 1vz3ck000/1vz3j0000
started_at: 2026-03-07T18:26:58
submitted_at: 2026-03-07T18:59:20
completed_at: 2026-03-07T18:59:29
---

# Define Hosted Machine Inventory Contract

## Summary

Define the first hosted machine inventory and lifecycle contract so the current
`machine list|status|stop` verbs can target local runtime roots or future
node-agent-backed ownership without changing the operator model.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] Port publishes implementation-ready lifecycle and inventory contracts for local versus hosted ownership, including how machine status is sourced and routed. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3kt000/verify-ac-1.sh, proof: ac-1.log-->
