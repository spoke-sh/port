---
id: 1vzRov000
title: Define Managed Service Execution Contract
type: feat
status: backlog
created_at: 2026-03-08T21:02:17
updated_at: 2026-03-08T21:04:26
scope: 1vz4Yn000/1vzRnO000
---

# Define Managed Service Execution Contract

## Summary

Define the shared contract for hosted service and sandbox execution so the
runtime, guest agent, CLI, and SDK all target one canonical lifecycle model.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Port defines the managed service execution contract, route vocabulary, and runtime-state model without adding hosted-only service verbs or a second runtime surface. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRov000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-01:end, proof: ac-2.log -->
- [ ] [SRS-01/AC-02] The contract keeps one canonical `port service` vocabulary across local and hosted lanes and makes the no-hosted-only-verb boundary explicit before implementation begins. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRov000/verify-ac-2.sh, proof: ac-2.log -->
