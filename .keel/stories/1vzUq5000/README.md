---
id: 1vzUq5000
title: Define Durable Hosted Registry Contract
type: feat
status: done
created_at: 2026-03-09T00:15:41
updated_at: 2026-03-09T01:33:49
scope: 1vzUnI000/1vzUoK000
started_at: 2026-03-09T01:31:36
completed_at: 2026-03-09T01:33:49
---

# Define Durable Hosted Registry Contract

## Summary

Define the shared durable hosted registry contract in Port’s model and hosted
protocol so the control plane can represent persisted node registration,
freshness, and imported inventory provenance through one canonical identity
namespace.

## Acceptance Criteria

<!-- verify: command, SRS-06:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-01] Shared model and hosted protocol types represent persisted hosted node registration records with node identity, endpoint, registration time, last-seen time, and freshness state. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-06:start:end, proof: ac-2.log -->
- [x] [SRS-06/AC-02] Shared contracts represent imported inventory records and provenance metadata that merge onto canonical configured node names without introducing a second fleet namespace. <!-- [SRS-06/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-06:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-03] Contract validation and serialization errors include explicit durable-registry or import context together with affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-06/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-3.sh, proof: ac-3.log -->
<!-- verify: command, SRS-06:end, proof: ac-3.log -->
