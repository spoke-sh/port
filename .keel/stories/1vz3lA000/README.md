---
id: 1vz3lA000
title: Plan Pvm Host Kit
type: feat
status: in-progress
created_at: 2026-03-07T18:20:48
updated_at: 2026-03-07T18:59:53
scope: 1vz3ck000/1vz3j0000
started_at: 2026-03-07T18:59:53
---

# Plan Pvm Host Kit

## Summary

Define the first x86_64 PVM host-kit and artifact-kit contract for Port,
including host kernel, VMM, artifact variants, validation, and explicit
operator prerequisites while keeping arm64 Firecracker/PVM research-only.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-01] Port publishes an implementation-ready host-kit and artifact-kit contract for the x86_64 PVM lane, including prepared host components and validation expectations. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3lA000/verify-ac-1.sh, proof: ac-1.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-03/AC-02] The story records an explicit x86_64 keep / arm64 research-only boundary with operator-visible implications and follow-on implementation work. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3lA000/verify-ac-2.sh, proof: ac-2.log-->
