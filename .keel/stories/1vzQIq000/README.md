---
id: 1vzQIq000
title: Define Hosted Detached Forward Contract
type: feat
status: in-progress
created_at: 2026-03-08T19:25:04
updated_at: 2026-03-08T19:27:50
scope: 1vzETR000/1vzQEj000
started_at: 2026-03-08T19:27:50
---

# Define Hosted Detached Forward Contract

## Summary

Define the shared hosted route, payload, and SDK contract for detached guest
forward lifecycle operations so implementation stories can land on one
canonical API surface.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] The shared hosted contract defines detached forward start, list, and stop request/response shapes, including named session identity, without inventing a second guest command family. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIq000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-01/AC-02] The detached forward contract preserves enough machine, node, runtime-root, and forward-name context for later routing and operator-facing failures. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIq000/verify-ac-2.sh, proof: ac-2.log -->
