---
id: 1vzRpC000
title: Implement Guest-Agent Managed Process Supervisor
type: feat
status: backlog
created_at: 2026-03-08T21:02:34
updated_at: 2026-03-08T21:04:26
scope: 1vz4Yn000/1vzRnO000
---

# Implement Guest-Agent Managed Process Supervisor

## Summary

Extend the guest agent with a managed-process supervisor that can launch,
inspect, and stop service and sandbox commands with stable runtime metadata.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] The guest agent can start, list/status, and stop managed service or sandbox processes while preserving the existing guest transport and non-service operations. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpC000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:continues, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Managed processes capture operator-visible runtime metadata, including live state, exit status, and log paths, while keeping injected secret values out of status responses and logs. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpC000/verify-ac-2.sh, proof: ac-2.log -->
