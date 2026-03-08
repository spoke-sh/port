---
id: 1vzGrf000
title: Materialize PVM Artifact Variants
type: feat
status: in-progress
created_at: 2026-03-08T09:20:23
updated_at: 2026-03-08T09:37:01
scope: 1vz3ck000/1vzGo0000
started_at: 2026-03-08T09:37:01
---

# Materialize PVM Artifact Variants

## Summary

Add dedicated `x86_64/firecracker/pvm` kernel and guest-image build/validate
variants so artifact commands can materialize the PVM lane without reusing the
standard Firecracker artifacts.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-03/AC-01] `port artifacts build|validate` supports `x86_64/firecracker/pvm` kernel and guest-image variants through dedicated scripts or contracts, and fails immediately when the PVM variant is missing. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrf000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-02] PVM artifact selection remains deterministic and separate from the standard Firecracker lane in code, output paths, and validation behavior. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzGrf000/verify-ac-2.sh, proof: ac-2.log -->
