---
id: 1vz4gb000
title: Define Hosted Auth And API Contract
type: feat
status: done
created_at: 2026-03-07T19:20:09
updated_at: 2026-03-07T19:29:04
scope: 1vz4Yn000/1vz4cU000
started_at: 2026-03-07T19:22:58
submitted_at: 2026-03-07T19:29:01
completed_at: 2026-03-07T19:29:04
---

# Define Hosted Auth And API Contract

## Summary

Define the first hosted control-plane endpoint and token-auth contract so Port
can model authenticated hosted targets without inventing a second operator
surface.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-01/AC-01] Port publishes implementation-ready hosted endpoint and token-auth contracts in the shared model, including how hosted API identity maps onto the canonical CLI target surface. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gb000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] README, hosted docs, and CLI help describe the hosted auth contract and clearly distinguish the modeled control-plane path from shipped local behavior. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4gb000/verify-ac-2.sh, proof: ac-2.log -->
