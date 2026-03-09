---
id: 1vzfV4000
title: Implement Service Supervision And Health State
type: feat
status: icebox
created_at: 2026-03-09T11:38:42
updated_at: 2026-03-09T11:39:32
scope: 1vzfT4000/1vzfTm000
---

# Implement Service Supervision And Health State

## Summary

Extend Port's runtime owner into a real managed-process supervisor that
enforces restart policy, tracks restart and exit state, and reports service
health through the canonical local and hosted status surfaces.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] Port supervises managed service or sandbox processes according to the selected restart policy and records restart count, last exit detail, and health state under the existing runtime owner. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime service_supervision && cargo test -q -p port-runtime service_health', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Local and hosted `port service status` project the same restart and health state without introducing a second service runtime model. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli service_status && cargo test -q -p port-sdk service_status', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->
