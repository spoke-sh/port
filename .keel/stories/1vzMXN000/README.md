---
id: 1vzMXN000
title: Define Streamed Guest Session Contract
type: feat
status: done
created_at: 2026-03-08T15:23:49
updated_at: 2026-03-08T15:40:14
scope: 1vzMVF000/1vzMVY000
started_at: 2026-03-08T15:26:48
completed_at: 2026-03-08T15:40:14
---

# Define Streamed Guest Session Contract

## Summary

Define the shared protocol, hosted attach contract, and SDK surface for
streamed PTY, log-follow, copy, and forward so the implementation stories can
land on one canonical guest-control contract.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] The shared guest protocol and hosted route contract define streamed session lifecycle semantics for attach, payload, EOF, exit, and failure without introducing a second guest command family. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXN000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-01/AC-02] The contract makes stream ownership and termination behavior explicit enough for CLI, runtime, node-agent, and SDK callers to implement deterministic cleanup. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXN000/verify-ac-2.sh, proof: ac-2.log -->
